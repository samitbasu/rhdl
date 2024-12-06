use crate::ast::ast_impl::{FunctionId, NodeId};
use std::hash::Hash;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hasher,
    ops::Range,
};

use super::source_pool::SourcePool;

#[derive(Clone, Debug)]
pub struct SpannedSource {
    pub source: String,
    pub name: String,
    pub span_map: HashMap<NodeId, Range<usize>>,
    pub fallback: NodeId,
    pub filename: String,
    pub function_id: FunctionId,
}

impl SpannedSource {
    pub fn source(&self) -> SourcePool {
        let mut map = BTreeMap::new();
        map.insert(self.function_id, self.clone());
        SourcePool::new(&map)
    }
    pub fn span(&self, id: NodeId) -> Range<usize> {
        self.span_map[&id].clone()
    }
    pub fn text(&self, id: NodeId) -> &str {
        let span = self.span(id);
        &self.source[span]
    }
    pub fn snippet(&self, id: NodeId) -> &str {
        let span = self.span(id);
        &self.source[span]
    }
}

impl Hash for SpannedSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.name.hash(state);
        for (id, val) in &self.span_map {
            id.hash(state);
            val.start.hash(state);
            val.end.hash(state);
        }
        self.fallback.hash(state);
    }
}
