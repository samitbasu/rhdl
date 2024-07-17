use crate::{
    error::RHDLError,
    rhif::{
        spec::{Assign, OpCode},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUselessCastsPass {}

impl Pass for RemoveUselessCastsPass {
    fn name() -> &'static str {
        "remove_useless_casts"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for op in input.ops.iter_mut() {
            match op.clone() {
                OpCode::AsBits(cast) => {
                    if let Ok(literal) = cast.arg.as_literal() {
                        let literal_val = &input.literals[&literal];
                        if let Some(len) = cast.len {
                            if literal_val.kind.is_unsigned() && literal_val.bits.len() == len {
                                *op = OpCode::Assign(Assign {
                                    lhs: cast.lhs,
                                    rhs: cast.arg,
                                })
                            }
                        }
                    }
                }
                OpCode::AsSigned(cast) => {
                    if let Ok(literal) = cast.arg.as_literal() {
                        let literal_val = &input.literals[&literal];
                        if let Some(len) = cast.len {
                            if literal_val.kind.is_signed() && literal_val.bits.len() == len {
                                *op = OpCode::Assign(Assign {
                                    lhs: cast.lhs,
                                    rhs: cast.arg,
                                })
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(input)
    }
}
