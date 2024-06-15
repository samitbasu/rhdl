use crate::{
    error::RHDLError,
    rhif::{spec::OpCode, Object},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveEmptyCasesPass {}

impl Pass for RemoveEmptyCasesPass {
    fn name() -> &'static str {
        "remove_empty_cases"
    }
    fn description() -> &'static str {
        "Remove empty cases (ones for which the target register is empty)"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for op in input.ops.iter_mut() {
            match op {
                OpCode::Case(case) => {
                    if input.kind[&case.lhs].is_empty() {
                        *op = OpCode::Noop;
                    }
                }
                OpCode::Select(select) => {
                    if input.kind[&select.lhs].is_empty() {
                        *op = OpCode::Noop;
                    }
                }
                _ => {}
            }
        }
        Ok(input)
    }
}
