use crate::{
    ast::source_location::SourceLocation,
    compiler::mir::error::{RHDLCompileError, ICE},
    error::rhdl_error,
    rtl::{
        object::{LocatedOpCode, RegisterKind},
        spec::{
            Assign, Binary, Case, CaseArgument, Cast, CastKind, Concat, DynamicIndex,
            DynamicSplice, Index, LiteralId, OpCode, Operand, Select, Splice, Unary,
        },
        Object,
    },
    types::bit_string::BitString,
    RHDLError, TypedBits,
};

use super::pass::Pass;

fn assign_literal(loc: SourceLocation, value: BitString, obj: &mut Object) -> Operand {
    let literal = obj.literal_max_index().next();
    obj.literals.insert(literal, value);
    obj.symbols
        .operand_map
        .insert(Operand::Literal(literal), loc);
    Operand::Literal(literal)
}

fn propagate_binary(
    loc: SourceLocation,
    params: Binary,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Binary {
        lhs,
        op,
        arg1,
        arg2,
    } = params;
    if let (Operand::Literal(arg1), Operand::Literal(arg2)) = (arg1, arg2) {
        let arg1_val: TypedBits = obj.literals[&arg1].clone().into();
        let arg2_val: TypedBits = obj.literals[&arg2].clone().into();
        let result: BitString = crate::rhif::runtime_ops::binary(op, arg1_val, arg2_val)?.into();
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, result, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Binary(Binary {
                lhs,
                op,
                arg1,
                arg2,
            }),
            loc,
        })
    }
}

fn propagate_unary(
    loc: SourceLocation,
    params: Unary,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Unary { lhs, op, arg1 } = params;
    if let Operand::Literal(arg1) = arg1 {
        let arg_val: TypedBits = obj.literals[&arg1].clone().into();
        let result: BitString = crate::rhif::runtime_ops::unary(op, arg_val)?.into();
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs,
                rhs: assign_literal(loc, result, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Unary(Unary { lhs, op, arg1 }),
            loc,
        })
    }
}

fn propagate_concat(
    loc: SourceLocation,
    concat: Concat,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let all_literals = concat
        .args
        .iter()
        .map(|operand| match operand {
            Operand::Literal(lit) => Some(*lit),
            _ => None,
        })
        .collect::<Option<Vec<LiteralId>>>();
    if let Some(literals) = all_literals {
        let bits = literals
            .iter()
            .flat_map(|lit| obj.literals[lit].bits())
            .copied()
            .collect::<Vec<_>>();
        let arg = match obj.kind(concat.lhs) {
            RegisterKind::Signed(_) => BitString::Signed(bits),
            RegisterKind::Unsigned(_) => BitString::Unsigned(bits),
        };
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: concat.lhs,
                rhs: assign_literal(loc, arg, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Concat(concat),
            loc,
        })
    }
}

fn propagate_case(
    loc: SourceLocation,
    case: Case,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Case {
        lhs,
        discriminant,
        table,
    } = &case;
    if let Operand::Literal(disc) = discriminant {
        let discriminant_val = obj.literals[disc].clone();
        let rhs = table
            .iter()
            .find(|(case_arg, _)| match case_arg {
                CaseArgument::Literal(lit) => obj.literals[lit] == discriminant_val,
                CaseArgument::Wild => true,
            })
            .unwrap()
            .1;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign { lhs: *lhs, rhs }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Case(case),
            loc,
        })
    }
}

fn propagate_cast(
    loc: SourceLocation,
    cast: crate::rtl::spec::Cast,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Cast {
        lhs,
        arg,
        len,
        kind,
    } = &cast;
    if let Operand::Literal(arg) = arg {
        let arg = obj.literals[arg].clone();
        let result = match kind {
            CastKind::Signed => arg.signed_cast(*len),
            CastKind::Unsigned => arg.unsigned_cast(*len),
            CastKind::Resize => arg.resize(*len),
        }?;
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, result, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Cast(cast),
            loc,
        })
    }
}

fn propagate_dynamic_index(
    loc: SourceLocation,
    dynamic_index: crate::rtl::spec::DynamicIndex,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let DynamicIndex {
        lhs,
        arg,
        offset,
        len,
    } = &dynamic_index;
    if let (Operand::Literal(arg), Operand::Literal(offset)) = (arg, offset) {
        let arg = obj.literals[arg].clone();
        let offset = obj.literals[offset].clone();
        let offset: TypedBits = offset.into();
        let offset = offset.as_i64()? as usize;
        let slice = arg.bits()[offset..(offset + *len)].to_vec();
        let result = match obj.kind(*lhs) {
            RegisterKind::Signed(_) => BitString::Signed(slice),
            RegisterKind::Unsigned(_) => BitString::Unsigned(slice),
        };
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, result, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::DynamicIndex(dynamic_index),
            loc,
        })
    }
}

