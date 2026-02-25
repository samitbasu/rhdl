use crate::rhif::{Object, spec::OpCode};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerBinaryCaseToSelectPass {}

impl Pass for LowerBinaryCaseToSelectPass {
    fn description() -> &'static str {
        "Lower binary case statements to select statements"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for lop in input.ops.iter_mut() {
            if let OpCode::Select(select) = &lop.op
                && input.kind(select.cond).bits() == 1
            {}
        }
        Ok(input)
    }
}
