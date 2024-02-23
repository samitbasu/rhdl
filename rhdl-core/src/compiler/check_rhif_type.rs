// Check a RHIF object for type correctness.

use std::collections::HashSet;

use crate::{
    path::{sub_kind, Path, PathElement},
    rhif::{
        self,
        spec::{
            AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Discriminant,
            Enum, Exec, Index, OpCode, Repeat, Select, Slot, Splice, Struct, Tuple, Unary,
        },
        Object,
    },
    Kind,
};
use anyhow::{anyhow, bail};
use anyhow::{ensure, Result};

use super::pass::Pass;

pub struct TypeCheckPass;

impl Pass for TypeCheckPass {
    fn name(&self) -> &'static str {
        "check_rhif_type"
    }
    fn description(&self) -> &'static str {
        "Check RHIF type correctness"
    }
    fn run(input: Object) -> Result<Object> {
        check_type_correctness(&input)?;
        Ok(input)
    }
}

// For type checking purposes, a dynamic path index
// cannot change the types of any of the parts of the
// expression.  So we replace all of the dynamic components
// with 0.
fn approximate_dynamic_paths(path: &Path) -> Path {
    path.iter()
        .map(|e| match e {
            PathElement::DynamicIndex(_) => PathElement::Index(0),
            _ => e.clone(),
        })
        .collect()
}

