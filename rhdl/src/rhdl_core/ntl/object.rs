use crate::{
    prelude::HDLDescriptor,
    rhdl_core::{
        ast::{
            ast_impl::{FunctionId, NodeId},
            source::{source_location::SourceLocation, spanned_source_set::SpannedSourceSet},
        },
        ntl::spec::{OpCode, Operand, RegisterId},
    },
};

#[derive(Clone, Hash, PartialEq, Copy, Debug)]
pub enum BlackBoxMode {
    Synchronous,
    Asynchronous,
}

#[derive(Clone, Hash)]
pub struct BlackBox {
    code: HDLDescriptor,
    mode: BlackBoxMode,
}

#[derive(Clone, Default)]
pub struct Object {
    pub name: String,
    pub inputs: Vec<Vec<RegisterId>>,
    pub outputs: Vec<Operand>,
    pub ops: Vec<LocatedOpCode>,
    pub code: SpannedSourceSet,
    pub black_boxes: Vec<BlackBox>,
}

#[derive(Clone, Hash)]
pub struct LocatedOpCode {
    pub op: OpCode,
    pub loc: SourceLocation,
}

impl LocatedOpCode {
    pub fn new(op: OpCode, id: NodeId, func: FunctionId) -> Self {
        Self {
            op,
            loc: SourceLocation { node: id, func },
        }
    }
}
