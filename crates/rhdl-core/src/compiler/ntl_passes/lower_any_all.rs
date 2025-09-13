use crate::{
    ntl::{
        Object,
        spec::{OpCode, Unary, UnaryOp, assign},
    },
    {BitX, RHDLError},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerAnyAll {}

fn lower_any_all(input: &Object, op: &OpCode) -> Option<OpCode> {
    let OpCode::Unary(unary) = op else {
        return None;
    };
    match unary.op {
        UnaryOp::All => {
            if let Some(arg) = unary
                .arg
                .iter()
                .find(|op| input.bitx(**op) == Some(BitX::Zero))
            {
                // Short cut this all instruction - it must be false
                Some(assign(unary.lhs[0], *arg))
            } else {
                // Filter out any arguments that are boolean true
                let rhs = unary
                    .arg
                    .iter()
                    .filter(|op| input.bitx(**op) != Some(BitX::One))
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
            if let Some(arg) = unary
                .arg
                .iter()
                .find(|op| input.bitx(**op) == Some(BitX::One))
            {
                // Short cut this any instruction - it must be true
                Some(assign(unary.lhs[0], *arg))
            } else {
                // Filter out any arguments that are boolean false
                let rhs = unary
                    .arg
                    .iter()
                    .filter(|op| input.bitx(**op) != Some(BitX::Zero))
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
        let mut ops = std::mem::take(&mut input.ops);
        ops.iter_mut().for_each(|lop| {
            if let Some(rep_op) = lower_any_all(&input, &lop.op) {
                lop.op = rep_op;
            }
        });
        input.ops = ops;
        Ok(input)
    }
}