fn check_type_correctness(obj: &Object) -> Result<()> {
    let slot_type = |slot: &Slot| -> Result<Kind> {
        if matches!(*slot, Slot::Empty) {
            return Ok(Kind::Empty);
        }
        obj.kind
            .get(slot)
            .cloned()
            .ok_or(anyhow!("slot {:?} not found", slot))
    };
    let eq_kinds = |a, b| -> Result<()> {
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
    for op in &obj.ops {
        eprintln!("check op: {:?}", op);
        match op {
            OpCode::Noop => {}
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
                eq_kinds(slot_type(lhs)?, slot_type(arg1)?)?;
                eq_kinds(slot_type(lhs)?, slot_type(arg2)?)?;
            }
            OpCode::Binary(Binary {
                op: AluBinary::Shl | AluBinary::Shr,
                lhs,
                arg1,
                arg2,
            }) => {
                eq_kinds(slot_type(lhs)?, slot_type(arg1)?)?;
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
                eq_kinds(slot_type(arg1)?, slot_type(arg2)?)?;
                eq_kinds(slot_type(lhs)?, Kind::make_bool())?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::Not | AluUnary::Neg,
                lhs,
                arg1,
            }) => {
                eq_kinds(slot_type(lhs)?, slot_type(arg1)?)?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::All | AluUnary::Any | AluUnary::Xor,
                lhs,
                arg1: _,
            }) => {
                eq_kinds(slot_type(lhs)?, Kind::make_bool())?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::Signed,
                lhs,
                arg1,
            }) => {
                let arg1_ty = slot_type(arg1)?;
                let Kind::Bits(x) = arg1_ty else {
                    bail!("signed operator requires unsigned argument")
                };
                eq_kinds(slot_type(lhs)?, Kind::Signed(x))?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::Unsigned,
                lhs,
                arg1,
            }) => {
                let arg1_ty = slot_type(arg1)?;
                let Kind::Signed(x) = arg1_ty else {
                    bail!("unsigned operator requires signed argument")
                };
                eq_kinds(slot_type(lhs)?, Kind::make_bits(x))?;
            }
            OpCode::Array(Array { lhs, elements }) => eq_kinds(
                slot_type(lhs)?,
                Kind::make_array(slot_type(&elements[0])?, elements.len()),
            )?,
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                eq_kinds(slot_type(lhs)?, slot_type(true_value)?)?;
                eq_kinds(slot_type(lhs)?, slot_type(false_value)?)?;
                eq_kinds(slot_type(cond)?, Kind::make_bool())?;
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                eq_kinds(slot_type(lhs)?, slot_type(rhs)?)?;
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                path,
                subst,
            }) => {
                eq_kinds(
                    sub_kind(slot_type(lhs)?, &approximate_dynamic_paths(path))?,
                    slot_type(subst)?,
                )?;
                eq_kinds(slot_type(lhs)?, slot_type(orig)?)?;
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let ty = fields.iter().map(slot_type).collect::<Result<Vec<_>>>()?;
                eq_kinds(slot_type(lhs)?, Kind::make_tuple(ty))?;
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let ty = slot_type(arg)?;
                let ty = sub_kind(ty, &approximate_dynamic_paths(path))?;
                eq_kinds(ty, slot_type(lhs)?)?;
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
                eq_kinds(ty.clone(), template.kind.clone())?;
                if let Some(rest) = rest {
                    let rest_ty = slot_type(rest)?;
                    eq_kinds(ty.clone(), rest_ty)?;
                }
                for field in fields {
                    match &field.member {
                        rhif::spec::Member::Named(name) => {
                            let path = Path::default().field(name);
                            let ty = sub_kind(ty.clone(), &path)?;
                            eq_kinds(slot_type(&field.value)?, ty)?;
                        }
                        rhif::spec::Member::Unnamed(index) => {
                            let path = Path::default().index(*index as usize);
                            let ty = sub_kind(ty.clone(), &path)?;
                            eq_kinds(slot_type(&field.value)?, ty)?;
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
                let discriminant_value = template.discriminant()?.as_i64()?;
                let variant_kind = ty.lookup_variant(discriminant_value)?;
                for field in fields {
                    match &field.member {
                        rhif::spec::Member::Named(name) => {
                            let ty = sub_kind(variant_kind.clone(), &Path::default().field(name))?;
                            eq_kinds(slot_type(&field.value)?, ty)?;
                        }
                        rhif::spec::Member::Unnamed(index) => {
                            let ty = sub_kind(
                                variant_kind.clone(),
                                &Path::default().index(*index as usize),
                            )?;
                            eq_kinds(slot_type(&field.value)?, ty)?;
                        }
                    }
                }
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                let ty = slot_type(lhs)?;
                let Kind::Array(array_ty) = &ty else {
                    bail!("expected array type")
                };
                eq_kinds(slot_type(value)?, *array_ty.base.clone())?;
                eq_kinds(ty.clone(), Kind::make_array(*array_ty.base.clone(), *len))?;
            }
            OpCode::Comment(_) => {}
            OpCode::Case(Case {
                lhs,
                discriminant: expr,
                table,
            }) => {
                let arg_ty = slot_type(expr)?;
                let mut discriminants: HashSet<Vec<bool>> = Default::default();
                for (entry_test, entry_body) in table {
                    eq_kinds(slot_type(lhs)?, slot_type(entry_body)?)?;
                    match entry_test {
                        CaseArgument::Constant(constant) => {
                            if discriminants.contains(&constant.bits) {
                                bail!("Match contains a duplicate discriminant {constant}, which is not allowed in RHDL");
                            } else {
                                discriminants.insert(constant.bits.clone());
                            }
                            let constant_ty = constant.kind.clone();
                            eq_kinds(arg_ty.clone(), constant_ty)?;
                        }
                        CaseArgument::Wild => {}
                    }
                }
            }
            OpCode::Discriminant(Discriminant { lhs, arg }) => {
                let arg_ty = slot_type(arg)?;
                if arg_ty.is_enum() {
                    eq_kinds(slot_type(lhs)?, arg_ty.get_discriminant_kind()?)?;
                } else {
                    eq_kinds(slot_type(lhs)?, arg_ty)?;
                }
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                // Get the function signature.
                let signature = obj.externals[id.0].signature.clone();
                eq_kinds(slot_type(lhs)?, signature.ret)?;
                for (arg, param) in args.iter().zip(signature.arguments.iter()) {
                    eq_kinds(slot_type(arg)?, param.clone())?;
                }
                ensure!(
                    args.len() == signature.arguments.len(),
                    "wrong number of arguments"
                )
            }
            OpCode::AsBits(Cast { lhs, arg: _, len }) => {
                eq_kinds(slot_type(lhs)?, Kind::make_bits(*len))?;
            }
            OpCode::AsSigned(Cast { lhs, arg: _, len }) => {
                eq_kinds(slot_type(lhs)?, Kind::make_signed(*len))?;
            }
        }
    }
    Ok(())
}
