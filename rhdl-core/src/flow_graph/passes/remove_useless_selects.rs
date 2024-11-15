use std::collections::HashSet;

use petgraph::{
    graph::{self, EdgeIndex},
    visit::{EdgeRef, NodeFilteredNodes},
};

use crate::{
    flow_graph::{
        component::ComponentKind,
        edge_kind::EdgeKind,
        flow_graph_impl::{FlowIx, GraphType},
    },
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUselessSelectsPass {}

fn get_select_constant_replacement(node: FlowIx, graph: &GraphType) -> Option<bool> {
    if matches!(graph[node].kind, ComponentKind::Select) {
        // See if the true path resolves to a constant
        let true_path = graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .find(|edge| matches!(edge.weight(), EdgeKind::True))?;
        let false_path = graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .find(|edge| matches!(edge.weight(), EdgeKind::False))?;
        let ComponentKind::Constant(true_path) = graph[true_path.source()].kind else {
            return None;
        };
        let ComponentKind::Constant(false_path) = graph[false_path.source()].kind else {
            return None;
        };
        if true_path == false_path {
            return Some(true_path);
        }
    }
    None
}

impl Pass for RemoveUselessSelectsPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let candidates = graph
            .node_indices()
            .flat_map(|node| {
                get_select_constant_replacement(node, &graph).map(|replacement| (node, replacement))
            })
            .collect::<Vec<_>>();
        let edges_to_drop = candidates
            .iter()
            .flat_map(|node| {
                graph
                    .edges_directed(node.0, petgraph::Direction::Incoming)
                    .map(|edge| edge.id())
            })
            .collect::<HashSet<EdgeIndex>>();
        for (node, replacement) in candidates {
            graph.node_weight_mut(node).unwrap().kind = ComponentKind::Constant(replacement);
        }
        graph.retain_edges(|_, edge| !edges_to_drop.contains(&edge));
        Ok(FlowGraph { graph, ..input })
    }
}
