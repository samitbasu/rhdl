use crate::rhdl_core::{
    compiler::mir::error::ICE,
    error::RHDLError,
    rhif::{
        spec::{OpCode, Retime},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerInferredRetimesPass {}

impl Pass for LowerInferredRetimesPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = input.ops.clone();
        for lop in ops.iter_mut() {
            if let OpCode::Retime(retime) = &lop.op {
                if retime.color.is_none() {
                    let Some(dest_color) = input.kind(retime.lhs).signal_clock() else {
                        let op = lop.op.clone();
                        return Err(Self::raise_ice(
                            &input,
                            ICE::UnableToInferClockDomainForRetime { op },
                            lop.loc,
                        ));
                    };
                    lop.op = OpCode::Retime(Retime {
                        lhs: retime.lhs,
                        arg: retime.arg,
                        color: Some(dest_color),
                    });
                }
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
