use crate::{
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
    fn name() -> &'static str {
        "lower_inferred_retimes"
    }
    fn description() -> &'static str {
        "Lower inferred retimes to concrete retimes"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for (op, location) in input.ops.iter_mut().zip(input.symbols.opcode_map.iter()) {
            if let OpCode::Retime(retime) = op {
                if retime.color.is_none() {
                    let Some(dest_color) = input.kind[&retime.lhs].signal_clock() else {
                        let op = op.clone();
                        return Err(Self::raise_ice(
                            &input,
                            ICE::UnableToInferClockDomainForRetime { op },
                            location.node,
                        ));
                    };
                    *op = OpCode::Retime(Retime {
                        lhs: retime.lhs,
                        arg: retime.arg,
                        color: Some(dest_color),
                    });
                }
            }
        }
        Ok(input)
    }
}
