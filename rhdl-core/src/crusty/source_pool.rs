use miette::{MietteError, SourceCode, SourceSpan, SpanContents};
use std::{collections::HashMap, ops::Range, sync::Arc};

use crate::{
    ast::ast_impl::FunctionId,
    rhif::{object::SourceLocation, spanned_source::SpannedSource},
};

#[derive(Clone, Debug)]
pub struct SourcePool {
    pub source: HashMap<FunctionId, SpannedSource>,
    pub ranges: HashMap<FunctionId, Range<usize>>,
}

impl SourcePool {
    pub(crate) fn new(source: HashMap<FunctionId, SpannedSource>) -> Self {
        let mut ranges = HashMap::new();
        let mut offset = 0;
        for (id, src) in &source {
            let len = src.source.len();
            ranges.insert(*id, offset..offset + len);
            offset += len;
        }
        Self { source, ranges }
    }

    pub(crate) fn get_range_from_location(&self, location: SourceLocation) -> Option<Range<usize>> {
        let range = self.ranges.get(&location.func)?;
        let local_span = self
            .source
            .get(&location.func)?
            .span_map
            .get(&location.node)?;
        let start = range.start + local_span.start;
        let end = range.start + local_span.end;
        Some(start..end)
    }
}

#[derive(Clone, Debug)]
pub struct SharedSourcePool {
    pub pool: Arc<SourcePool>,
}

impl SharedSourcePool {
    pub(crate) fn get_range_from_location(&self, location: SourceLocation) -> Option<Range<usize>> {
        self.pool.get_range_from_location(location)
    }
}

impl From<SourcePool> for SharedSourcePool {
    fn from(pool: SourcePool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

impl SourceCode for SharedSourcePool {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        self.pool
            .read_span(span, context_lines_before, context_lines_after)
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
        if let Some((function_id, function_range)) =
            self.ranges.iter().find(|(_, range)| range.contains(&start))
        {
            let local_offset = start - function_range.start;
            let local_span = SourceSpan::new(local_offset.into(), len);
            let source = self.source.get(function_id).unwrap();
            let local =
                source
                    .source
                    .read_span(&local_span, context_lines_before, context_lines_after)?;
            let local_span = local.span();
            let span = (local_span.offset() + function_range.start, local_span.len()).into();
            Ok(Box::new(miette::MietteSpanContents::new_named(
                source.name.clone(),
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
