use std::collections::HashMap;

use petgraph::visit::EdgeRef;

use crate::{flow_graph::component::ComponentKind, FlowGraph, RHDLError};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ConstantBufferEliminationPass {}

impl Pass for ConstantBufferEliminationPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        // Do this in several passes.  The first pass, scans the graph and finds buffer nodes
        // with a single input that is a constant node.
        let buffer_nodes = graph
            .node_indices()
            .filter(|node| !input.output.contains(node))
            .filter_map(|candidate| {
                let weight = &graph[candidate];
                if let ComponentKind::Buffer(_) = weight.kind {
                    let mut edges = graph.edges_directed(candidate, petgraph::Incoming);
                    if let Some(edge) = edges.next() {
                        if edges.next().is_none() {
                            let source = edge.source();
                            let source_weight = &graph[source];
                            if let ComponentKind::Constant(c) = source_weight.kind {
                                return Some((candidate, c));
                            }
                        }
                    }
                }
                None
            })
            .collect::<HashMap<_, _>>();
        eprintln!("Eliminating {} buffer nodes", buffer_nodes.len());
        // Second pass, we rewrite the node weights of the buffer nodes to be constant nodes.
        for (buffer_node, value) in &buffer_nodes {
            graph[*buffer_node].kind = ComponentKind::Constant(*value);
        }
        // Third pass, we remove the edges that were leading to the buffer nodes.
        graph.retain_edges(|graph, edge| {
            let (_, target) = graph.edge_endpoints(edge).unwrap();
            !buffer_nodes.contains_key(&target)
        });
        Ok(FlowGraph { graph, ..input })
    }
}
