use crate::rhdl_core::{
    rtl::{spec::OpCode, Object},
    RHDLError,
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
                    .filter(|arg| !input.kind(*arg).is_empty())
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
