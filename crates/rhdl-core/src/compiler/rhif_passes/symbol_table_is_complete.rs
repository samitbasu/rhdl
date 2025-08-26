use crate::rhdl_core::{
    compiler::mir::error::ICE,
    error::RHDLError,
    rhif::{Object, visit::visit_object_slots},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct SymbolTableIsComplete {}

impl Pass for SymbolTableIsComplete {
    fn description() -> &'static str {
        "Check that symbol table is complete"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut error = None;
        let id = input.symbols.fallback(input.fn_id);
        visit_object_slots(&input, |sense, &slot| {
            if error.is_none() {
                if !input.symtab.is_key_valid(slot) {
                    error = Some(Err(Self::raise_ice(
                        &input,
                        ICE::SymbolTableIsIncomplete { slot },
                        id,
                    )));
                }
                if sense.is_write() && slot.is_lit() {
                    log::info!("{input:?}");
                    log::error!("Write to literal {slot} detected");
                    panic!("Cannot write to literal slot {slot}!");
                }
            }
        });
        if let Some(err) = error {
            err
        } else {
            Ok(input)
        }
    }
}
