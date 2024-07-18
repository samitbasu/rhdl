use std::collections::BTreeMap;

use crate::{
    ast::ast_impl::FunctionId,
    rhif::{
        object::SymbolMap,
        spec::{ExternalFunction, FuncId, Slot},
    },
    TypedBits,
};

use super::spec::{LiteralId, OpCode, Operand, RegisterId};

#[derive(Clone)]
pub struct Object {
    pub symbols: SymbolMap,
    pub literals: BTreeMap<LiteralId, TypedBits>,
    pub operand_map: BTreeMap<Operand, Slot>,
    pub return_register: Operand,
    pub externals: BTreeMap<FuncId, ExternalFunction>,
    pub ops: Vec<OpCode>,
    pub arguments: Vec<RegisterId>,
    pub name: String,
    pub fn_id: FunctionId,
}
