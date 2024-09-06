use crate::{
    ast::source_location::SourceLocation,
    error::rhdl_error,
    flow_graph::{
        error::{FlowGraphError, FlowGraphICE},
        flow_graph_impl::FlowGraph,
    },
    RHDLError,
};

pub trait Pass {
    fn raise_ice(fg: &FlowGraph, cause: FlowGraphICE, loc: Option<SourceLocation>) -> RHDLError {
        rhdl_error(FlowGraphError {
            cause,
            src: fg.code.source(),
            err_span: loc.map(|x| fg.code.span(x).into()),
        })
    }
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError>;
}
