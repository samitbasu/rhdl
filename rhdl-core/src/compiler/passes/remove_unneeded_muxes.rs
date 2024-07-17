use crate::{
    error::RHDLError,
    rhif::{
        spec::{Assign, OpCode},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnneededMuxesPass {}

impl Pass for RemoveUnneededMuxesPass {
    fn name() -> &'static str {
        "remove_unneeded_muxes"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for op in input.ops.iter_mut() {
            if let OpCode::Select(select) = op.clone() {
                if let Ok(literal) = select.cond.as_literal() {
                    let val = &input.literals[&literal];
                    if val.as_bool()? {
                        *op = OpCode::Assign(Assign {
                            lhs: select.lhs,
                            rhs: select.true_value,
                        });
                    } else {
                        *op = OpCode::Assign(Assign {
                            lhs: select.lhs,
                            rhs: select.false_value,
                        });
                    }
                } else if select.true_value == select.false_value {
                    *op = OpCode::Assign(Assign {
                        lhs: select.lhs,
                        rhs: select.true_value,
                    });
                }
            }
        }
        Ok(input)
    }
}
