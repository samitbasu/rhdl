use crate::{
    prelude::{BitString, RHDLError},
    rhdl_core::{
        TypedBits,
        ast::source::source_location::SourceLocation,
        ntl::{
            Object,
            object::LocatedOpCode,
            spec::{
                Binary, BinaryOp, Case, CaseEntry, Not, OpCode, Wire, Unary, UnaryOp, Vector,
                VectorOp, assign,
            },
        },
        rtl::{
            runtime_ops::{binary, unary},
            spec::{AluBinary, AluUnary},
        },
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ConstantPropagationPass {}

fn compute_binary(binary: Binary) -> OpCode {
    match (binary.arg1.bitx(), binary.arg2.bitx()) {
        (Some(reg1), Some(reg2)) => {
            let res = match binary.op {
                BinaryOp::And => reg1 & reg2,
                BinaryOp::Or => reg1 | reg2,
                BinaryOp::Xor => reg1 ^ reg2,
            };
            assign(binary.lhs, Wire::from(res))
        }
        _ => OpCode::Binary(binary),
    }
}

fn vec_op(signed: bool, arg: &[Wire]) -> Option<TypedBits> {
    let bits = arg.iter().map(|x| x.bitx()).collect::<Option<Vec<_>>>();
    bits.map(|b| {
        if signed {
            BitString::Signed(b)
        } else {
            BitString::Unsigned(b)
        }
        .into()
    })
}

fn compute_not(not_op: Not) -> OpCode {
    if let Some(val) = not_op.arg.bitx() {
        assign(not_op.lhs, Wire::from(!val))
    } else {
        OpCode::Not(not_op)
    }
}

fn compute_unary(unary_op: Unary, source: Option<SourceLocation>, lop: &mut Vec<LocatedOpCode>) {
    let signed = unary_op.op == UnaryOp::Neg;
    let arg = vec_op(signed, &unary_op.arg);
    let alu = match unary_op.op {
        UnaryOp::All => AluUnary::All,
        UnaryOp::Any => AluUnary::Any,
        UnaryOp::Neg => AluUnary::Neg,
        UnaryOp::Xor => AluUnary::Xor,
    };
    if let Some(arg) = arg {
        if let Ok(val) = unary(alu, arg) {
            for (&lhs, &rhs) in unary_op.lhs.iter().zip(&val.bits) {
                lop.push(LocatedOpCode {
                    op: assign(lhs, Wire::from(rhs)),
                    loc: source,
                })
            }
            return;
        }
    }
    lop.push(LocatedOpCode {
        op: OpCode::Unary(unary_op),
        loc: source,
    })
}

fn compute_vector(vector: Vector, source: Option<SourceLocation>, lop: &mut Vec<LocatedOpCode>) {
    let arg1 = vec_op(vector.signed, &vector.arg1);
    let arg2 = vec_op(vector.signed, &vector.arg2);
    match (arg1, arg2) {
        (Some(arg1), Some(arg2)) => {
            let alu = match vector.op {
                VectorOp::Add => AluBinary::Add,
                VectorOp::Sub => AluBinary::Sub,
                VectorOp::Mul => AluBinary::Mul,
                VectorOp::Eq => AluBinary::Eq,
                VectorOp::Ne => AluBinary::Ne,
                VectorOp::Lt => AluBinary::Lt,
                VectorOp::Le => AluBinary::Le,
                VectorOp::Gt => AluBinary::Gt,
                VectorOp::Ge => AluBinary::Ge,
                VectorOp::Shl => AluBinary::Shl,
                VectorOp::Shr => AluBinary::Shr,
            };
            if let Ok(res) = binary(alu, arg1, arg2) {
                for (&lhs, &rhs) in vector.lhs.iter().zip(&res.bits) {
                    lop.push(LocatedOpCode {
                        op: assign(lhs, Wire::from(rhs)),
                        loc: source,
                    })
                }
            } else {
                lop.push(LocatedOpCode {
                    op: OpCode::Vector(vector),
                    loc: source,
                });
            }
        }
        _ => lop.push(LocatedOpCode {
            op: OpCode::Vector(vector),
            loc: source,
        }),
    }
}

fn compute_case(case: Case) -> OpCode {
    let Some(disc) = vec_op(false, &case.discriminant) else {
        return OpCode::Case(case);
    };
    let Some(entry_ndx) = case.entries.iter().position(|entry| match &entry.0 {
        CaseEntry::Literal(value) => disc == value.into(),
        CaseEntry::WildCard => true,
    }) else {
        return OpCode::Case(case);
    };
    if let Some(_input) = case.entries[entry_ndx].1.reg() {
        return OpCode::Case(case);
    }
    assign(case.lhs, case.entries[entry_ndx].1)
}

impl Pass for ConstantPropagationPass {
    fn description() -> &'static str {
        "Constant propagation"
    }

    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = Vec::with_capacity(input.ops.len());
        let orig = std::mem::take(&mut input.ops);
        for lop in orig {
            match lop.op {
                OpCode::Binary(binary) => ops.push(LocatedOpCode {
                    loc: lop.loc,
                    op: compute_binary(binary),
                }),
                OpCode::Vector(vector) => compute_vector(vector, lop.loc, &mut ops),
                OpCode::Case(case) => ops.push(LocatedOpCode {
                    loc: lop.loc,
                    op: compute_case(case),
                }),
                OpCode::Unary(unary) => compute_unary(unary, lop.loc, &mut ops),
                OpCode::Not(not) => ops.push(LocatedOpCode {
                    loc: lop.loc,
                    op: compute_not(not),
                }),
                _ => ops.push(lop),
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
