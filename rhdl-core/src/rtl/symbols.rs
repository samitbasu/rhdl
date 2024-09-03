use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{
    ast::{ast_impl::FunctionId, spanned_source::SpannedSource},
    rhif::object::SourceLocation,
};

use super::spec::Operand;

#[derive(Debug, Clone)]
pub struct SymbolMap {
    pub sources: BTreeMap<FunctionId, SpannedSource>,
    pub operand_map: BTreeMap<Operand, SourceLocation>,
    pub operand_names: BTreeMap<Operand, String>,
    pub aliases: BTreeMap<Operand, BTreeSet<Operand>>,
}

