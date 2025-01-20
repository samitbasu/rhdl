use crate::rhdl_core::{
    ast::source::source_location::SourceLocation,
    error::rhdl_error,
    flow_graph::{
        error::{FlowGraphError, FlowGraphICE},
        flow_graph_impl::FlowGraph,
    },
    RHDLError,
};

pub trait Pass {
    fn raise_ice(fg: &FlowGraph, cause: FlowGraphICE, loc: Option<SourceLocation>) -> RHDLError {
        let elements = if let Some(loc) = loc {
            vec![fg.code.span(loc).into()]
        } else {
            vec![]
        };
        rhdl_error(FlowGraphError {
            cause,
            src: fg.code.source(),
            elements,
        })
    }
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError>;
}
