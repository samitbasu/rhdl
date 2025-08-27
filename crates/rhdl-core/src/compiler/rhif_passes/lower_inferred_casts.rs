use crate::{
    error::RHDLError,
    rhif::{
        spec::{Cast, OpCode},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerInferredCastsPass {}

impl Pass for LowerInferredCastsPass {
    fn description() -> &'static str {
        "Lower inferred casts to concrete casts"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = input.ops.clone();
        for lop in ops.iter_mut() {
            match lop.op.clone() {
                OpCode::AsBits(cast) => {
                    if cast.len.is_none() {
                        let dest_width = input.kind(cast.lhs).bits();
                        lop.op = OpCode::AsBits(Cast {
                            lhs: cast.lhs,
                            arg: cast.arg,
                            len: Some(dest_width),
                        })
                    }
                }
                OpCode::AsSigned(cast) => {
                    if cast.len.is_none() {
                        let dest_width = input.kind(cast.lhs).bits();
                        lop.op = OpCode::AsSigned(Cast {
                            lhs: cast.lhs,
                            arg: cast.arg,
                            len: Some(dest_width),
                        })
                    }
                }
                OpCode::Resize(cast) => {
                    if cast.len.is_none() {
                        let dest_width = input.kind(cast.lhs).bits();
                        lop.op = OpCode::Resize(Cast {
                            lhs: cast.lhs,
                            arg: cast.arg,
                            len: Some(dest_width),
                        })
                    }
                }
                _ => {}
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
