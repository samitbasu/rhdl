use crate::rhdl_core::{
    rtl::{
        spec::{Assign, OpCode},
        Object,
    },
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerEmptySpliceToCopy {}

impl Pass for LowerEmptySpliceToCopy {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        for lop in ops.iter_mut() {
            match &mut lop.op {
                OpCode::Splice(splice) if splice.bit_range.is_empty() => {
                    lop.op = OpCode::Assign(Assign {
                        lhs: splice.lhs,
                        rhs: splice.orig,
                    })
                }
                OpCode::DynamicSplice(splice) if splice.len == 0 => {
                    lop.op = OpCode::Assign(Assign {
                        lhs: splice.lhs,
                        rhs: splice.arg,
                    })
                }
                _ => {}
            }
        }
        input.ops = ops;
        Ok(input)
    }
    fn description() -> &'static str {
        "Lower empty splice to copy"
    }
}
