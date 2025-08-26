use crate::rhdl_core::{
    error::RHDLError,
    rhif::{
        Object,
        spec::{Assign, OpCode},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUselessCastsPass {}

impl Pass for RemoveUselessCastsPass {
    fn description() -> &'static str {
        "Remove useless casts"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for lop in input.ops.iter_mut() {
            match lop.op.clone() {
                OpCode::AsBits(cast) => {
                    if let Some(literal) = cast.arg.lit() {
                        let literal_val = &input.symtab[literal];
                        if let Some(len) = cast.len {
                            if literal_val.kind.is_unsigned() && literal_val.bits.len() == len {
                                lop.op = OpCode::Assign(Assign {
                                    lhs: cast.lhs,
                                    rhs: cast.arg,
                                })
                            }
                        }
                    }
                }
                OpCode::AsSigned(cast) => {
                    if let Some(literal) = cast.arg.lit() {
                        let literal_val = &input.symtab[literal];
                        if let Some(len) = cast.len {
                            if literal_val.kind.is_signed() && literal_val.bits.len() == len {
                                lop.op = OpCode::Assign(Assign {
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
