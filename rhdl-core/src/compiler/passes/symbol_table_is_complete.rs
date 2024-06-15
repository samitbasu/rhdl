use std::{collections::HashSet, hash::Hash};

use crate::{
    compiler::{mir::error::ICE, utils::remap_slots},
    error::RHDLError,
    rhif::{spec::Slot, Object},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct SymbolTableIsComplete {}

impl Pass for SymbolTableIsComplete {
    fn name() -> &'static str {
        "symbol_table_is_complete"
    }
    fn description() -> &'static str {
        "Check if the symbol table is complete"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Slot> = Default::default();
        used_set.extend(input.arguments.iter());
        used_set.insert(input.return_slot);
        for op in input.ops.iter() {
            remap_slots(op.clone(), |slot| {
                used_set.insert(slot);
                slot
            });
        }
        let id = input.symbols.source.fallback;
        for slot in used_set {
            if !input.symbols.slot_map.contains_key(&slot) {
                return Err(Self::raise_ice(
                    &input,
                    ICE::SymbolTableIsIncomplete { slot },
                    id,
                ));
            }
        }
        Ok(input)
    }
}
