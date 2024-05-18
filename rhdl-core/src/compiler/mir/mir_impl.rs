use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ast::ast_impl::{ExprLit, FunctionId, NodeId},
    rhif::{
        object::SymbolMap,
        spec::{ExternalFunction, OpCode, Slot},
    },
    Kind,
};

#[derive(Clone, Debug, PartialEq)]
pub struct OpCodeWithSource {
    pub op: OpCode,
    pub source: NodeId,
}

impl From<(OpCode, NodeId)> for OpCodeWithSource {
    fn from((op, source): (OpCode, NodeId)) -> Self {
        OpCodeWithSource { op, source }
    }
}
pub struct Mir {
    pub symbols: SymbolMap,
    pub ops: Vec<OpCodeWithSource>,
    pub literals: BTreeMap<Slot, ExprLit>,
    pub ty: BTreeMap<Slot, Kind>,
    pub ty_equate: BTreeSet<(Slot, Slot)>,
    pub stash: Vec<ExternalFunction>,
    pub return_slot: Slot,
    pub arguments: Vec<Slot>,
    pub fn_id: FunctionId,
    pub name: String,
}
