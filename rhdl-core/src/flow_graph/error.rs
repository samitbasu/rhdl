use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum FlowGraphICE {
    #[error("Flow graph contains an undriven node")]
    UndrivenNode,
    #[error("Flow graph is not sealed")]
    UnSealedFlowGraph,
    #[error("Clock or reset signal is driven by a constant")]
    #[diagnostic(help(
        "The flow graph includes these elements, but does not connect them to a valid clock source.  Check that the clock and reset signals are propagated through these elements"
    ))]
    UnconnectedClockReset,
}

#[derive(Debug, Error)]
pub struct FlowGraphError {
    pub cause: FlowGraphICE,
    pub src: String,
    pub elements: Vec<SourceSpan>,
}

impl std::fmt::Display for FlowGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Diagnostic for FlowGraphError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.help()
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(self.elements.iter().map(|span| {
            miette::LabeledSpan::new_primary_with_span(None, *span)
        })))
    }
}
