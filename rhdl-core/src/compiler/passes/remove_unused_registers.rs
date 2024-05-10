use std::collections::HashSet;

use crate::{
    compiler::utils::remap_slots,
    rhif::{
        spec::{OpCode, Slot},
        Object,
    },
};
use anyhow::Result;

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnusedRegistersPass {}

impl Pass for RemoveUnusedRegistersPass {
    fn name(&self) -> &'static str {
        "remove_unused_registers"
    }
    fn description(&self) -> &'static str {
        "Remove unused registers"
    }
    fn run(mut input: Object) -> Result<Object> {
        let mut used_set: HashSet<Slot> = Default::default();
        used_set.extend(input.arguments.iter());
        used_set.insert(input.return_slot);
        for op in input.ops.iter() {
            remap_slots(op.clone(), |slot| {
                used_set.insert(slot);
                slot
            });
        }
        input.kind.retain(|slot, _| used_set.contains(slot));
        Ok(input)
    }
}
