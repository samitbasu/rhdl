// Check a RHIF object for type correctness.

use crate::{
    object::Object,
    rhif::{AluBinary, AluUnary, CaseArgument, OpCode, Slot},
    ty::{
        self, ty_array, ty_array_base, ty_as_ref, ty_bool, ty_named_field, ty_unnamed_field, Bits,
        Ty,
    },
};
use anyhow::{anyhow, bail};
use anyhow::{ensure, Result};

pub fn check_type_correctness(obj: &Object) -> Result<()> {
    let slot_type = |slot: &Slot| -> Result<Ty> {
        if matches!(*slot, Slot::Empty) {
            return Ok(ty::ty_empty());
        }
        obj.ty
            .get(slot)
            .cloned()
            .ok_or(anyhow!("slot {:?} not found", slot))
    };
    let eq_types = |a, b| -> Result<()> {
        if a == b {
            Ok(())
        } else {
            Err(anyhow!("type mismatch: {:?} != {:?}", a, b))
        }
    };
    for block in &obj.blocks {
        for op in &block.ops {
            eprintln!("op: {:?}", op);
            match op {
                OpCode::Binary {
                    op:
                        AluBinary::Add
                        | AluBinary::Sub
                        | AluBinary::Mul
                        | AluBinary::BitAnd
                        | AluBinary::BitOr
                        | AluBinary::BitXor
                        | AluBinary::Shl
                        | AluBinary::Shr,
                    lhs,
                    arg1,
                    arg2,
                } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                    eq_types(slot_type(lhs)?, slot_type(arg2)?)?;
                }
                OpCode::Binary {
                    op: AluBinary::And | AluBinary::Or,
                    lhs,
                    arg1,
                    arg2,
                } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                    eq_types(slot_type(lhs)?, slot_type(arg2)?)?;
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Binary {
                    op:
                        AluBinary::Eq
                        | AluBinary::Ge
                        | AluBinary::Gt
                        | AluBinary::Le
                        | AluBinary::Lt
                        | AluBinary::Ne,
                    lhs,
                    arg1,
                    arg2,
                } => {
                    eq_types(slot_type(arg1)?, slot_type(arg2)?)?;
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Unary {
                    op: AluUnary::Not | AluUnary::Neg,
                    lhs,
                    arg1,
                } => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                }
                OpCode::Unary {
                    op: AluUnary::All | AluUnary::Any | AluUnary::Xor,
                    lhs,
                    arg1,
                } => {
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Unary {
                    op: AluUnary::Signed,
                    lhs,
                    arg1,
                } => {
                    let arg1_ty = slot_type(arg1)?;
                    let Ty::Const(Bits::Unsigned(x)) = arg1_ty else {
                        bail!("signed operator requires unsigned argument")
                    };
                    eq_types(slot_type(lhs)?, ty::ty_signed(x))?;
                }
                OpCode::Unary {
                    op: AluUnary::Unsigned,
                    lhs,
                    arg1,
                } => {
                    let arg1_ty = slot_type(arg1)?;
                    let Ty::Const(Bits::Signed(x)) = arg1_ty else {
                        bail!("unsigned operator requires signed argument")
                    };
                    eq_types(slot_type(lhs)?, ty::ty_bits(x))?;
                }
                OpCode::Array { lhs, elements } => eq_types(
                    slot_type(lhs)?,
                    ty_array(slot_type(&elements[0])?, elements.len()),
                )?,
                OpCode::If {
                    lhs,
                    cond,
                    then_branch,
                    else_branch,
                } => {
                    eq_types(slot_type(cond)?, ty::ty_bool())?;
                }
                OpCode::Index { lhs, arg, index } => {
                    eq_types(slot_type(lhs)?, ty_array_base(&slot_type(arg)?)?)?;
                }
                OpCode::Ref { lhs, arg } => {
                    eq_types(slot_type(lhs)?, ty_as_ref(slot_type(arg)?))?;
                }
                OpCode::Assign { lhs, rhs } => {
                    eq_types(slot_type(lhs)?, ty_as_ref(slot_type(rhs)?))?;
                }
                OpCode::Copy { lhs, rhs } => {
                    eq_types(slot_type(lhs)?, slot_type(rhs)?)?;
                }
                OpCode::Tuple { lhs, fields } => {
                    let ty = fields.iter().map(slot_type).collect::<Result<Vec<_>>>()?;
                    eq_types(slot_type(lhs)?, ty::ty_tuple(ty))?;
                }
                OpCode::Field { lhs, arg, member } => {
                    let ty = slot_type(arg)?;
                    match member {
                        crate::rhif::Member::Named(name) => {
                            let ty = ty_named_field(&ty, name)?;
                            eq_types(slot_type(lhs)?, ty)?;
                        }
                        crate::rhif::Member::Unnamed(index) => {
                            let ty = ty_unnamed_field(&ty, *index as usize)?;
                            eq_types(slot_type(lhs)?, ty)?;
                        }
                    }
                }
                OpCode::FieldRef { lhs, arg, member } => {
                    let ty = slot_type(arg)?;
                    match member {
                        crate::rhif::Member::Named(name) => {
                            let ty = ty_named_field(&ty, name)?;
                            eq_types(slot_type(lhs)?, ty_as_ref(ty))?;
                        }
                        crate::rhif::Member::Unnamed(index) => {
                            let ty = ty_unnamed_field(&ty, *index as usize)?;
                            eq_types(slot_type(lhs)?, ty_as_ref(ty))?;
                        }
                    }
                }
                OpCode::IndexRef { lhs, arg, index } => {
                    let ty = slot_type(arg)?;
                    let ty = ty_array_base(&ty)?;
                    eq_types(slot_type(lhs)?, ty_as_ref(ty))?;
                }
                OpCode::Struct {
                    lhs,
                    path,
                    fields,
                    rest,
                } => {
                    let ty = slot_type(lhs)?;
                    if let Some(rest) = rest {
                        let rest_ty = slot_type(rest)?;
                        eq_types(ty.clone(), rest_ty)?;
                    }
                    for field in fields {
                        match &field.member {
                            crate::rhif::Member::Named(name) => {
                                let ty = ty_named_field(&ty, name)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                            crate::rhif::Member::Unnamed(index) => {
                                let ty = ty_unnamed_field(&ty, *index as usize)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                        }
                    }
                }
                OpCode::Enum {
                    lhs,
                    path,
                    discriminant,
                    fields,
                } => {
                    let ty = slot_type(lhs)?;
                    let Ty::Enum(enum_ty) = &ty else {
                        bail!("expected enum type")
                    };
                    let payload_kind = enum_ty.payload.kind.clone();
                    let discriminant_value = obj.literal(*discriminant)?;
                    let discriminant_value = discriminant_value.as_i64()?;
                    let variant_kind = payload_kind.lookup_variant(discriminant_value)?;
                    let variant_ty = variant_kind.into();
                    for field in fields {
                        match &field.member {
                            crate::rhif::Member::Named(name) => {
                                let ty = ty_named_field(&variant_ty, name)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                            crate::rhif::Member::Unnamed(index) => {
                                let ty = ty_unnamed_field(&variant_ty, *index as usize)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                        }
                    }
                }
                OpCode::Repeat { lhs, value, len } => {
                    let ty = slot_type(lhs)?;
                    let Ty::Array(array_ty) = &ty else {
                        bail!("expected array type")
                    };
                    let len_value = obj.literal(*len)?;
                    let len_value = len_value.as_i64()?;
                    eq_types(slot_type(value)?, array_ty[0].clone())?;
                    eq_types(
                        ty.clone(),
                        ty_array(array_ty[0].clone(), len_value as usize),
                    )?;
                }
                OpCode::Comment(_) => {}
                OpCode::Return { result } => {
                    if let Some(result) = result {
                        eq_types(slot_type(result)?, slot_type(&obj.return_slot)?)?;
                    }
                }
                OpCode::Block(_) => {}
                OpCode::Payload {
                    lhs,
                    arg,
                    discriminant,
                } => {
                    let ty = slot_type(arg)?;
                    let Ty::Enum(enum_ty) = &ty else {
                        bail!(format!("expected enum type {}", ty));
                    };
                    let payload_kind = enum_ty.payload.kind.clone();
                    let discriminant_value = obj.literal(*discriminant)?;
                    let discriminant_value = discriminant_value.as_i64()?;
                    let variant_kind = payload_kind.lookup_variant(discriminant_value)?;
                    let variant_ty = variant_kind.into();
                    eq_types(slot_type(lhs)?, variant_ty)?;
                }
                OpCode::Case {
                    discriminant: expr,
                    table,
                } => {
                    let arg_ty = slot_type(expr)?;
                    for (entry_test, entry_body) in table {
                        match entry_test {
                            CaseArgument::Literal(lit) => {
                                let lit_ty = slot_type(lit)?;
                                eq_types(arg_ty.clone(), lit_ty)?;
                            }
                            CaseArgument::Wild => {}
                        }
                    }
                }
                OpCode::Discriminant { lhs, arg } => {
                    let arg_ty = slot_type(arg)?;
                    if let Ty::Enum(enum_ty) = &arg_ty {
                        eq_types(slot_type(lhs)?, *enum_ty.discriminant.clone())?;
                    } else {
                        eq_types(slot_type(lhs)?, arg_ty)?;
                    }
                }
                OpCode::Exec { lhs, id, args } => {
                    // Get the function signature.
                    let signature = obj.externals[id.0].signature.clone();
                    eq_types(slot_type(lhs)?, signature.ret.into())?;
                    for (arg, param) in args.iter().zip(signature.arguments.iter()) {
                        eq_types(slot_type(arg)?, param.clone().into())?;
                    }
                    ensure!(
                        args.len() == signature.arguments.len(),
                        "wrong number of arguments"
                    )
                }
                OpCode::AsBits { lhs, arg, len } => {
                    eq_types(slot_type(lhs)?, ty::ty_bits(*len))?;
                }
                OpCode::AsSigned { lhs, arg, len } => {
                    eq_types(slot_type(lhs)?, ty::ty_signed(*len))?;
                }
            }
        }
    }
    Ok(())
}
