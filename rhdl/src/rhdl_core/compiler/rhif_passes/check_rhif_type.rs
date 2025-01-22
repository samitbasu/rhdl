// Check a RHIF object for type correctness.
use log::debug;

use crate::rhdl_core::{
    ast::{ast_impl::WrapOp, source::source_location::SourceLocation},
    compiler::mir::error::ICE,
    error::RHDLError,
    rhif::{
        self,
        spec::{
            AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec,
            Index, OpCode, Repeat, Retime, Select, Slot, Splice, Struct, Tuple, Unary, Wrap,
        },
        Object,
    },
    types::path::{sub_kind, Path, PathElement},
    Kind,
};

use super::pass::Pass;

pub struct TypeCheckPass;

impl Pass for TypeCheckPass {
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

fn xops_kind(obj: &Object, loc: SourceLocation, a: Kind, diff: isize) -> Result<Kind, RHDLError> {
    match a {
        Kind::Bits(a) => Ok(Kind::Bits(((a as isize) + diff) as usize)),
        Kind::Signed(a) => Ok(Kind::Signed(((a as isize) + diff) as usize)),
        _ => Err(TypeCheckPass::raise_ice(
            obj,
            ICE::InvalidPadKind { a },
            loc,
        )),
    }
}

fn xneg_kind(obj: &Object, loc: SourceLocation, a: Kind) -> Result<Kind, RHDLError> {
    match a {
        Kind::Signed(a) | Kind::Bits(a) => Ok(Kind::Signed(a + 1)),
        _ => Err(TypeCheckPass::raise_ice(
            obj,
            ICE::InvalidPadKind { a },
            loc,
        )),
    }
}

fn xsgn_kind(obj: &Object, loc: SourceLocation, a: Kind) -> Result<Kind, RHDLError> {
    match a {
        Kind::Bits(a) => Ok(Kind::Signed(a + 1)),
        _ => Err(TypeCheckPass::raise_ice(
            obj,
            ICE::InvalidPadKind { a },
            loc,
        )),
    }
}

fn xadd_xmul_kind<F: Fn(usize, usize) -> usize>(
    obj: &Object,
    loc: SourceLocation,
    a: Kind,
    b: Kind,
    size_fn: F,
) -> Result<Kind, RHDLError> {
    match (a, b) {
        (Kind::Bits(a), Kind::Bits(b)) => Ok(Kind::Bits(size_fn(a, b))),
        (Kind::Signed(a), Kind::Signed(b)) => Ok(Kind::Signed(size_fn(a, b))),
        _ => Err(TypeCheckPass::raise_ice(
            obj,
            ICE::InvalidXopsKind { a, b },
            loc,
        )),
    }
}

fn xsub_kind(obj: &Object, loc: SourceLocation, a: Kind, b: Kind) -> Result<Kind, RHDLError> {
    let size_fn = |a: usize, b: usize| a.max(b) + 1;
    match (a, b) {
        (Kind::Bits(a), Kind::Bits(b)) | (Kind::Signed(a), Kind::Signed(b)) => {
            Ok(Kind::Signed(size_fn(a, b)))
        }
        _ => Err(TypeCheckPass::raise_ice(
            obj,
            ICE::InvalidXopsKind { a, b },
            loc,
        )),
    }
}

fn check_type_correctness(obj: &Object) -> Result<(), RHDLError> {
    let slot_type = |slot: &Slot| -> Kind {
        if matches!(*slot, Slot::Empty) {
            return Kind::Empty;
        }
        obj.kind(*slot)
    };
    // Checks that two kinds are equal, but ignores clocking information
    let eq_kinds = |a: Kind, b: Kind, loc: SourceLocation| -> Result<(), RHDLError> {
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
                loc,
            ))
        }
    };
    debug!("Checking RHIF type correctness {:?}", obj);
    for lop in &obj.ops {
        let op = &lop.op;
        let loc = lop.loc;
        log::trace!("Checking {:?}", op);
        match op {
            OpCode::Noop => {}
            OpCode::Binary(Binary {
                op:
                    AluBinary::Add
                    | AluBinary::Sub
                    | AluBinary::BitAnd
                    | AluBinary::BitOr
                    | AluBinary::Mul
                    | AluBinary::BitXor,
                lhs,
                arg1,
                arg2,
            }) => {
                eq_kinds(slot_type(lhs), slot_type(arg1), loc)?;
                eq_kinds(slot_type(lhs), slot_type(arg2), loc)?;
            }
            OpCode::Binary(Binary {
                op: AluBinary::XAdd,
                lhs,
                arg1,
                arg2,
            }) => {
                let arg1_ty = slot_type(arg1);
                let arg2_ty = slot_type(arg2);
                let result_ty = xadd_xmul_kind(obj, loc, arg1_ty, arg2_ty, |a, b| a.max(b) + 1)?;
                eq_kinds(slot_type(lhs), result_ty, loc)?;
            }
            OpCode::Binary(Binary {
                op: AluBinary::XMul,
                lhs,
                arg1,
                arg2,
            }) => {
                let arg1_ty = slot_type(arg1);
                let arg2_ty = slot_type(arg2);
                let result_ty = xadd_xmul_kind(obj, loc, arg1_ty, arg2_ty, |a, b| a + b)?;
                eq_kinds(slot_type(lhs), result_ty, loc)?;
            }
            OpCode::Binary(Binary {
                op: AluBinary::XSub,
                lhs,
                arg1,
                arg2,
            }) => {
                let arg1_ty = slot_type(arg1);
                let arg2_ty = slot_type(arg2);
                let result_ty = xsub_kind(obj, loc, arg1_ty, arg2_ty)?;
                eq_kinds(slot_type(lhs), result_ty, loc)?;
            }
            OpCode::Binary(Binary {
                op: AluBinary::Shl | AluBinary::Shr,
                lhs,
                arg1,
                arg2,
            }) => {
                eq_kinds(slot_type(lhs), slot_type(arg1), loc)?;
                if !slot_type(arg2).is_unsigned() {
                    return Err(TypeCheckPass::raise_ice(
                        obj,
                        ICE::ShiftOperatorRequiresUnsignedArgument {
                            kind: slot_type(arg2),
                        },
                        loc,
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
                eq_kinds(slot_type(arg1), slot_type(arg2), loc)?;
                eq_kinds(slot_type(lhs), Kind::make_bool(), loc)?;
            }
            // The unary operators can sneak through to RHIF if the user defines
            // them for their own types.  So we need to check that they are only
            // applied to base types.
            OpCode::Unary(Unary {
                op: AluUnary::Not | AluUnary::Neg | AluUnary::Val,
                lhs,
                arg1,
            }) => {
                eq_kinds(slot_type(lhs), slot_type(arg1), loc)?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::All | AluUnary::Any | AluUnary::Xor,
                lhs,
                arg1: _,
            }) => {
                eq_kinds(slot_type(lhs), Kind::make_bool(), loc)?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::XExt(diff) | AluUnary::XShl(diff),
                lhs,
                arg1,
            }) => {
                eq_kinds(
                    slot_type(lhs),
                    xops_kind(obj, loc, slot_type(arg1), *diff as isize)?,
                    loc,
                )?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::XShr(diff),
                lhs,
                arg1,
            }) => {
                eq_kinds(
                    slot_type(lhs),
                    xops_kind(obj, loc, slot_type(arg1), -(*diff as isize))?,
                    loc,
                )?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::XNeg,
                lhs,
                arg1,
            }) => {
                eq_kinds(slot_type(lhs), xneg_kind(obj, loc, slot_type(arg1))?, loc)?;
            }
            OpCode::Unary(Unary {
                op: AluUnary::XSgn,
                lhs,
                arg1,
            }) => {
                eq_kinds(slot_type(lhs), xsgn_kind(obj, loc, slot_type(arg1))?, loc)?;
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
                        loc,
                    ));
                };
                eq_kinds(slot_type(lhs), Kind::Signed(x), loc)?;
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
                        loc,
                    ));
                };
                eq_kinds(slot_type(lhs), Kind::make_bits(x), loc)?;
            }
            OpCode::Array(Array { lhs, elements }) => eq_kinds(
                slot_type(lhs),
                Kind::make_array(slot_type(&elements[0]), elements.len()),
                loc,
            )?,
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                eq_kinds(slot_type(lhs), slot_type(true_value), loc)?;
                eq_kinds(slot_type(lhs), slot_type(false_value), loc)?;
                eq_kinds(slot_type(cond), Kind::make_bool(), loc)?;
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                eq_kinds(slot_type(lhs), slot_type(rhs), loc)?;
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
                    loc,
                )?;
                eq_kinds(slot_type(lhs), slot_type(orig), loc)?;
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let ty = fields.iter().map(slot_type).collect::<Vec<_>>();
                eq_kinds(slot_type(lhs), Kind::make_tuple(ty), loc)?;
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let ty = slot_type(arg).signal_data();
                let ty = sub_kind(ty, &approximate_dynamic_paths(path))?;
                eq_kinds(ty, slot_type(lhs), loc)?;
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
                eq_kinds(ty, template.kind, loc)?;
                if let Some(rest) = rest {
                    let rest_ty = slot_type(rest);
                    eq_kinds(ty, rest_ty, loc)?;
                }
                for field in fields {
                    match &field.member {
                        rhif::spec::Member::Named(name) => {
                            let path = Path::default().field(name);
                            let ty = sub_kind(ty, &path)?;
                            eq_kinds(slot_type(&field.value), ty, loc)?;
                        }
                        rhif::spec::Member::Unnamed(index) => {
                            let path = Path::default().index(*index as usize);
                            let ty = sub_kind(ty, &path)?;
                            eq_kinds(slot_type(&field.value), ty, loc)?;
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
                            ty,
                        },
                        obj.symbols.slot_map[lhs],
                    ))?
                    .kind;
                for field in fields {
                    match &field.member {
                        rhif::spec::Member::Named(name) => {
                            let ty = sub_kind(variant_kind, &Path::default().field(name))?;
                            eq_kinds(slot_type(&field.value), ty, loc)?;
                        }
                        rhif::spec::Member::Unnamed(index) => {
                            let ty = sub_kind(
                                variant_kind,
                                &Path::default().tuple_index(*index as usize),
                            )?;
                            eq_kinds(slot_type(&field.value), ty, loc)?;
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
                eq_kinds(ty, Kind::make_array(*array_ty.base.clone(), *len as _), loc)?;
            }
            OpCode::Comment(_) => {}
            OpCode::Case(Case {
                lhs,
                discriminant: expr,
                table,
            }) => {
                let arg_ty = slot_type(expr);
                for (entry_test, entry_body) in table {
                    eq_kinds(slot_type(lhs), slot_type(entry_body), loc)?;
                    match entry_test {
                        CaseArgument::Slot(slot) => {
                            if !matches!(slot, Slot::Literal(_)) {
                                return Err(TypeCheckPass::raise_ice(
                                    obj,
                                    ICE::MatchPatternValueMustBeLiteral,
                                    obj.symbols.slot_map[slot],
                                ));
                            }
                            eq_kinds(arg_ty, slot_type(slot), loc)?;
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
                eq_kinds(slot_type(lhs), sub.kind(sub.return_slot), loc)?;
                for (arg, param) in args.iter().zip(sub_args) {
                    eq_kinds(slot_type(arg), param, loc)?;
                }
                if args.len() != sub.arguments.len() {
                    return Err(TypeCheckPass::raise_ice(
                        obj,
                        ICE::ArgumentCountMismatchOnCall,
                        loc,
                    ));
                }
            }
            OpCode::AsBits(Cast { lhs, arg: _, len }) => {
                let len = len.ok_or(TypeCheckPass::raise_ice(
                    obj,
                    ICE::BitCastMissingRequiredLength,
                    loc,
                ))?;
                eq_kinds(slot_type(lhs), Kind::make_bits(len), loc)?;
            }
            OpCode::AsSigned(Cast { lhs, arg: _, len }) => {
                let len = len.ok_or(TypeCheckPass::raise_ice(
                    obj,
                    ICE::BitCastMissingRequiredLength,
                    loc,
                ))?;
                eq_kinds(slot_type(lhs), Kind::make_signed(len), loc)?;
            }
            OpCode::Resize(Cast { lhs, arg, len }) => {
                let len = len.ok_or(TypeCheckPass::raise_ice(
                    obj,
                    ICE::BitCastMissingRequiredLength,
                    loc,
                ))?;
                if slot_type(arg).is_signed() {
                    eq_kinds(slot_type(lhs), Kind::make_signed(len), loc)?;
                } else {
                    eq_kinds(slot_type(lhs), Kind::make_bits(len), loc)?;
                }
            }
            OpCode::Retime(Retime { lhs, arg, color: _ }) => {
                eq_kinds(slot_type(lhs), slot_type(arg), loc)?;
            }
            OpCode::Wrap(Wrap { op, lhs, arg, kind }) => {
                let Some(wrap_kind) = kind else {
                    return Err(TypeCheckPass::raise_ice(obj, ICE::WrapMissingKind, loc));
                };
                match op {
                    WrapOp::Err | WrapOp::Ok => {
                        if !wrap_kind.is_result() {
                            return Err(TypeCheckPass::raise_ice(
                                obj,
                                ICE::WrapRequiresResultKind { kind: *wrap_kind },
                                loc,
                            ));
                        }
                    }
                    WrapOp::Some | WrapOp::None => {
                        if !wrap_kind.is_option() {
                            return Err(TypeCheckPass::raise_ice(
                                obj,
                                ICE::WrapRequiresOptionKind { kind: *wrap_kind },
                                loc,
                            ));
                        }
                    }
                }
                eq_kinds(slot_type(lhs), *wrap_kind, loc)?;
                let payload_path = match op {
                    WrapOp::Ok => Path::default().payload("Ok").tuple_index(0),
                    WrapOp::Err => Path::default().payload("Err").tuple_index(0),
                    WrapOp::Some => Path::default().payload("Some").tuple_index(0),
                    WrapOp::None => Path::default().payload("None"),
                };
                let payload_ty = sub_kind(*wrap_kind, &payload_path)?;
                eq_kinds(slot_type(arg), payload_ty, loc)?;
            }
        }
    }
    Ok(())
}
