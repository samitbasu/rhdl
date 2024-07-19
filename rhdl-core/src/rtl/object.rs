use std::collections::BTreeMap;

use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    rhif::{
        object::SymbolMap,
        spec::{ExternalFunction, FuncId, Slot},
    },
    TypedBits,
};

use super::spec::{LiteralId, OpCode, Operand, RegisterId};

#[derive(Clone, Debug)]
pub struct LocatedOpCode {
    pub op: OpCode,
    pub id: NodeId,
}

impl LocatedOpCode {
    pub fn new(op: OpCode, id: NodeId) -> Self {
        Self { op, id }
    }
}

impl From<(OpCode, NodeId)> for LocatedOpCode {
    fn from((op, id): (OpCode, NodeId)) -> Self {
        Self::new(op, id)
    }
}

#[derive(Clone)]
pub struct Object {
    pub symbols: SymbolMap,
    pub literals: BTreeMap<LiteralId, TypedBits>,
    pub operand_map: BTreeMap<Operand, Slot>,
    pub return_register: Option<Operand>,
    pub externals: BTreeMap<FuncId, ExternalFunction>,
    pub ops: Vec<LocatedOpCode>,
    pub arguments: Vec<Option<RegisterId>>,
    pub name: String,
    pub fn_id: FunctionId,
}
