use crate::{
    RHDLError,
    compiler::mir::error::ICE,
    rtl::{Object, visit::visit_object_operands},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct SymbolTableIsComplete {}

impl Pass for SymbolTableIsComplete {
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut error = None;
        let id = input.symbols.fallback(input.fn_id);
        visit_object_operands(&input, |sense, &operand| {
            if error.is_none() && !input.symtab.is_key_valid(operand) {
                error = Some(Err(Self::raise_ice(
                    &input,
                    ICE::SymbolTableIsIncompleteForRTL { operand },
                    id,
                )));
            }
            if sense.is_write() && operand.is_lit() {
                log::info!("{input:?}");
                log::error!("Write to literal {operand} detected");
                panic!("Cannot write to literal operand {operand}");
            }
        });
        if let Some(err) = error {
            err
        } else {
            Ok(input)
        }
    }
    fn description() -> &'static str {
        "Check that symbol table is complete"
    }
}
