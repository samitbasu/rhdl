use crate::rhif::{
    spec::{Assign, OpCode},
    Object,
};
use anyhow::Result;

use super::pass::Pass;

pub struct LowerIndexToCopy {}

impl Pass for LowerIndexToCopy {
    fn name(&self) -> &'static str {
        "lower_index_to_copy"
    }
    fn description(&self) -> &'static str {
        "Lower index operations with empty paths to copy operations"
    }
    fn run(mut input: Object) -> Result<Object> {
        let mut ops = Vec::new();
        for op in input.ops {
            match op {
                OpCode::Index(index) => {
                    if index.path.is_empty() {
                        ops.push(OpCode::Assign(Assign {
                            lhs: index.lhs,
                            rhs: index.arg,
                        }));
                    } else {
                        ops.push(OpCode::Index(index));
                    }
                }
                op => ops.push(op),
            }
        }
        input.ops = ops;
        Ok(input)
    }
}
