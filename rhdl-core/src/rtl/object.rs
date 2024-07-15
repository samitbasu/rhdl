use std::collections::BTreeMap;

use crate::{
    ast::ast_impl::FunctionId,
    rhif::{
        object::SymbolMap,
        spec::{ExternalFunction, FuncId, Slot},
    },
    TypedBits,
};

use super::spec::OpCode;

#[derive(Clone)]
pub struct Object {
    pub symbols: SymbolMap,
    pub literals: BTreeMap<Operand, TypedBits>,
    pub operand_map: BTreeMap<Operand, Slot>,
    pub return_register: Operand,
    pub externals: BTreeMap<FuncId, ExternalFunction>,
    pub ops: Vec<OpCode>,
    pub arguments: Vec<Operand>,
    pub name: String,
    pub fn_id: FunctionId,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Operand {
    Literal(usize),
    Register(usize),
}
