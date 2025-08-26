use crate::rhdl_core::{
    error::RHDLError,
    rhif::{Object, spec::OpCode},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveEmptyCasesPass {}

impl Pass for RemoveEmptyCasesPass {
    fn description() -> &'static str {
        "Remove empty cases and empty selects"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            match &lop.op {
                OpCode::Case(case) => {
                    if input.kind(case.lhs).is_empty() {
                        lop.op = OpCode::Noop;
                    }
                }
                OpCode::Select(select) => {
                    if input.kind(select.lhs).is_empty() {
                        lop.op = OpCode::Noop;
                    }
                }
                _ => {}
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
