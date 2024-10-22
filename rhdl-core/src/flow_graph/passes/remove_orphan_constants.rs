use crate::{flow_graph::component::ComponentKind, FlowGraph, RHDLError};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveOrphanConstantsPass {}

impl Pass for RemoveOrphanConstantsPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        graph.retain_nodes(|graph, node| {
            let component = graph.node_weight(node).unwrap();
            let outgoing = graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .count();
            !(matches!(
                component.kind,
                ComponentKind::Constant(_) | ComponentKind::BitString(_)
            ) && outgoing == 0)
        });
        Ok(FlowGraph { graph, ..input })
    }
}
