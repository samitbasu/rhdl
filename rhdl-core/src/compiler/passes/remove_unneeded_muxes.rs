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
    fn name(&self) -> &'static str {
        "remove_unneeded_muxes"
    }
    fn description(&self) -> &'static str {
        "Remove unneeded muxes (ones for which the two options are the same or ones with hardwired selectors)"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for op in input.ops.iter_mut() {
            if let OpCode::Select(select) = op.clone() {
                if let Some(val) = input.literals.get(&select.cond) {
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