fn propagate_select(
    loc: SourceLocation,
    select: Select,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Select {
        lhs,
        cond,
        true_value,
        false_value,
    } = &select;
    if let Operand::Literal(cond) = cond {
        let cond = obj.literals[cond].clone();
        let tb = cond.bits()[0].to_bool().ok_or_else(|| {
            rhdl_error(RHDLCompileError {
                cause: ICE::SelectOnUninitializedValue { value: cond },
                src: obj.symbols.source(),
                err_span: obj.symbols.span(loc).into(),
            })
        })?;
        let rhs = if tb { *true_value } else { *false_value };
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign { lhs: *lhs, rhs }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Select(select),
            loc,
        })
    }
}

fn propagate_splice(
    loc: SourceLocation,
    splice: Splice,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Splice {
        lhs,
        orig,
        bit_range,
        value,
    } = &splice;
    if let (Operand::Literal(orig), Operand::Literal(value)) = (orig, value) {
        let orig = obj.literals[orig].clone();
        let value = obj.literals[value].clone();
        let mut bits = orig.bits().to_vec();
        bits.splice(bit_range.clone(), value.bits().iter().copied());
        let result = match obj.kind(*lhs) {
            RegisterKind::Signed(_) => BitString::Signed(bits),
            RegisterKind::Unsigned(_) => BitString::Unsigned(bits),
        };
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, result, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Splice(splice),
            loc,
        })
    }
}

fn propagate_dynamic_splice(
    loc: SourceLocation,
    dynamic_splice: DynamicSplice,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let DynamicSplice {
        lhs,
        arg,
        offset,
        len,
        value,
    } = &dynamic_splice;
    if let (Operand::Literal(arg), Operand::Literal(offset), Operand::Literal(value)) =
        (arg, offset, value)
    {
        let arg = obj.literals[arg].clone();
        let offset = obj.literals[offset].clone();
        let value = obj.literals[value].clone();
        let offset: TypedBits = offset.into();
        let offset = offset.as_i64()? as usize;
        let value = value.bits().to_vec();
        let mut bits = arg.bits().to_vec();
        bits.splice(offset..(offset + *len), value);
        let result = match obj.kind(*lhs) {
            RegisterKind::Signed(_) => BitString::Signed(bits),
            RegisterKind::Unsigned(_) => BitString::Unsigned(bits),
        };
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, result, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::DynamicSplice(dynamic_splice),
            loc,
        })
    }
}

fn propagate_index(
    loc: SourceLocation,
    index: Index,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Index {
        lhs,
        arg,
        bit_range,
    } = &index;
    if let Operand::Literal(arg) = arg {
        let arg = obj.literals[arg].clone();
        let slice = arg.bits()[bit_range.clone()].to_vec();
        let result = match obj.kind(*lhs) {
            RegisterKind::Signed(_) => BitString::Signed(slice),
            RegisterKind::Unsigned(_) => BitString::Unsigned(slice),
        };
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: assign_literal(loc, result, obj),
            }),
            loc,
        })
    } else {
        Ok(LocatedOpCode {
            op: OpCode::Index(index),
            loc,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct ConstantPropagationPass {}

impl Pass for ConstantPropagationPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = std::mem::take(&mut input.ops);
        input.ops = ops
            .into_iter()
            .map(|lop| match lop.op {
                OpCode::Binary(inner) => propagate_binary(lop.loc, inner, &mut input),
                OpCode::Unary(inner) => propagate_unary(lop.loc, inner, &mut input),
                OpCode::Concat(inner) => propagate_concat(lop.loc, inner, &mut input),
                OpCode::Case(inner) => propagate_case(lop.loc, inner, &mut input),
                OpCode::Cast(inner) => propagate_cast(lop.loc, inner, &mut input),
                OpCode::DynamicIndex(inner) => propagate_dynamic_index(lop.loc, inner, &mut input),
                OpCode::Select(inner) => propagate_select(lop.loc, inner, &mut input),
                OpCode::Splice(inner) => propagate_splice(lop.loc, inner, &mut input),
                OpCode::DynamicSplice(inner) => {
                    propagate_dynamic_splice(lop.loc, inner, &mut input)
                }
                OpCode::Index(inner) => propagate_index(lop.loc, inner, &mut input),
                OpCode::Assign(_) | OpCode::Comment(_) | OpCode::Noop => Ok(lop),
            })
            .collect::<Result<Vec<_>, RHDLError>>()?;
        Ok(input)
    }
}
