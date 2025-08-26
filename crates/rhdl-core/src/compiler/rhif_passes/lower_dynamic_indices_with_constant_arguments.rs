use crate::rhdl_core::{
    error::RHDLError,
    rhif::{
        Object,
        object::LocatedOpCode,
        spec::{OpCode, Slot},
    },
    types::path::{Path, PathElement},
};

use super::pass::Pass;

pub struct LowerDynamicIndicesWithConstantArguments {}

fn simplify_path(path: Path, obj: &Object) -> Path {
    path.elements()
        .map(|x| {
            if let PathElement::DynamicIndex(Slot::Literal(x)) = x {
                let literal_value = obj.symtab[x].as_i64().unwrap();
                let literal_value: usize = literal_value.try_into().unwrap();
                PathElement::Index(literal_value)
            } else {
                x
            }
        })
        .collect()
}

impl Pass for LowerDynamicIndicesWithConstantArguments {
    fn description() -> &'static str {
        "Lower dynamic index ops with constant arguments"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let ops = input
            .ops
            .clone()
            .into_iter()
            .map(|lop| match lop.op {
                OpCode::Index(mut index) => {
                    index.path = simplify_path(index.path, &input);
                    LocatedOpCode {
                        op: OpCode::Index(index),
                        loc: lop.loc,
                    }
                }
                OpCode::Splice(mut splice) => {
                    splice.path = simplify_path(splice.path, &input);
                    LocatedOpCode {
                        op: OpCode::Splice(splice),
                        loc: lop.loc,
                    }
                }
                _ => lop,
            })
            .collect();
        input.ops = ops;
        Ok(input)
    }
}
