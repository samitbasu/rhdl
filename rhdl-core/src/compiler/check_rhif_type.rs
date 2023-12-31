// Check a RHIF object for type correctness.

use std::collections::HashSet;

use crate::{
    compiler::ty::{self, ty_array, ty_bool, ty_named_field, ty_path, ty_unnamed_field, Bits, Ty},
    rhif,
    rhif::rhif_spec::{
        AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Discriminant, Enum,
        Exec, If, Index, OpCode, Repeat, Slot, Struct, Tuple, Unary,
    },
    rhif::Object,
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
            Err(anyhow!(
                "type mismatch while checking RHIF type correctness: {:?} != {:?}",
                a,
                b
            ))
        }
    };
    for block in &obj.blocks {
        for op in &block.ops {
            eprintln!("check op: {:?}", op);
            match op {
                OpCode::Binary(Binary {
                    op:
                        AluBinary::Add
                        | AluBinary::Sub
                        | AluBinary::Mul
                        | AluBinary::BitAnd
                        | AluBinary::BitOr
                        | AluBinary::BitXor,
                    lhs,
                    arg1,
                    arg2,
                }) => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                    eq_types(slot_type(lhs)?, slot_type(arg2)?)?;
                }
                OpCode::Binary(Binary {
                    op: AluBinary::Shl | AluBinary::Shr,
                    lhs,
                    arg1,
                    arg2,
                }) => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                    ensure!(slot_type(arg2)?.is_unsigned());
                }
                OpCode::Binary(Binary {
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
                }) => {
                    eq_types(slot_type(arg1)?, slot_type(arg2)?)?;
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Unary(Unary {
                    op: AluUnary::Not | AluUnary::Neg,
                    lhs,
                    arg1,
                }) => {
                    eq_types(slot_type(lhs)?, slot_type(arg1)?)?;
                }
                OpCode::Unary(Unary {
                    op: AluUnary::All | AluUnary::Any | AluUnary::Xor,
                    lhs,
                    arg1: _,
                }) => {
                    eq_types(slot_type(lhs)?, ty_bool())?;
                }
                OpCode::Unary(Unary {
                    op: AluUnary::Signed,
                    lhs,
                    arg1,
                }) => {
                    let arg1_ty = slot_type(arg1)?;
                    let Ty::Const(Bits::Unsigned(x)) = arg1_ty else {
                        bail!("signed operator requires unsigned argument")
                    };
                    eq_types(slot_type(lhs)?, ty::ty_signed(x))?;
                }
                OpCode::Unary(Unary {
                    op: AluUnary::Unsigned,
                    lhs,
                    arg1,
                }) => {
                    let arg1_ty = slot_type(arg1)?;
                    let Ty::Const(Bits::Signed(x)) = arg1_ty else {
                        bail!("unsigned operator requires signed argument")
                    };
                    eq_types(slot_type(lhs)?, ty::ty_bits(x))?;
                }
                OpCode::Array(Array { lhs, elements }) => eq_types(
                    slot_type(lhs)?,
                    ty_array(slot_type(&elements[0])?, elements.len()),
                )?,
                OpCode::If(If {
                    lhs: _,
                    cond,
                    then_branch: _,
                    else_branch: _,
                }) => {
                    eq_types(slot_type(cond)?, ty::ty_bool())?;
                }
                OpCode::Assign(Assign { lhs, rhs, path }) => {
                    eq_types(ty_path(slot_type(lhs)?, path)?, slot_type(rhs)?)?;
                }
                OpCode::Tuple(Tuple { lhs, fields }) => {
                    let ty = fields.iter().map(slot_type).collect::<Result<Vec<_>>>()?;
                    eq_types(slot_type(lhs)?, ty::ty_tuple(ty))?;
                }
                OpCode::Index(Index { lhs, arg, path }) => {
                    let ty = slot_type(arg)?;
                    let ty = ty_path(ty, path)?;
                    eq_types(ty, slot_type(lhs)?)?;
                    for slot in path.dynamic_slots() {
                        ensure!(slot_type(slot)?.is_unsigned(), "index must be unsigned");
                    }
                }
                OpCode::Struct(Struct {
                    lhs,
                    fields,
                    rest,
                    template,
                }) => {
                    let ty = slot_type(lhs)?;
                    eq_types(ty.clone(), template.kind.clone().into())?;
                    if let Some(rest) = rest {
                        let rest_ty = slot_type(rest)?;
                        eq_types(ty.clone(), rest_ty)?;
                    }
                    for field in fields {
                        match &field.member {
                            rhif::rhif_spec::Member::Named(name) => {
                                let ty = ty_named_field(&ty, name)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                            rhif::rhif_spec::Member::Unnamed(index) => {
                                let ty = ty_unnamed_field(&ty, *index as usize)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                        }
                    }
                }
                OpCode::Enum(Enum {
                    lhs,
                    fields,
                    template,
                }) => {
                    let ty = slot_type(lhs)?;
                    let Ty::Enum(enum_ty) = &ty else {
                        bail!("expected enum type")
                    };
                    let payload_kind = enum_ty.payload.kind.clone();
                    let discriminant_value = template.discriminant()?.as_i64()?;
                    let variant_kind = payload_kind.lookup_variant(discriminant_value)?;
                    let variant_ty = variant_kind.into();
                    for field in fields {
                        match &field.member {
                            rhif::rhif_spec::Member::Named(name) => {
                                let ty = ty_named_field(&variant_ty, name)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                            rhif::rhif_spec::Member::Unnamed(index) => {
                                let ty = ty_unnamed_field(&variant_ty, *index as usize)?;
                                eq_types(slot_type(&field.value)?, ty)?;
                            }
                        }
                    }
                }
                OpCode::Repeat(Repeat { lhs, value, len }) => {
                    let ty = slot_type(lhs)?;
                    let Ty::Array(array_ty) = &ty else {
                        bail!("expected array type")
                    };
                    eq_types(slot_type(value)?, array_ty[0].clone())?;
                    eq_types(ty.clone(), ty_array(array_ty[0].clone(), *len))?;
                }
                OpCode::Comment(_) => {}
                OpCode::Return => {
                    break;
                }
                OpCode::Block(_) => {}
                OpCode::Case(Case {
                    discriminant: expr,
                    table,
                }) => {
                    let arg_ty = slot_type(expr)?;
                    let mut discriminants: HashSet<Vec<bool>> = Default::default();
                    for (entry_test, _entry_body) in table {
                        match entry_test {
                            CaseArgument::Constant(constant) => {
                                if discriminants.contains(&constant.bits) {
                                    bail!("Match contains a duplicate discriminant {constant}, which is not allowed in RHDL");
                                } else {
                                    discriminants.insert(constant.bits.clone());
                                }
                                let constant_ty = constant.kind.clone().into();
                                eq_types(arg_ty.clone(), constant_ty)?;
                            }
                            CaseArgument::Wild => {}
                        }
                    }
                }
                OpCode::Discriminant(Discriminant { lhs, arg }) => {
                    let arg_ty = slot_type(arg)?;
                    if let Ty::Enum(enum_ty) = &arg_ty {
                        eq_types(slot_type(lhs)?, *enum_ty.discriminant.clone())?;
                    } else {
                        eq_types(slot_type(lhs)?, arg_ty)?;
                    }
                }
                OpCode::Exec(Exec { lhs, id, args }) => {
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
                OpCode::AsBits(Cast { lhs, arg: _, len }) => {
                    eq_types(slot_type(lhs)?, ty::ty_bits(*len))?;
                }
                OpCode::AsSigned(Cast { lhs, arg: _, len }) => {
                    eq_types(slot_type(lhs)?, ty::ty_signed(*len))?;
                }
            }
        }
    }
    Ok(())
}
