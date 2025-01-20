use std::collections::HashSet;

use petgraph::visit::EdgeRef;

use crate::rhdl_core::{
    bitx::BitX,
    flow_graph::{
        component::ComponentKind,
        edge_kind::EdgeKind,
        flow_graph_impl::{FlowIx, GraphType},
    },
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerSelectToBufferPass {}

struct Replacement {
    original: FlowIx,
    replacement: FlowIx,
}

fn is_identity_select(node: FlowIx, graph: &GraphType) -> Option<Replacement> {
    let ComponentKind::Select = &graph[node].kind else {
        return None;
    };
    let true_path = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .find(|edge| matches!(edge.weight(), EdgeKind::True))?;
    let false_path = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .find(|edge| matches!(edge.weight(), EdgeKind::False))?;
    if !matches!(
        graph[true_path.source()].kind,
        ComponentKind::Constant(BitX::One)
    ) {
        return None;
    }
    if !matches!(
        graph[false_path.source()].kind,
        ComponentKind::Constant(BitX::Zero),
    ) {
        return None;
    }
    let selector_path = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .find(|edge| matches!(edge.weight(), EdgeKind::Selector(0)))?;
    Some(Replacement {
        original: node,
        replacement: selector_path.source(),
    })
}

impl Pass for LowerSelectToBufferPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let candidates = graph
            .node_indices()
            .filter_map(|node| is_identity_select(node, &graph))
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
