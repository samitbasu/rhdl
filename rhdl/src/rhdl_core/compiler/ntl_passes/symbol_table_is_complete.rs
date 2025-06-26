use crate::{
    prelude::RHDLError,
    rhdl_core::{compiler::mir::error::ICE, ntl::Object},
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
            let Some(location) = lop.loc else {
                continue;
            };
            let rtl = location.rtl;
            let location = rtl.rhif;
            if !input.code.sources.contains_key(&location.func) {
                return Err(Self::raise_ice(&input, ICE::IncompleteSymbolTable, None));
            }
            if !input.rtl.contains_key(&location.func) {
                eprintln!("input rtl lib is missing {:?}", location.func);
                return Err(Self::raise_ice(&input, ICE::IncompleteSymbolTable, None));
            }
        }
        Ok(input)
    }
}
