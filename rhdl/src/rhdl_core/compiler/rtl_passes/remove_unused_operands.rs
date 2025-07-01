use std::collections::HashSet;

use crate::rhdl_core::{
    error::RHDLError,
    rtl::{remap::remap_operands, spec::Operand, Object},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnusedOperandsPass {}

impl Pass for RemoveUnusedOperandsPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Operand> = Default::default();
        used_set.extend(
            input
                .arguments
                .iter()
                .flat_map(|r| r.as_ref())
                .map(|r| Operand::Register(*r)),
        );
        used_set.insert(input.return_register);
        for lop in input.ops.iter() {
            remap_operands(lop.op.clone(), |slot| {
                used_set.insert(slot);
                slot
            });
        }
        input
            .register_size
            .retain(|&reg_id, _| used_set.contains(&Operand::Register(reg_id)));
        input
            .literals
            .retain(|&lit_id, _| used_set.contains(&Operand::Literal(lit_id)));
        Ok(input)
    }
    fn description() -> &'static str {
        "Remove unused operands from input list"
    }
}
