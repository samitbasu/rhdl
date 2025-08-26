use crate::rhdl_core::{
    RHDLError,
    rtl::{Object, spec::OpCode},
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct StripEmptyArgsFromConcat {}

impl Pass for StripEmptyArgsFromConcat {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            if let OpCode::Concat(concat) = &mut lop.op {
                let args = concat
                    .args
                    .iter()
                    .copied()
                    .filter(|arg| input.kind(*arg).bits() != 0)
                    .collect();
                concat.args = args;
            }
        }
        input.ops = ops;
        Ok(input)
    }
    fn description() -> &'static str {
        "Strip empty args from concats"
    }
}
