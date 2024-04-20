use std::collections::HashSet;

use crate::rhif::{spec::Slot, Object};
use anyhow::Result;

use super::pass::Pass;
use crate::compiler::utils::remap_slots;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnusedLiterals {}

impl Pass for RemoveUnusedLiterals {
    fn name(&self) -> &'static str {
        "remove_unused_literals"
    }
    fn description(&self) -> &'static str {
        "Remove unused literals"
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
        input.literals.retain(|slot, _| used_set.contains(slot));
        Ok(input)
    }
}
