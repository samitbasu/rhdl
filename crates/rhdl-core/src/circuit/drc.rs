//! Design Rule Checking for Circuits
//!
//! Eventually this module will contain various design rule checking functions to apply to RHDL circuits.
//! For now, it contains a single check for combinatorial paths in synchronous circuits.
//!
//! A combinatorial path is a path from an input to an output that does not terminate on any flip flops or
//! other black box components.  Some specifications discourage or even forbid combinatorial paths, as they can
//! lead to timing issues.  This module provides a function to check for such paths and report them.
//!
//! See the [book] for an example of how to use it.
use crate::{Synchronous, ast::SourcePool, circuit::scoped_name::ScopedName};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Diagnostic for combinatorial paths in synchronous circuits.
#[derive(Debug, Error)]
#[error("RHDL Combinatorial Path")]
pub struct CombinatorialPath {
    src: SourcePool,
    elements: Vec<SourceSpan>,
}

impl Diagnostic for CombinatorialPath {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(
            "This is a combinatorial pathway between an input and an output",
        ))
    }
    fn labels<'a>(
        &'a self,
    ) -> Option<Box<dyn std::iter::Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(self.elements.iter().map(|span| {
            miette::LabeledSpan::new_primary_with_span(None, *span)
        })))
    }
}

/// Check that the given synchronous circuit has no combinatorial paths from inputs to outputs.
///
/// This function analyzes the netlist of the circuit and looks for paths from inputs to outputs
/// that do not pass through any flip flops or black box components.  If such paths are found,
/// it returns an error with a diagnostic that includes the source locations of the elements
/// involved in the path.
///
pub fn no_combinatorial_paths<T: Synchronous>(uut: &T) -> miette::Result<()> {
    let descriptor = uut.descriptor(ScopedName::top())?;
    let schematic = descriptor.schematic()?;
    schematic.check_for_combinatorial_io_paths()?;
    return Ok(());
}
