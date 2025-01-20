use std::collections::HashSet;

use crate::rhdl_core::{
    error::RHDLError,
    rhif::{
        remap::remap_slots,
        spec::{OpCode, Slot},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct DeadCodeEliminationPass {}

impl Pass for DeadCodeEliminationPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // Start with an active set containing only the return slot
        let mut active_set: HashSet<Slot> = HashSet::new();
        active_set.insert(input.return_slot);
        // Get the mapping from slots to opcodes
        let mut alive_ops: Vec<bool> = vec![false; input.ops.len()];
        // Iterate through the ops backwards
        for (alive_marker, lop) in alive_ops.iter_mut().rev().zip(input.ops.iter().rev()) {
            let mut mark_active = |lhs: &Slot| {
                if active_set.contains(lhs) {
                    remap_slots(lop.op.clone(), |slot| {
                        active_set.insert(slot);
                        slot
                    });
                    *alive_marker = true;
                }
            };
            match &lop.op {
                OpCode::Array(array) => {
                    mark_active(&array.lhs);
                }
                OpCode::AsBits(cast) | OpCode::AsSigned(cast) | OpCode::Resize(cast) => {
                    mark_active(&cast.lhs);
                }
                OpCode::Assign(assign) => {
                    mark_active(&assign.lhs);
                }
                OpCode::Binary(binary) => {
                    mark_active(&binary.lhs);
                }
                OpCode::Case(case) => {
                    mark_active(&case.lhs);
                }
                OpCode::Enum(enumerate) => {
                    mark_active(&enumerate.lhs);
                }
                OpCode::Exec(exec) => {
                    mark_active(&exec.lhs);
                }
                OpCode::Index(index) => {
                    mark_active(&index.lhs);
                }
                OpCode::Repeat(repeat) => {
                    mark_active(&repeat.lhs);
                }
                OpCode::Retime(retime) => {
                    mark_active(&retime.lhs);
                }
                OpCode::Select(select) => {
                    mark_active(&select.lhs);
                }
                OpCode::Splice(splice) => {
                    mark_active(&splice.lhs);
                }
                OpCode::Struct(structure) => {
                    mark_active(&structure.lhs);
                }
                OpCode::Tuple(tuple) => {
                    mark_active(&tuple.lhs);
                }
                OpCode::Unary(unary) => {
                    mark_active(&unary.lhs);
                }
                OpCode::Wrap(wrap) => {
                    mark_active(&wrap.lhs);
                }
                OpCode::Noop | OpCode::Comment(_) => {
                    // Noop and Comment ops are always alive
                    *alive_marker = true;
                }
            }
        }
        // Rewrite the dead ops as NOOPs
        input.ops = input
            .ops
            .into_iter()
            .zip(alive_ops)
            .map(|(op, alive)| {
                if alive {
                    op
                } else {
                    (OpCode::Noop, op.loc).into()
                }
            })
            .collect();
        Ok(input)
    }
}
