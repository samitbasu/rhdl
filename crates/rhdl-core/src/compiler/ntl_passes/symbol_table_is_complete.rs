use crate::{
    RHDLError,
    {
        compiler::mir::error::ICE,
        ntl::{Object, visit::visit_wires},
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
            visit_wires(&lop.op, |_sense, &wire| {
                if err.is_none() && !input.symtab.is_key_valid(wire) {
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
        }
        Ok(input)
    }
}
