use crate::{
    compiler::utils::rename_read_register,
    error::RHDLError,
    rhif::{
        spec::{Assign, OpCode},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

fn find_assign_op(ops: &[OpCode], mergeable: &[bool]) -> Option<usize> {
    ops.iter()
        .zip(mergeable.iter())
        .position(|(op, flag)| *flag && matches!(op, OpCode::Assign(_)))
}

impl Pass for RemoveExtraRegistersPass {
    fn name() -> &'static str {
        "remove_extra_registers"
    }
    fn description() -> &'static str {
        "Remove extra registers (any instance of r3 <- r2, is replaced with renaming all instances of r3 to r2)"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut eligible = vec![true; input.ops.len()];
        while let Some(op_ndx) = find_assign_op(&input.ops, &eligible) {
            let op = input.ops[op_ndx].clone();
            eprintln!("Found assign op {:?}", op);
            if let OpCode::Assign(assign) = op {
                let lhs_name = input.symbols.slot_names.get(&assign.lhs);
                let rhs_name = input.symbols.slot_names.get(&assign.rhs);
                if !can_merge_names(lhs_name, rhs_name) {
                    eprintln!(
                        "Cannot merge names {:?} {:?} for registers {:?} {:?}",
                        lhs_name, rhs_name, assign.lhs, assign.rhs
                    );
                    eligible[op_ndx] = false;
                    continue;
                }
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
                // Merge the names of the registers
                let lhs_name = input.symbols.slot_names.get(&assign.lhs);
                let rhs_name = input.symbols.slot_names.get(&assign.rhs);
                if let Some(merged_name) = merge_names(lhs_name, rhs_name) {
                    input.symbols.slot_names.insert(assign.rhs, merged_name);
                }
                // Delete the register from the register map
                //input.symbols.slot_map.remove(&assign.lhs);
                input.kind.remove(&assign.lhs);
                // Check the output register
                input.return_slot = input.return_slot.rename(assign.lhs, assign.rhs);
                // Record the alias in the symbol table
                // This is used to find equivalent expressions when emitting error messages
                input.symbols.aliases.insert(assign.rhs, assign.lhs);
            }
        }
        Ok(input)
    }
}

fn merge_names(a: Option<&String>, b: Option<&String>) -> Option<String> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (Some(a), Some(b)) if a == b => Some(a.clone()),
        (Some(a), Some(b)) => Some(format!("{}_then_{}", b, a)),
    }
}

fn can_merge_names(a: Option<&String>, b: Option<&String>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(_), None) => true,
        (None, Some(_)) => true,
        (Some(a), Some(b)) => a == b,
    }
}
