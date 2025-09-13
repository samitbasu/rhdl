use crate::{
    RHDLError,
    rtl::{
        Object,
        spec::{Assign, OpCode},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerSingleConcatToCopy {}

impl Pass for LowerSingleConcatToCopy {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            if let OpCode::Concat(concat) = &mut lop.op {
                if concat.args.len() == 1 {
                    lop.op = OpCode::Assign(Assign {
                        lhs: concat.lhs,
                        rhs: concat.args[0],
                    })
                }
            }
        }
        input.ops = ops;
        Ok(input)
    }
    fn description() -> &'static str {
        "Lower single concat to a copy"
    }
}
