use crate::{
    prelude::RHDLError,
    rhdl_core::{
        compiler::mir::error::ICE,
        ntl::{Object, visit::visit_operands},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct SymbolTableIsComplete {}

impl Pass for SymbolTableIsComplete {
    fn description() -> &'static str {
        "Check that symbol table is complete"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        for lop in &input.ops {
            let mut err = None;
            visit_operands(&lop.op, |_sense, operand| {
                if err.is_none() && !input.kinds.contains_key(operand) {
                    err = Some(Err(Self::raise_ice(
                        &input,
                        ICE::IncompleteSymbolTableInNetList,
                        lop.loc,
                    )))
                }
            });
            if let Some(err) = err {
                return err;
            }
            let Some(location) = lop.loc else {
                continue;
            };
            if !input.code.sources.contains_key(&location.func) {
                return Err(Self::raise_ice(
                    &input,
                    ICE::IncompleteSymbolTableInNetList,
                    None,
                ));
            }
        }
        Ok(input)
    }
}
