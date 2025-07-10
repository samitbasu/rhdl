use std::collections::HashSet;

use crate::rhdl_core::{
    common::symtab::SymbolTable,
    error::RHDLError,
    rhif::{
        Object,
        spec::Slot,
        visit::{visit_object_slots, visit_object_slots_mut},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnusedRegistersPass {}

impl Pass for RemoveUnusedRegistersPass {
    fn description() -> &'static str {
        "Remove unused registers"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Slot> = Default::default();
        visit_object_slots(&input, |_sense, &slot| {
            used_set.insert(slot);
        });
        let (literals, mut registers) = std::mem::take(&mut input.symtab).into_parts();
        let remap = registers.retain(|rid, _| used_set.contains(&Slot::Register(rid)));
        visit_object_slots_mut(&mut input, |_sense, slot| {
            if let Some(rid) = slot.reg() {
                *slot = Slot::Register(remap[&rid]);
            }
        });
        input.symtab = SymbolTable::from_parts(literals, registers);
        Ok(input)
    }
}
