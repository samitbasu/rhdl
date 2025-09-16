use std::ops::Range;

use crate::ast::{
    SourceLocation, SourcePool, ast_impl::FunctionId, spanned_source::SpannedSourceSet,
};

#[derive(Debug, Clone, Default, Hash)]
pub struct SymbolMap {
    pub source_set: SpannedSourceSet,
}

impl SymbolMap {
    pub fn source(&self) -> SourcePool {
        self.source_set.source()
    }
    pub fn span<T: Into<SourceLocation>>(&self, loc: T) -> Range<usize> {
        self.source_set.span(loc)
    }
    pub fn fallback(&self, func: FunctionId) -> SourceLocation {
        self.source_set.fallback(func)
    }
}
