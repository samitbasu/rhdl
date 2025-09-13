use std::collections::HashSet;

use crate::{
    error::RHDLError,
    rtl::{
        Object,
        spec::Operand,
        visit::{visit_object_operands, visit_object_operands_mut},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnusedOperandsPass {}

impl Pass for RemoveUnusedOperandsPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut used_set: HashSet<Operand> = Default::default();
        visit_object_operands(&input, |_sense, operand| {
            used_set.insert(*operand);
        });
        let remap = input.symtab.retain(|symb, _| used_set.contains(&symb));
        visit_object_operands_mut(&mut input, |_sense, operand| {
            *operand = remap(*operand).expect("Remapped operand should be present");
        });
        Ok(input)
    }
    fn description() -> &'static str {
        "Remove unused operands from input list"
    }
}
