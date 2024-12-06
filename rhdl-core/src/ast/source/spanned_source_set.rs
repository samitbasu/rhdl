use std::{collections::BTreeMap, ops::Range};

use crate::ast::ast_impl::FunctionId;

use super::{
    source_location::SourceLocation, source_pool::SourcePool, spanned_source::SpannedSource,
};

#[derive(Clone, Debug, Default, Hash)]
pub struct SpannedSourceSet {
    pub sources: BTreeMap<FunctionId, SpannedSource>,
}

impl SpannedSourceSet {
    pub fn source(&self) -> SourcePool {
        SourcePool::new(&self.sources)
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
        panic!("SourceLocation not found in SpannedSourceSet");
    }
    pub fn fallback(&self, func: FunctionId) -> SourceLocation {
        (func, self.sources[&func].fallback).into()
    }
}

impl Extend<(FunctionId, SpannedSource)> for SpannedSourceSet {
    fn extend<T: IntoIterator<Item = (FunctionId, SpannedSource)>>(&mut self, iter: T) {
        for (id, src) in iter {
            self.sources.insert(id, src);
        }
    }
}

impl From<(FunctionId, SpannedSource)> for SpannedSourceSet {
    fn from((id, src): (FunctionId, SpannedSource)) -> Self {
        let mut set = SpannedSourceSet::default();
        set.sources.insert(id, src);
        set
    }
}
