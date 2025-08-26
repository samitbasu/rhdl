use crate::rhdl_core::{
    RHDLError,
    rtl::{
        Object,
        spec::{Assign, OpCode},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerIndexAllToCopy {}

impl Pass for LowerIndexAllToCopy {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            if let OpCode::Index(index) = &mut lop.op {
                let arg_len = input.kind(index.arg).bits();
                if index.bit_range == (0..arg_len) {
                    lop.op = OpCode::Assign(Assign {
                        lhs: index.lhs,
                        rhs: index.arg,
                    })
                }
            }
        }
        input.ops = ops;
        Ok(input)
    }
    fn description() -> &'static str {
        "Lower index all to a copy"
    }
}
