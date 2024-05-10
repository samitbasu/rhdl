use anyhow::Result;

use crate::{
    path::Path,
    rhif::{
        spec::{Assign, OpCode},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveDiscriminantsForNonEnumTypesPass {}

impl Pass for RemoveDiscriminantsForNonEnumTypesPass {
    fn name(&self) -> &'static str {
        "remove_discriminants_for_non_enum_types"
    }
    fn description(&self) -> &'static str {
        "Remove discriminants for non-enum types"
    }
    fn run(mut input: Object) -> Result<Object> {
        for op in input.ops.iter_mut() {
            if let OpCode::Index(index) = op {
                if index.path == Path::default().discriminant() && !input.kind[&index.arg].is_enum()
                {
                    *op = OpCode::Assign(Assign {
                        lhs: index.lhs,
                        rhs: index.arg,
                    })
                }
            }
        }
        Ok(input)
    }
}
