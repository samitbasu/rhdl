use super::FlowGraph;
use std::io::Result;
use std::io::Write;

pub fn write_dot(flow_graph: &FlowGraph, mut w: impl Write) -> Result<()> {
    let dot = petgraph::dot::Dot::new(&flow_graph.graph);
    write!(w, "{:?}", dot)
}
