use petgraph::visit::EdgeRef;

use crate::{flow_graph::component::ComponentKind, FlowGraph, RHDLError};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveOrphanConstantsPass {}

impl Pass for RemoveOrphanConstantsPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        graph.retain_edges(|frozen, edge| {
            if let Some(endpoints) = frozen.edge_endpoints(edge) {
                let source = endpoints.0;
                let target = endpoints.1;
                frozen.contains_node(source) && frozen.contains_node(target)
            } else {
                false
            }
        });
        for node in graph.node_indices() {
            let component = graph.node_weight(node).unwrap();
            for edge in graph.edges(node) {
                let source = edge.source();
                let target = edge.target();
                eprintln!(
                    "component: {node:?} -> {component:?} [{source:?} -> {target:?}]",
                    component = component.kind
                );
            }
        }
        graph.retain_nodes(|graph, node| {
            let component = graph.node_weight(node).unwrap();
            let outgoing = graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .count();
            !matches!(
                component.kind,
                ComponentKind::Constant(_) | ComponentKind::BitSelect(_)
            ) || outgoing > 0
        });
        Ok(FlowGraph { graph, ..input })
    }
}
