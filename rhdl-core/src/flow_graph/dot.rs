use std::io::Result;
use std::io::Write;

use petgraph::visit::EdgeRef;

use super::flow_graph_impl::FlowGraph;

pub fn write_dot(flow_graph: &FlowGraph, mut w: impl Write) -> Result<()> {
    let dot = petgraph::dot::Dot::new(&flow_graph.graph);
    write!(w, "{:?}", dot)
}
