use std::collections::{HashMap, HashSet};

use petgraph::Graph;

use crate::{
    flow_graph::{component::ComponentKind, flow_graph_impl::FlowGraph},
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ElideBuffers {}

impl Pass for ElideBuffers {
    fn name() -> &'static str {
        "elide_buffers"
    }
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        // First pass, find all buffers that have a single incoming edge and a single outgoing edge.
        let buffer_nodes = graph.node_indices().filter(|node| {
            matches!(graph[*node].kind, ComponentKind::Buffer(_)) && graph.edges_directed(*node, petgraph::Incoming).count() == 1 && graph.edges_directed(*node, petgraph::Outgoing).count() == 1
        }).collect::<Vec<_>>();
        // Second pass, 

        let mut subst_map = HashMap::new();
        let mut output_graph = Graph::new();
        let mut topo = petgraph::visit::Topo::new(&input.graph);
        while let Some(ix) = topo.next(&input.graph) {
            if let ComponentKind::Buffer(_) = &input.graph[ix].kind {
                let outgoing_edges = input.graph.edges_directed(ix, petgraph::Outgoing);
                let replacements = outgoing_edges.map(|edge| subst_map[&edge.target()]);
            }
            // Get the cost for this node
            let node_cost = cost(ix, fg, &cost_map);
            cost_map.insert(ix, node_cost);
        }
    }
        for node in input.graph.node_indices() {
            let component = &input.graph[node];
            let incoming_edges = input.graph.edges_directed(node, petgraph::Incoming);
            let outgoing_edges = input.graph.edges_directed(node, petgraph::Outgoing);
            if let ComponentKind::Buffer(_) = &component.kind {
            } else {
                let new_index = output_graph.add_node(component.clone());
                subst_map.insert(node, new_index);

            }
        }
        Ok(input)
    }
}
