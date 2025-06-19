use miette::{Diagnostic, SourceSpan};
use petgraph::algo::DfsSpace;

use crate::{
    prelude::Synchronous,
    rhdl_core::{
        ntl::{
            graph::{make_net_graph, GraphMode, WriteSource},
            spec::Operand,
        },
        SourcePool,
    },
};
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
    let descriptor = uut.descriptor("uut")?;
    let ntl = &descriptor.ntl;
    let dep = make_net_graph(ntl, GraphMode::Synchronous);
    // Get the graph node that represents the inputs for the device
    let input_node = dep.input_node;
    let mut space = DfsSpace::new(&dep.graph);
    let code = &descriptor.ntl.code;
    for output in descriptor.ntl.outputs.iter().flat_map(Operand::reg) {
        let source = dep.reg_map[&output];
        match source {
            WriteSource::ClockReset => {}
            WriteSource::Input => {
                return Err(miette::Report::new(CombinatorialPath {
                    src: code.source(),
                    elements: Vec::new(),
                }))
            }
            WriteSource::OpCode(ndx) => {
                // The output is written by the opcode ndx.
                // Get the node from the graph
                let op_node = dep.op_nodes[ndx];
                if petgraph::algo::has_path_connecting(
                    &dep.graph,
                    input_node,
                    op_node,
                    Some(&mut space),
                ) {
                    let path = petgraph::algo::all_simple_paths::<Vec<_>, _>(
                        &dep.graph, input_node, op_node, 1, None,
                    )
                    .next()
                    .unwrap();
                    let elements = path
                        .iter()
                        .map(|ix| dep.graph[*ix])
                        .filter_map(|ws| match ws {
                            WriteSource::OpCode(ndx) => Some(ndx),
                            _ => None,
                        })
                        .flat_map(|x| ntl.ops[x].loc)
                        .map(|loc| SourceSpan::from(code.span(loc)))
                        .collect();
                    return Err(miette::Report::new(CombinatorialPath {
                        src: code.source(),
                        elements,
                    }));
                }
            }
        }
    }
    Ok(())
}
