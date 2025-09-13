use miette::{MietteError, SourceCode, SourceSpan, SpanContents};

use crate::ast::ast_impl::FunctionId;
use std::{collections::BTreeMap, hash::Hash, ops::Range};

use super::spanned_source::SpannedSource;

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
