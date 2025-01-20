use crate::rhdl_core::{flow_graph::component::ComponentKind, FlowGraph, RHDLError};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnusedBuffers {}

impl Pass for RemoveUnusedBuffers {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        graph.retain_nodes(|graph, node| {
            let component = graph.node_weight(node).unwrap();
            let outgoing = graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .count();
            let is_used_by_output = input.output.contains(&node);
            !matches!(component.kind, ComponentKind::Buffer(_)) || outgoing > 0 || is_used_by_output
        });
        Ok(FlowGraph { graph, ..input })
    }
}
