use crate::{
    compiler::mir::error::ICE,
    rtl::{spec::OpCode, Object},
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct CheckNoZeroResize {}

impl Pass for CheckNoZeroResize {
    fn run(input: Object) -> Result<Object, RHDLError> {
        for op in &input.ops {
            if let OpCode::Cast(cast) = &op.op {
                if cast.len == 0 {
                    return Err(Self::raise_ice(
                        &input,
                        ICE::InvalidResize {
                            lhs: cast.lhs,
                            arg: cast.arg,
                            len: cast.len,
                        },
                        op.loc,
                    ));
                }
            }
        }
        Ok(input)
    }
}
