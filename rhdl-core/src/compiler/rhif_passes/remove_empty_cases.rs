use crate::{
    error::RHDLError,
    rhif::{spec::OpCode, Object},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveEmptyCasesPass {}

impl Pass for RemoveEmptyCasesPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for lop in input.ops.iter_mut() {
            match &lop.op {
                OpCode::Case(case) => {
                    if case.lhs.is_empty() {
                        lop.op = OpCode::Noop;
                    }
                }
                OpCode::Select(select) => {
                    if select.lhs.is_empty() {
                        lop.op = OpCode::Noop;
                    }
                }
                _ => {}
            }
        }
        Ok(input)
    }
}
