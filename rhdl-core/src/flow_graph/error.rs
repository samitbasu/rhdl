use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::SourcePool;

use super::component::ComponentKind;

#[derive(Error, Debug, Diagnostic)]
pub enum FlowGraphICE {
    #[error("Flow graph contains an undriven node of kind {kind:?}")]
    UndrivenNode { kind: ComponentKind },
    #[error("Flow graph contains a logic loop")]
    #[diagnostic(help(
        "The flow graph includes a loop of logic elements which is not allowed.  That loop includes some of the elements identified."
    ))]
    LogicLoop,
    #[error("Flow graph is not sealed")]
    UnSealedFlowGraph,
    #[error("Clock or reset signal is driven by a constant")]
    #[diagnostic(help(
        "The flow graph includes these elements, but does not connect them to a valid clock source.  Check that the clock and reset signals are propagated through these elements"
    ))]
    UnconnectedClockReset,
    #[error("Expected a select component")]
    ExpectedSelectComponent,
    #[error("Select node has no control node")]
    SelectControlNodeNotFound,
    #[error("Select node has no true node")]
    SelectTrueNodeNotFound,
    #[error("Select node has no false node")]
    SelectFalseNodeNotFound,
    #[error("Expected a buffer component")]
    ExpectedBufferComponent,
    #[error("Expected a binary component")]
    ExpectedBinaryComponent,
    #[error("Expected a unary component")]
    ExpectedUnaryComponent,
    #[error("Expected a dff component")]
    ExpectedDFFComponent,
    #[error("DFF input has no driver")]
    DFFInputDriverNotFound,
    #[error("Buffer node has no parent")]
    BufferParentNotFound,
    #[error("Missing argument to component with bit {bit}")]
    MissingArgument { bit: usize },
    #[error("Clock for DFF not found")]
    ClockNotFound,
    #[error("Reset for DFF not found")]
    ResetNotFound,
    #[error("Expected a case component")]
    ExpectedCaseComponent,
    #[error("Black box nodes are not supported")]
    BlackBoxNotSupported,
}

#[derive(Debug, Error)]
pub struct FlowGraphError {
    pub cause: FlowGraphICE,
    pub src: SourcePool,
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
