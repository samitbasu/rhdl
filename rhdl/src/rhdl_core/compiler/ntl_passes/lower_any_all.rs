use crate::{
    prelude::RHDLError,
    rhdl_core::ntl::{
        spec::{assign, OpCode, Operand, Unary, UnaryOp},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerAnyAll {}

fn lower_any_all(op: &OpCode) -> Option<OpCode> {
    let OpCode::Unary(unary) = op else {
        return None;
    };
    match unary.op {
        UnaryOp::All => {
            if unary.arg.iter().any(|op| op.bool() == Some(false)) {
                // Short cut this all instruction - it must be false
                Some(assign(unary.lhs[0], Operand::Zero))
            } else {
                // Filter out any arguments that are boolean true
                let rhs = unary
                    .arg
                    .iter()
                    .filter(|op| op.bool() != Some(true))
                    .copied()
                    .collect::<Vec<_>>();
                if rhs.len() == 1 {
                    Some(assign(unary.lhs[0], rhs[0]))
                } else {
                    Some(OpCode::Unary(Unary {
                        op: UnaryOp::All,
                        lhs: unary.lhs.clone(),
                        arg: rhs,
                    }))
                }
            }
        }
        UnaryOp::Any => {
            if unary.arg.iter().any(|op| op.bool() == Some(true)) {
                // Short cut this any instruction - it must be true
                Some(assign(unary.lhs[0], Operand::One))
            } else {
                // Filter out any arguments that are boolean false
                let rhs = unary
                    .arg
                    .iter()
                    .filter(|op| op.bool() != Some(false))
                    .copied()
                    .collect::<Vec<_>>();
                if rhs.len() == 1 {
                    Some(assign(unary.lhs[0], rhs[0]))
                } else {
                    Some(OpCode::Unary(Unary {
                        op: UnaryOp::Any,
                        lhs: unary.lhs.clone(),
                        arg: rhs,
                    }))
                }
            }
        }
        _ => None,
    }
}

impl Pass for LowerAnyAll {
    fn description() -> &'static str {
        "Lower any and all ops with boolen arguments"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        input.ops.iter_mut().for_each(|lop| {
            if let Some(rep_op) = lower_any_all(&lop.op) {
                lop.op = rep_op;
            }
        });
        Ok(input)
    }
}
