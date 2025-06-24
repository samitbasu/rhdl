use crate::{
    prelude::RHDLError,
    rhdl_core::ntl::{
        spec::{assign, Binary, BinaryOp, Not, OpCode, Operand},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerBitwiseOpWithConstant {}

struct LowerArgs {
    constant_arg: bool,
    reg_arg: Operand,
}

fn classify_binary_args(binary: &Binary) -> Option<LowerArgs> {
    match (binary.arg1.bool(), binary.arg2.bool()) {
        (None, Some(constant_arg)) => Some(LowerArgs {
            constant_arg,
            reg_arg: binary.arg1,
        }),
        (Some(constant_arg), None) => Some(LowerArgs {
            constant_arg,
            reg_arg: binary.arg2,
        }),
        _ => None,
    }
}

fn lower_binary_op(op: &OpCode) -> Option<OpCode> {
    let OpCode::Binary(binary) = op else {
        return None;
    };
    let lower_args = classify_binary_args(binary)?;
    match binary.op {
        BinaryOp::And => {
            if lower_args.constant_arg {
                // x AND 1 -> x
                Some(assign(binary.lhs, lower_args.reg_arg))
            } else {
                // x AND 0 -> 0
                Some(assign(binary.lhs, Operand::Zero))
            }
        }
        BinaryOp::Or => {
            if lower_args.constant_arg {
                // x OR 1 -> 1
                Some(assign(binary.lhs, Operand::One))
            } else {
                // x OR 0 -> x
                Some(assign(binary.lhs, lower_args.reg_arg))
            }
        }
        BinaryOp::Xor => {
            if lower_args.constant_arg {
                // x XOR 1 -> !x
                Some(OpCode::Not(Not {
                    lhs: binary.lhs,
                    arg: lower_args.reg_arg,
                }))
            } else {
                // x XOR 0 -> x
                Some(assign(binary.lhs, lower_args.reg_arg))
            }
        }
    }
}

impl Pass for LowerBitwiseOpWithConstant {
    fn description() -> &'static str {
        "Lower bitwise ops with constant args (e.g., And/Or with 1/0, and Xor with 0/1)"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        input.ops.iter_mut().for_each(|lop| {
            if let Some(rep_op) = lower_binary_op(&lop.op) {
                lop.op = rep_op;
            }
        });
        Ok(input)
    }
}
