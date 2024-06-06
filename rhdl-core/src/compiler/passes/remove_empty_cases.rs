use crate::{
    error::RHDLError,
    rhif::{spec::OpCode, Object},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveEmptyCasesPass {}

impl Pass for RemoveEmptyCasesPass {
    fn name(&self) -> &'static str {
        "remove_empty_cases"
    }
    fn description(&self) -> &'static str {
        "Remove empty cases (ones for which the target register is empty)"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for op in input.ops.iter_mut() {
            if let OpCode::Case(case) = op.clone() {
                if input.kind.get(&case.lhs).unwrap().is_empty() {
                    *op = OpCode::Noop;
                }
            }
        }
        Ok(input)
    }
}
