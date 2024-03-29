use crate::{
    compiler::utils::rename_read_register,
    rhif::{
        spec::{Assign, OpCode},
        Object,
    },
};
use anyhow::Result;

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

fn find_assign_op(ops: &[OpCode]) -> Option<OpCode> {
    ops.iter()
        .find(|op| matches!(op, OpCode::Assign(_)))
        .cloned()
}

impl Pass for RemoveExtraRegistersPass {
    fn name(&self) -> &'static str {
        "remove_extra_registers"
    }
    fn description(&self) -> &'static str {
        "Remove extra registers (any instance of r3 <- r2, is replaced with renaming all instances of r3 to r2)"
    }
    fn run(mut input: Object) -> Result<Object> {
        while let Some(op) = find_assign_op(&input.ops) {
            eprintln!("Found assign op {}", op);
            if let OpCode::Assign(assign) = op {
                input.ops = input
                    .ops
                    .into_iter()
                    .map(|op| {
                        let old = assign.lhs;
                        let new = assign.rhs;
                        match op {
                            OpCode::Assign(Assign { lhs, rhs }) => {
                                let new_rhs = if rhs == old { new } else { rhs };
                                if new_rhs == lhs {
                                    OpCode::Noop
                                } else {
                                    OpCode::Assign(Assign { lhs, rhs: new_rhs })
                                }
                            }
                            _ => rename_read_register(op, old, new),
                        }
                    })
                    .map(|op| match op {
                        OpCode::Assign(Assign { lhs, rhs: _ }) => {
                            if lhs != assign.lhs {
                                op
                            } else {
                                OpCode::Noop
                            }
                        }
                        _ => op,
                    })
                    .collect();
                // Delete the register from the register map
                input.symbols.slot_map.remove(&assign.lhs);
                input.kind.remove(&assign.lhs);
                // Check the output register
                input.return_slot = input.return_slot.rename(assign.lhs, assign.rhs);
            }
        }
        Ok(input)
    }
}
