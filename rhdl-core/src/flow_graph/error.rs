use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum FlowGraphICE {
    #[error("Flow graph contains an undriven node")]
    UndrivenNode,
}

#[derive(Debug, Error)]
#[error("RHDL Flow Graph Error")]
pub struct FlowGraphError {
    pub cause: FlowGraphICE,
    pub src: String,
    pub err_span: Option<SourceSpan>,
}

impl Diagnostic for FlowGraphError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.help()
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(self.err_span.iter().map(|span| {
            miette::LabeledSpan::new_primary_with_span(Some(self.cause.to_string()), *span)
        })))
    }
}
