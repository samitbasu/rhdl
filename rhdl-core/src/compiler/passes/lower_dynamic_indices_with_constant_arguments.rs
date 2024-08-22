use crate::{
    compiler::mir::error::RHDLTypeError,
    error::RHDLError,
    rhif::{
        object::LocatedOpCode,
        spec::{OpCode, Slot},
        Object,
    },
    types::path::{Path, PathElement},
};

use super::pass::Pass;

pub struct LowerDynamicIndicesWithConstantArguments {}

fn simplify_path(path: Path, obj: &Object) -> Path {
    path.into_iter()
        .map(|x| {
            if let PathElement::DynamicIndex(Slot::Literal(x)) = x {
                let literal_value = obj.literals[&x].as_i64().unwrap();
                let literal_value: usize = literal_value.try_into().unwrap();
                PathElement::Index(literal_value)
            } else {
                x
            }
        })
        .collect()
}

impl Pass for LowerDynamicIndicesWithConstantArguments {
    fn name() -> &'static str {
        "lower_dynamic_indices_with_constant_arguments"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = input
            .ops
            .clone()
            .into_iter()
            .map(|lop| {
                if let OpCode::Index(mut index) = lop.op {
                    index.path = simplify_path(index.path, &input);
                    LocatedOpCode {
                        op: OpCode::Index(index),
                        id: lop.id,
                    }
                } else {
                    lop
                }
            })
            .collect();
        input.ops = ops;
        Ok(input)
    }
}
