use crate::{
    error::RHDLError,
    rhif::{
        object::LocatedOpCode,
        remap::rename_read_register,
        spec::{Assign, OpCode},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

fn find_assign_op(ops: &[LocatedOpCode]) -> Option<usize> {
    ops.iter()
        .position(|lop| matches!(lop.op, OpCode::Assign(_)))
}

impl Pass for RemoveExtraRegistersPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        while let Some(op_ndx) = find_assign_op(&input.ops) {
            let lop = input.ops[op_ndx].clone();
            let op = &lop.op;
            if let OpCode::Assign(assign) = op {
                input.ops = input
                    .ops
                    .into_iter()
                    .map(|lop| {
                        let LocatedOpCode { op, id } = lop;
                        let old = assign.lhs;
                        let new = assign.rhs;
                        match op {
                            OpCode::Assign(Assign { lhs, rhs }) => {
                                let new_rhs = if rhs == old { new } else { rhs };
                                if new_rhs == lhs {
                                    LocatedOpCode::new(OpCode::Noop, id)
                                } else {
                                    LocatedOpCode::new(
                                        OpCode::Assign(Assign { lhs, rhs: new_rhs }),
                                        id,
                                    )
                                }
                            }
                            _ => LocatedOpCode::new(rename_read_register(op, old, new), id),
                        }
                    })
                    .map(|lop| {
                        let LocatedOpCode { op, id } = lop;
                        match op {
                            OpCode::Assign(Assign { lhs, rhs: _ }) => {
                                if lhs != assign.lhs {
                                    (op, id).into()
                                } else {
                                    (OpCode::Noop, id).into()
                                }
                            }
                            _ => (op, id).into(),
                        }
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
                eprintln!("Removing register {:?}", assign.lhs);
                if assign.lhs.is_reg() {
                    input.kind.remove(&assign.lhs.as_reg().unwrap());
                }
                // Check the output register
                input.return_slot = input.return_slot.rename(assign.lhs, assign.rhs);
                // Record the alias in the symbol table
                // This is used to find equivalent expressions when emitting error messages
                input.symbols.alias(assign.rhs, assign.lhs);
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
