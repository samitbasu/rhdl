use anyhow::Result;

use crate::rhif::{
    spec::{Assign, OpCode, Slot},
    Object,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUselessCastsPass {}

impl Pass for RemoveUselessCastsPass {
    fn name(&self) -> &'static str {
        "remove_useless_casts"
    }
    fn description(&self) -> &'static str {
        "Remove useless casts"
    }
    fn run(mut input: Object) -> Result<Object> {
        for op in input.ops.iter_mut() {
            match op.clone() {
                OpCode::AsBits(cast) => {
                    if let Slot::Literal(ndx) = cast.arg {
                        let literal_val = &input.literals[ndx];
                        if literal_val.kind.is_unsigned() && literal_val.bits.len() == cast.len {
                            *op = OpCode::Assign(Assign {
                                lhs: cast.lhs,
                                rhs: Slot::Literal(ndx),
                            })
                        }
                    }
                }
                OpCode::AsSigned(cast) => {
                    if let Slot::Literal(ndx) = cast.arg {
                        let literal_val = &input.literals[ndx];
                        if literal_val.kind.is_signed() && literal_val.bits.len() == cast.len {
                            *op = OpCode::Assign(Assign {
                                lhs: cast.lhs,
                                rhs: Slot::Literal(ndx),
                            })
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(input)
    }
}
