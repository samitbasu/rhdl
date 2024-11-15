use std::collections::HashSet;

use petgraph::visit::EdgeRef;

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
pub struct LowerSelectWithIdenticalArgs {}

struct Replacement {
    original: FlowIx,
    replacement: FlowIx,
}

fn is_buffer_like_select(node: FlowIx, graph: &GraphType) -> Option<Replacement> {
    let ComponentKind::Select = &graph[node].kind else {
        return None;
    };
    let true_path = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .find(|edge| matches!(edge.weight(), EdgeKind::True))?;
    let false_path = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .find(|edge| matches!(edge.weight(), EdgeKind::False))?;
    let true_node = true_path.source();
    let false_node = false_path.source();
    if true_node == false_node {
        return Some(Replacement {
            original: node,
            replacement: true_node,
        });
    }
    None
}

impl Pass for LowerSelectWithIdenticalArgs {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let candidates = graph
            .node_indices()
            .filter_map(|node| is_buffer_like_select(node, &graph))
            .collect::<Vec<_>>();
        let edges_to_drop: HashSet<_> = candidates
            .iter()
            .flat_map(|node| {
                graph
                    .edges_directed(node.original, petgraph::Direction::Incoming)
                    .map(|edge| edge.id())
            })
            .collect();
        graph.retain_edges(|_, edge| !edges_to_drop.contains(&edge));
        for Replacement {
            original,
            replacement,
        } in candidates
        {
            graph.node_weight_mut(original).unwrap().kind =
                ComponentKind::Buffer(format!("select_{original:?}"));
            graph.add_edge(replacement, original, EdgeKind::ArgBit(0, 0));
        }
        Ok(FlowGraph { graph, ..input })
    }
}
