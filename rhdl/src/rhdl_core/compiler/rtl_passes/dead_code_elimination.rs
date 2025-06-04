use std::collections::HashSet;

use crate::rhdl_core::{
    rtl::{
        remap::remap_operands,
        spec::{OpCode, Operand},
        Object,
    },
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct DeadCodeEliminationPass {}

impl Pass for DeadCodeEliminationPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // Start with an active set containing only the return operand
        let mut active_set: HashSet<Operand> = HashSet::new();
        active_set.insert(input.return_register);
        // Get the mapping from operands to opcodes
        let mut alive_ops: Vec<bool> = vec![false; input.ops.len()];
        // Iterate through the ops backwards
        for (alive_marker, lop) in alive_ops.iter_mut().rev().zip(input.ops.iter().rev()) {
            let mut mark_active = |lhs: &Operand| {
                if active_set.contains(lhs) {
                    remap_operands(lop.op.clone(), |operand| {
                        active_set.insert(operand);
                        operand
                    });
                    *alive_marker = true;
                }
            };
            match &lop.op {
                OpCode::Assign(assign) => {
                    mark_active(&assign.lhs);
                }
                OpCode::Binary(binary) => {
                    mark_active(&binary.lhs);
                }
                OpCode::Case(case) => {
                    mark_active(&case.lhs);
                }
                OpCode::Cast(cast) => {
                    mark_active(&cast.lhs);
                }
                OpCode::Concat(concat) => {
                    mark_active(&concat.lhs);
                }
                OpCode::DynamicIndex(index) => {
                    mark_active(&index.lhs);
                }
                OpCode::DynamicSplice(splice) => {
                    mark_active(&splice.lhs);
                }
                OpCode::Index(index) => {
                    mark_active(&index.lhs);
                }
                OpCode::Select(select) => {
                    mark_active(&select.lhs);
                }
                OpCode::Splice(splice) => {
                    mark_active(&splice.lhs);
                }
                OpCode::Unary(unary) => {
                    mark_active(&unary.lhs);
                }
                OpCode::Noop => {}
                OpCode::Comment(_) => {
                    *alive_marker = true;
                }
            }
        }
        // Filter out the dead ops
        input.ops = input
            .ops
            .into_iter()
            .zip(alive_ops)
            .filter_map(|(op, alive)| if alive { Some(op) } else { None })
            .collect();
        Ok(input)
    }
    fn description() -> &'static str {
        "Dead code elimination"
    }
}
