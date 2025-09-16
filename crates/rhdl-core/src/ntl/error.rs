use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::ast::SourcePool;

#[derive(Error, Debug, Diagnostic)]
pub enum NetListICE {
    #[error("Expected a register to write to, not a constant")]
    ExpectedRegisterNotConstant,
    #[error("Design contains a logic loop")]
    #[diagnostic(help(
        "The design includes a loop of logic elements which is not allowed.  That loop includes the identified instruction."
    ))]
    LogicLoop,
    #[error("Net list contains an undriven node")]
    UndrivenNetlistNode,
}

#[derive(Debug, Error)]
pub struct NetLoopError {
    pub src: SourcePool,
    pub elements: Vec<(Option<String>, SourceSpan)>,
}

impl std::fmt::Display for NetLoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Design contains a logic loop")
    }
}

impl Diagnostic for NetLoopError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(
            "Design contains a loop.  You need to either insert a flip flop on the path or otherwise break the loop.",
        ))
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(self.elements.iter().map(|(text, span)| {
            miette::LabeledSpan::new_primary_with_span(text.clone(), *span)
        })))
    }
}

#[derive(Debug, Error)]
pub struct NetListError {
    pub cause: NetListICE,
    pub src: SourcePool,
    pub elements: Vec<SourceSpan>,
}

impl std::fmt::Display for NetListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Diagnostic for NetListError {
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
