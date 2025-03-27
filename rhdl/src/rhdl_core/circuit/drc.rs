use std::collections::HashSet;

use miette::{Diagnostic, SourceSpan};

use crate::{prelude::Synchronous, rhdl_core::SourcePool};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("RHDL Combinatorial Path")]
pub struct CombinatorialPath {
    pub src: SourcePool,
    pub elements: Vec<SourceSpan>,
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

pub fn no_combinatorial_paths<T: Synchronous>(uut: &T) -> miette::Result<()> {
    let flow_graph = uut.flow_graph("top")?;
    // We know that input[1] is the one we care about. input[0] is the clock and reset input.
    let inputs: HashSet<_> = flow_graph.inputs[1].iter().collect();
    let outputs: HashSet<_> = flow_graph.output.iter().collect();
    let g = &flow_graph.graph;
    let mut workspace = petgraph::algo::DfsSpace::new(g);
    for input in inputs {
        for output in &outputs {
            if petgraph::algo::has_path_connecting(g, *input, **output, Some(&mut workspace)) {
                let path =
                    petgraph::algo::all_simple_paths::<Vec<_>, _>(g, *input, **output, 1, None)
                        .next()
                        .unwrap();
                let elements = path
                    .iter()
                    .filter_map(|ix| g[*ix].location)
                    .map(|x| flow_graph.code.span(x).into())
                    .collect();
                return Err(miette::Report::new(CombinatorialPath {
                    src: flow_graph.code.source(),
                    elements,
                }));
            }
        }
    }
    Ok(())
}
