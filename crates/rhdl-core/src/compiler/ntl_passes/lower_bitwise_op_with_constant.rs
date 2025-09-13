use crate::{
    ntl::{
        Object,
        spec::{Binary, BinaryOp, Not, OpCode, Wire, assign},
    },
    {BitX, RHDLError},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerBitwiseOpWithConstant {}

struct LowerArgs {
    constant_arg_value: bool,
    reg_arg: Wire,
    constant_arg: Wire,
}

fn classify_binary_args(input: &Object, binary: &Binary) -> Option<LowerArgs> {
    let as_bool = |arg| input.bitx(arg).and_then(BitX::to_bool);
    match (as_bool(binary.arg1), as_bool(binary.arg2)) {
        (None, Some(constant_arg)) => Some(LowerArgs {
            constant_arg_value: constant_arg,
            reg_arg: binary.arg1,
            constant_arg: binary.arg2,
        }),
        (Some(constant_arg), None) => Some(LowerArgs {
            constant_arg_value: constant_arg,
            reg_arg: binary.arg2,
            constant_arg: binary.arg1,
        }),
        _ => None,
    }
}

fn lower_binary_op(input: &Object, op: &OpCode) -> Option<OpCode> {
    let OpCode::Binary(binary) = op else {
        return None;
    };
    let lower_args = classify_binary_args(input, binary)?;
    match binary.op {
        BinaryOp::And => {
            if lower_args.constant_arg_value {
                // x AND 1 -> x
                Some(assign(binary.lhs, lower_args.reg_arg))
            } else {
                // x AND 0 -> 0
                Some(assign(binary.lhs, lower_args.constant_arg))
            }
        }
        BinaryOp::Or => {
            if lower_args.constant_arg_value {
                // x OR 1 -> 1
                Some(assign(binary.lhs, lower_args.constant_arg))
            } else {
                // x OR 0 -> x
                Some(assign(binary.lhs, lower_args.reg_arg))
            }
        }
        BinaryOp::Xor => {
            if lower_args.constant_arg_value {
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
        let mut ops = std::mem::take(&mut input.ops);
        ops.iter_mut().for_each(|lop| {
            if let Some(rep_op) = lower_binary_op(&input, &lop.op) {
                lop.op = rep_op;
            }
        });
        input.ops = ops;
        Ok(input)
    }
}
