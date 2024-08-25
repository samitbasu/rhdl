// Check a RHIF object for type correctness.

use crate::{
    ast::ast_impl::NodeId,
    compiler::mir::error::ICE,
    error::RHDLError,
    rhif::{
        self,
        spec::{
            AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec,
            Index, OpCode, Repeat, Retime, Select, Slot, Splice, Struct, Tuple, Unary,
        },
        Object,
    },
    types::path::{sub_kind, Path, PathElement},
    Kind,
};

use super::pass::Pass;

pub struct TypeCheckPass;

impl Pass for TypeCheckPass {
    fn name() -> &'static str {
        "check_rhif_type"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
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

fn check_type_correctness(obj: &Object) -> Result<(), RHDLError> {
    let slot_type = |slot: &Slot| -> Kind {
        if matches!(*slot, Slot::Empty) {
            return Kind::Empty;
        }
        obj.kind(*slot)
    };
    // Checks that two kinds are equal, but ignores clocking information
    let eq_kinds = |a: Kind, b: Kind, node: NodeId| -> Result<(), RHDLError> {
        // Special case Empty == Tuple([])
        if a.is_empty() && b.is_empty() {
            return Ok(());
        }
        let a = a.signal_data();
        let b = b.signal_data();
        if a == b {
            Ok(())
        } else {
            Err(TypeCheckPass::raise_ice(
                obj,
                ICE::MismatchInDataTypes { lhs: a, rhs: b },
                node,
            ))
        }
    };
    for lop in &obj.ops {
        let op = &lop.op;
        let id = lop.id;
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
                eq_kinds(slot_type(lhs), slot_type(arg1), id)?;
                eq_kinds(slot_type(lhs), slot_type(arg2), id)?;
            }
            OpCode::Binary(Binary {
                op: AluBinary::Shl | AluBinary::Shr,
                lhs,
                arg1,
                arg2,
            }) => {
                eq_kinds(slot_type(lhs), slot_type(arg1), id)?;
                if !slot_type(arg2).is_unsigned() {
                    return Err(TypeCheckPass::raise_ice(
                        obj,
                        ICE::ShiftOperatorRequiresUnsignedArgument,
                        id,
                    ));
                }
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
                eq_kinds(slot_type(arg1), slot_type(arg2), id)?;
                eq_kinds(slot_type(lhs), Kind::make_bool(), id)?;
            }
            // The unary operators can sneak through to RHIF if the user defines
            // them for their own types.  So we need to check that they are only
            // applied to base types.
            OpCode::Unary(Unary {
                op: AluUnary::Not | AluUnary::Neg | AluUnary::Val,
                lhs,
                arg1,
            }) => {
                eq_kinds(slot_type(lhs), slot_type(arg1), id)?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::All | AluUnary::Any | AluUnary::Xor,
                lhs,
                arg1: _,
            }) => {
                eq_kinds(slot_type(lhs), Kind::make_bool(), id)?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::Signed,
                lhs,
                arg1,
            }) => {
                let arg1_ty = slot_type(arg1);
                let Kind::Bits(x) = arg1_ty else {
                    return Err(TypeCheckPass::raise_ice(
                        obj,
                        ICE::SignedCastRequiresUnsignedArgument,
                        id,
                    ));
                };
                eq_kinds(slot_type(lhs), Kind::Signed(x), id)?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::Unsigned,
                lhs,
                arg1,
            }) => {
                let arg1_ty = slot_type(arg1);
                let Kind::Signed(x) = arg1_ty else {
                    return Err(TypeCheckPass::raise_ice(
                        obj,
                        ICE::UnsignedCastRequiresSignedArgument,
                        id,
                    ));
                };
                eq_kinds(slot_type(lhs), Kind::make_bits(x), id)?;
            }
            OpCode::Array(Array { lhs, elements }) => eq_kinds(
                slot_type(lhs),
                Kind::make_array(slot_type(&elements[0]), elements.len()),
                id,
            )?,
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                eq_kinds(slot_type(lhs), slot_type(true_value), id)?;
                eq_kinds(slot_type(lhs), slot_type(false_value), id)?;
                eq_kinds(slot_type(cond), Kind::make_bool(), id)?;
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                eq_kinds(slot_type(lhs), slot_type(rhs), id)?;
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                path,
                subst,
            }) => {
                eq_kinds(
                    sub_kind(slot_type(lhs), &approximate_dynamic_paths(path))?,
                    slot_type(subst),
                    id,
                )?;
                eq_kinds(slot_type(lhs), slot_type(orig), id)?;
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let ty = fields.iter().map(slot_type).collect::<Vec<_>>();
                eq_kinds(slot_type(lhs), Kind::make_tuple(ty), id)?;
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let ty = slot_type(arg).signal_data();
                let ty = sub_kind(ty, &approximate_dynamic_paths(path))?;
                eq_kinds(ty, slot_type(lhs), id)?;
                for slot in path.dynamic_slots() {
                    if !slot_type(slot).signal_data().is_unsigned() {
                        return Err(TypeCheckPass::raise_ice(
                            obj,
                            ICE::IndexValueMustBeUnsigned,
                            obj.symbols.slot_map[slot],
                        ));
                    }
                }
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                let ty = slot_type(lhs);
                eq_kinds(ty.clone(), template.kind.clone(), id)?;
                if let Some(rest) = rest {
                    let rest_ty = slot_type(rest);
                    eq_kinds(ty.clone(), rest_ty, id)?;
                }
                for field in fields {
                    match &field.member {
                        rhif::spec::Member::Named(name) => {
                            let path = Path::default().field(name);
                            let ty = sub_kind(ty.clone(), &path)?;
                            eq_kinds(slot_type(&field.value), ty, id)?;
                        }
                        rhif::spec::Member::Unnamed(index) => {
                            let path = Path::default().index(*index as usize);
                            let ty = sub_kind(ty.clone(), &path)?;
                            eq_kinds(slot_type(&field.value), ty, id)?;
                        }
                    }
                }
            }
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => {
                let ty = slot_type(lhs);
                let discriminant_value = template.discriminant()?.as_i64()?;
                let variant_kind = ty
                    .lookup_variant(discriminant_value)
                    .ok_or(TypeCheckPass::raise_ice(
                        obj,
                        ICE::VariantNotFoundInType {
                            variant: discriminant_value,
                            ty: ty.clone(),
                        },
                        obj.symbols.slot_map[lhs],
                    ))?
                    .kind
                    .clone();
                for field in fields {
                    match &field.member {
                        rhif::spec::Member::Named(name) => {
                            let ty = sub_kind(variant_kind.clone(), &Path::default().field(name))?;
                            eq_kinds(slot_type(&field.value), ty, id)?;
                        }
                        rhif::spec::Member::Unnamed(index) => {
                            let ty = sub_kind(
                                variant_kind.clone(),
                                &Path::default().tuple_index(*index as usize),
                            )?;
                            eq_kinds(slot_type(&field.value), ty, id)?;
                        }
                    }
                }
            }
            OpCode::Repeat(Repeat { lhs, value: _, len }) => {
                let ty = slot_type(lhs);
                let Kind::Array(array_ty) = &ty else {
                    return Err(TypeCheckPass::raise_ice(
                        obj,
                        ICE::ExpectedArrayType { kind: ty },
                        obj.symbols.slot_map[lhs],
                    ));
                };
                eq_kinds(
                    ty.clone(),
                    Kind::make_array(*array_ty.base.clone(), *len as _),
                    id,
                )?;
            }
            OpCode::Comment(_) => {}
            OpCode::Case(Case {
                lhs,
                discriminant: expr,
                table,
            }) => {
                let arg_ty = slot_type(expr);
                for (entry_test, entry_body) in table {
                    eq_kinds(slot_type(lhs), slot_type(entry_body), id)?;
                    match entry_test {
                        CaseArgument::Slot(slot) => {
                            if !matches!(slot, Slot::Literal(_)) {
                                return Err(TypeCheckPass::raise_ice(
                                    obj,
                                    ICE::MatchPatternValueMustBeLiteral,
                                    obj.symbols.slot_map[slot],
                                ));
                            }
                            eq_kinds(arg_ty.clone(), slot_type(slot), id)?;
                        }
                        CaseArgument::Wild => {}
                    }
                }
            }
            OpCode::Exec(Exec {
                lhs,
                id: func_id,
                args,
            }) => {
                // Get the function signature.
                let sub = &obj.externals[func_id];
                let sub_args = sub.arguments.iter().map(|x| sub.kind(Slot::Register(*x)));
                eq_kinds(slot_type(lhs), sub.kind(sub.return_slot), id)?;
                for (arg, param) in args.iter().zip(sub_args) {
                    eq_kinds(slot_type(arg), param.clone(), id)?;
                }
                if args.len() != sub.arguments.len() {
                    return Err(TypeCheckPass::raise_ice(
                        obj,
                        ICE::ArgumentCountMismatchOnCall,
                        id,
                    ));
                }
            }
            OpCode::AsBits(Cast { lhs, arg: _, len }) => {
                let len = len.ok_or(TypeCheckPass::raise_ice(
                    obj,
                    ICE::BitCastMissingRequiredLength,
                    id,
                ))?;
                eq_kinds(slot_type(lhs), Kind::make_bits(len), id)?;
            }
            OpCode::AsSigned(Cast { lhs, arg: _, len }) => {
                let len = len.ok_or(TypeCheckPass::raise_ice(
                    obj,
                    ICE::BitCastMissingRequiredLength,
                    id,
                ))?;
                eq_kinds(slot_type(lhs), Kind::make_signed(len), id)?;
            }
            OpCode::Retime(Retime { lhs, arg, color: _ }) => {
                eq_kinds(slot_type(lhs), slot_type(arg), id)?;
            }
        }
    }
    Ok(())
}
