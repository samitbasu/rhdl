use crate::rhdl_core::{flow_graph::component::ComponentKind, FlowGraph, RHDLError};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveOrphanConstantsPass {}

impl Pass for RemoveOrphanConstantsPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        graph.retain_nodes(|graph, node| {
            let component = graph.node_weight(node).unwrap();
            let has_outgoing = graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .count()
                != 0;
            let is_not_constant = !matches!(
                component.kind,
                ComponentKind::Constant(_) | ComponentKind::BitString(_)
            );
            has_outgoing || is_not_constant
        });
        Ok(FlowGraph { graph, ..input })
    }
}
