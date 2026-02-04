use miette::{MietteError, SourceCode, SourceSpan, SpanContents};

use crate::ast::ast_impl::{FunctionId, NodeId, SourceLocation};
use std::hash::Hash;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hasher,
    ops::Range,
};

#[derive(Clone, Debug, Default, Hash)]
pub struct SourcePool {
    pub source: BTreeMap<FunctionId, SpannedSource>,
    pub ranges: BTreeMap<FunctionId, Range<usize>>,
}

impl SourcePool {
    pub fn new(source: &BTreeMap<FunctionId, SpannedSource>) -> SourcePool {
        let mut ranges = BTreeMap::new();
        let mut offset = 0;
        for (id, src) in source {
            let len = src.source.len();
            ranges.insert(*id, offset..offset + len);
            offset += len;
        }
        SourcePool {
            source: source.clone(),
            ranges,
        }
    }
}

impl SourceCode for SourcePool {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let start = span.offset();
        let len = span.len();
        if let Some((function_id, function_range)) = self
            .ranges
            .iter()
            .find(|(_id, range)| range.contains(&start))
        {
            let local_offset = start - function_range.start;
            let local_span = SourceSpan::new(local_offset.into(), len);
            let source = self
                .source
                .get(function_id)
                .ok_or(MietteError::OutOfBounds)?;
            let local =
                source
                    .source
                    .read_span(&local_span, context_lines_before, context_lines_after)?;
            let local_span = local.span();
            let span = (local_span.offset() + function_range.start, local_span.len()).into();
            Ok(Box::new(miette::MietteSpanContents::new_named(
                source.filename.clone(),
                local.data(),
                span,
                local.line(),
                local.column(),
                local.line_count(),
            )))
        } else {
            Err(MietteError::OutOfBounds)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpannedSourceSet {
    pub sources: BTreeMap<FunctionId, SpannedSource>,
}

impl SpannedSourceSet {
    pub fn source(&self) -> SourcePool {
        SourcePool::new(&self.sources)
    }
    pub fn span<T: Into<SourceLocation>>(&self, loc: T) -> Range<usize> {
        let loc: SourceLocation = loc.into();
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
    pub fn filename(&self, id: FunctionId) -> &str {
        &self.sources[&id].filename
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
