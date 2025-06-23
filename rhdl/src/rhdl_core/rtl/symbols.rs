use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

use crate::rhdl_core::{
    ast::{
        ast_impl::FunctionId,
        source::{source_location::SourceLocation, spanned_source_set::SpannedSourceSet},
    },
    SourcePool,
};

use super::spec::Operand;

#[derive(Debug, Clone, Default, Hash)]
pub struct SymbolMap {
    pub source_set: SpannedSourceSet,
    pub operand_map: BTreeMap<Operand, SourceLocation>,
    pub operand_names: BTreeMap<Operand, String>,
    pub aliases: BTreeMap<Operand, BTreeSet<Operand>>,
}

impl SymbolMap {
    pub fn source(&self) -> SourcePool {
        self.source_set.source()
    }
    pub fn span<T: Into<SourceLocation>>(&self, loc: T) -> Range<usize> {
        self.source_set.span(loc)
    }
    pub fn alias(&mut self, op: Operand, alias: Operand) {
        self.aliases.entry(op).or_default().insert(alias);
    }
    pub fn fallback(&self, func: FunctionId) -> SourceLocation {
        self.source_set.fallback(func)
    }
}
