use std::collections::HashSet;

use crate::rhdl_core::{
    compiler::mir::error::ICE,
    rtl::{remap::remap_operands, spec::Operand, Object},
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct SymbolTableIsComplete {}

impl Pass for SymbolTableIsComplete {
    fn run(input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Operand> = Default::default();
        used_set.extend(
            input
                .arguments
                .iter()
                .filter_map(|x| x.as_ref())
                .map(|r| Operand::Register(*r)),
        );
        used_set.insert(input.return_register);
        for lop in input.ops.iter() {
            remap_operands(lop.op.clone(), |slot| {
                used_set.insert(slot);
                slot
            });
        }
        let id = input.symbols.fallback(input.fn_id);
        for operand in used_set {
            if !input.symbols.operand_map.contains_key(&operand) {
                return Err(Self::raise_ice(
                    &input,
                    ICE::RTLSymbolTableIsIncomplete { operand },
                    id,
                ));
            }
        }
        Ok(input)
    }
    fn description() -> &'static str {
        "Check that symbol table is complete"
    }
}
