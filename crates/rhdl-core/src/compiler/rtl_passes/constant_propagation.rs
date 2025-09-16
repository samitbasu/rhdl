use crate::{
    RHDLError, TypedBits,
    ast::SourceLocation,
    common::symtab::LiteralId,
    compiler::mir::error::{ICE, RHDLCompileError},
    error::rhdl_error,
    rtl::{
        Object,
        object::LocatedOpCode,
        spec::{
            Assign, Binary, Case, CaseArgument, Cast, CastKind, Concat, Index, OpCode, Operand,
            Select, Splice, Unary,
        },
    },
};

use super::pass::Pass;

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
        let arg1_val = obj.symtab[&arg1].clone();
        let arg2_val = obj.symtab[&arg2].clone();
        let result: TypedBits = crate::rtl::runtime_ops::binary(op, arg1_val, arg2_val)?;
        let details = obj.symtab[&lhs].clone();
        let result = obj.symtab.lit(result, details);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign { lhs, rhs: result }),
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
        let arg_val = obj.symtab[&arg1].clone();
        let result = crate::rtl::runtime_ops::unary(op, arg_val)?;
        let details = obj.symtab[&lhs].clone();
        let result = obj.symtab.lit(result, details);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign { lhs, rhs: result }),
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
        .collect::<Option<Vec<LiteralId<_>>>>();
    if let Some(literals) = all_literals {
        let result: TypedBits = literals.iter().map(|lit| obj.symtab[lit].clone()).collect();
        let details = obj.symtab[&concat.lhs].clone();
        let result = obj.symtab.lit(result, details);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: concat.lhs,
                rhs: result,
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
        let discriminant_val = &obj.symtab[disc];
        let rhs = table
            .iter()
            .find(|(case_arg, _)| match case_arg {
                CaseArgument::Literal(lit) => &obj.symtab[lit] == discriminant_val,
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
        let arg = obj.symtab[arg].clone();
        let result = match kind {
            CastKind::Signed => arg.signed_cast(*len),
            CastKind::Unsigned => arg.unsigned_cast(*len),
            CastKind::Resize => arg.resize(*len),
        }?;
        let details = obj.symtab[lhs].clone();
        let result = obj.symtab.lit(result, details);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: result,
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
        let cond = &obj.symtab[cond];
        let tb = cond.bits[0].to_bool().ok_or_else(|| {
            rhdl_error(RHDLCompileError {
                cause: ICE::SelectOnUninitializedValue {
                    value: cond.clone(),
                },
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
        path: _,
    } = &splice;
    if let (Operand::Literal(orig), Operand::Literal(value)) = (orig, value) {
        let orig = obj.symtab[orig].clone();
        let value = obj.symtab[value].clone();
        let mut bits = orig.bits.to_vec();
        bits.splice(bit_range.clone(), value.bits.iter().copied());
        let details = obj.symtab[lhs].clone();
        let kind = obj.kind(*lhs);
        let result = TypedBits { kind, bits };
        let result = obj.symtab.lit(result, details);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: result,
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

fn propagate_index(
    loc: SourceLocation,
    index: Index,
    obj: &mut Object,
) -> Result<LocatedOpCode, RHDLError> {
    let Index {
        lhs,
        arg,
        bit_range,
        path: _,
    } = &index;
    if let Operand::Literal(arg) = arg {
        let arg = obj.symtab[arg].clone();
        let bits = arg.bits[bit_range.clone()].to_vec();
        let details = obj.symtab[lhs].clone();
        let kind = obj.kind(*lhs);
        let result = TypedBits { kind, bits };
        let result = obj.symtab.lit(result, details);
        Ok(LocatedOpCode {
            op: OpCode::Assign(Assign {
                lhs: *lhs,
                rhs: result,
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
                OpCode::Select(inner) => propagate_select(lop.loc, inner, &mut input),
                OpCode::Splice(inner) => propagate_splice(lop.loc, inner, &mut input),
                OpCode::Index(inner) => propagate_index(lop.loc, inner, &mut input),
                OpCode::Assign(_) | OpCode::Comment(_) | OpCode::Noop => Ok(lop),
            })
            .collect::<Result<Vec<_>, RHDLError>>()?;
        Ok(input)
    }
    fn description() -> &'static str {
        "Constant propagation"
    }
}
