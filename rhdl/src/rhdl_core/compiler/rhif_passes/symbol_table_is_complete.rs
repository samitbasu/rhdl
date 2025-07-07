use std::collections::HashSet;

use crate::rhdl_core::{
    compiler::mir::error::ICE,
    error::RHDLError,
    rhif::{Object, remap::remap_slots, spec::Slot},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct SymbolTableIsComplete {}

impl Pass for SymbolTableIsComplete {
    fn description() -> &'static str {
        "Check that symbol table is complete"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Slot> = Default::default();
        used_set.extend(input.arguments.iter().map(|r| Slot::Register(*r)));
        used_set.insert(input.return_slot);
        for lop in input.ops.iter() {
            remap_slots(lop.op.clone(), |slot| {
                used_set.insert(slot);
                slot
            });
        }
        let id = input.symbols.fallback(input.fn_id);
        for slot in used_set {
            if !input.symtab.is_ke_valid(&slot) {
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
