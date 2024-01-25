use std::collections::HashMap;

use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    rhif::object::SourceLocation,
};

use super::SpannedSource;

use ariadne::{Label, Report, ReportKind, Source};

pub fn show_source_detail(sources: &HashMap<FunctionId, SpannedSource>, location: SourceLocation) {
    if let Some(source) = sources.get(&location.func) {
        if let Some(span) = source.span_map.get(&location.node).cloned() {
            let _ = Report::build(ReportKind::Error, &source.name, 0)
                .with_label(Label::new((&source.name, span)).with_message("Error"))
                .finish()
                .print((&source.name, Source::from(&source.source)));
        }
    }
}

pub fn show_source(source: &SpannedSource, label: &str, node: NodeId) {
    if let Some(span) = source.span_map.get(&node).cloned() {
        let _ = Report::build(ReportKind::Error, &source.name, 0)
            .with_label(Label::new((&source.name, span)).with_message(label))
            .finish()
            .print((&source.name, Source::from(&source.source)));
    }
}
