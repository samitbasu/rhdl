use std::collections::HashSet;

use crate::rhdl_core::{
    error::RHDLError,
    rhif::{remap::remap_slots, spec::Slot, Object},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnusedRegistersPass {}

impl Pass for RemoveUnusedRegistersPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Slot> = Default::default();
        used_set.extend(input.arguments.iter().map(|r| Slot::Register(*r)));
        used_set.insert(input.return_slot);
        for lop in input.ops.iter() {
            remap_slots(lop.op.clone(), |slot| {
                used_set.insert(slot);
                slot
            });
        }
        input
            .kind
            .retain(|&reg_id, _| used_set.contains(&reg_id.into()));
        Ok(input)
    }
}