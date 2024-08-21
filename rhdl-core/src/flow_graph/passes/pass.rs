use crate::{flow_graph::flow_graph_impl::FlowGraph, RHDLError};

pub trait Pass {
    fn name() -> &'static str;
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError>;
}
