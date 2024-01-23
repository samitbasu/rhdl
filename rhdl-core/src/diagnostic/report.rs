use crate::ast::ast_impl::NodeId;

use super::SpannedSource;

use ariadne::{Label, Report, ReportKind, Source};

pub fn show_source(source: &SpannedSource, label: &str, node: NodeId) {
    if let Some(span) = source.span_map.get(&node).cloned() {
        let _ = Report::build(ReportKind::Error, &source.name, 0)
            .with_label(Label::new((&source.name, span)).with_message(label))
            .finish()
            .print((&source.name, Source::from(&source.source)));
    }
}
