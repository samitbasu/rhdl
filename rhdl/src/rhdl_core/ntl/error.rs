use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::rhdl_core::SourcePool;

#[derive(Error, Debug, Diagnostic)]
pub enum NetListICE {
    #[error("Expected a register to write to, not a constant")]
    ExpectedRegisterNotConstant,
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
