use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::Range,
};

use crate::{
    ast::{ast_impl::FunctionId, spanned_source::SpannedSource},
    rhif::object::SourceLocation,
};

use super::spec::Operand;

#[derive(Debug, Clone, Default, Hash)]
pub struct SymbolMap {
    pub sources: BTreeMap<FunctionId, SpannedSource>,
    pub operand_map: BTreeMap<Operand, SourceLocation>,
    pub operand_names: BTreeMap<Operand, String>,
    pub aliases: BTreeMap<Operand, BTreeSet<Operand>>,
}

impl SymbolMap {
    pub fn source(&self) -> String {
        self.sources
            .values()
            .fold(String::new(), |acc, src| acc + &src.source)
    }
    pub fn span(&self, loc: SourceLocation) -> Range<usize> {
        let mut offset = 0;
        for (id, src) in &self.sources {
            if *id == loc.func {
                let span = src.span(loc.node);
                return (span.start + offset)..(span.end + offset);
            }
            offset += src.source.len();
        }
        panic!("SourceLocation not found in SymbolMap");
    }
    pub fn alias(&mut self, op: Operand, alias: Operand) {
        self.aliases.entry(op).or_default().insert(alias);
    }
    pub fn fallback(&self, func: FunctionId) -> SourceLocation {
        (func, self.sources[&func].fallback).into()
    }
}
