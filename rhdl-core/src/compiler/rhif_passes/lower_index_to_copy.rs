use crate::{
    error::RHDLError,
    rhif::{
        spec::{Assign, OpCode},
        Object,
    },
};

use super::pass::Pass;

pub struct LowerIndexToCopy {}

impl Pass for LowerIndexToCopy {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = Vec::new();
        for lop in input.ops {
            match lop.op {
                OpCode::Index(index) => {
                    if index.path.is_empty() {
                        ops.push(
                            (
                                OpCode::Assign(Assign {
                                    lhs: index.lhs,
                                    rhs: index.arg,
                                }),
                                lop.id,
                            )
                                .into(),
                        );
                    } else {
                        ops.push((OpCode::Index(index), lop.id).into());
                    }
                }
                _ => ops.push(lop),
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
