use petgraph::visit::EdgeRef;

use crate::{
    flow_graph::{component::ComponentKind, flow_graph_impl::FlowIx},
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ConstantBufferEliminationPass {}

struct Candidate {
    buffer: FlowIx,
    source: FlowIx,
}

impl Pass for ConstantBufferEliminationPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        // Do this in several passes.  The first pass, scans the graph and finds buffer nodes
        // with a single input that is a non-buffer node.
        let buffer_nodes = graph
            .node_indices()
            .filter(|node| !input.output.contains(node))
            .filter(|node| graph.edges_directed(*node, petgraph::Incoming).count() == 1)
            .filter_map(|candidate| {
                let weight = &graph[candidate];
                if let ComponentKind::Buffer(_) = weight.kind {
                    let mut edges = graph.edges_directed(candidate, petgraph::Incoming);
                    if let Some(edge) = edges.next() {
                        let source = edge.source();
                        let source_weight = &graph[source];
                        if !matches!(source_weight.kind, ComponentKind::Buffer(_)) {
                            return Some(Candidate {
                                buffer: candidate,
                                source,
                            });
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        eprintln!("Eliminating {} buffer nodes", buffer_nodes.len());
        // Second pass, for each buffer, take the each outgoing edge and set the target of that
        // outgoing edge to the source of the incoming edge.
        for candidate in &buffer_nodes {
            let outgoing_edges = graph
                .edges_directed(candidate.buffer, petgraph::Outgoing)
                .map(|edge| {
                    let target = edge.target();
                    let weight = edge.weight().clone();
                    (target, weight)
                })
                .collect::<Vec<_>>();
            for (edge_target, edge_weight) in outgoing_edges {
                graph.add_edge(candidate.source, edge_target, edge_weight);
            }
        }
        // Third pass, we remove the edges that were leading to the buffer node or out of it
        graph.retain_edges(|graph, edge| {
            let (source, target) = graph.edge_endpoints(edge).unwrap();
            !buffer_nodes
                .iter()
                .any(|candidate| candidate.buffer == target || candidate.buffer == source)
        });
        // Fourth pass, we remove the buffer itself
        for candidate in buffer_nodes {
            graph.remove_node(candidate.buffer);
        }
        Ok(FlowGraph { graph, ..input })
    }
}
