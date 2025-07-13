use std::collections::HashSet;

use crate::{
    prelude::RHDLError,
    rhdl_core::ntl::{
        spec::{OpCode, Wire, RegisterId},
        visit::{visit_wires, Sense},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct DeadCodeElimination {}

impl Pass for DeadCodeElimination {
    fn description() -> &'static str {
        "Dead Code Elimination"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut needed_set: HashSet<RegisterId> = HashSet::default();
        for lop in &input.ops {
            visit_wires(&lop.op, |sense, op| {
                if sense == Sense::Read {
                    if let Some(reg) = op.reg() {
                        needed_set.insert(reg);
                    }
                }
            });
        }
        needed_set.extend(input.outputs.iter().filter_map(Wire::reg));
        needed_set.extend(input.inputs.iter().flatten().copied());
        input.ops.retain(|lop| {
            let mut output_used = false;
            visit_wires(&lop.op, |sense, op| {
                if sense == Sense::Write {
                    if let Some(reg) = op.reg() {
                        output_used |= needed_set.contains(&reg);
                    }
                }
            });
            output_used || matches!(lop.op, OpCode::Comment(_))
        });
        Ok(input)
    }
}
