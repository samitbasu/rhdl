use std::collections::HashSet;

use crate::{
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
pub struct RemoveUnusedLiterals {}

impl Pass for RemoveUnusedLiterals {
    fn description() -> &'static str {
        "Remove unused literals"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Slot> = Default::default();
        visit_object_slots(&input, |sense, slot| {
            if sense.is_read() {
                used_set.insert(*slot);
            }
        });
        let (mut literals, registers) = std::mem::take(&mut input.symtab).into_parts();
        let remap = literals.retain(|lid, _| used_set.contains(&Slot::Literal(lid)));
        visit_object_slots_mut(&mut input, |sense, slot| {
            if sense.is_read() {
                if let Some(lid) = slot.lit() {
                    *slot = Slot::Literal(
                        remap(lid).expect("New symtab should include all used literals"),
                    );
                }
            }
        });
        input.symtab = SymbolTable::from_parts(literals, registers);
        log::debug!("{input:?}");
        Ok(input)
    }
}
