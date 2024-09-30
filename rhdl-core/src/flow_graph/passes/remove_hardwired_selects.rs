use std::collections::HashMap;

use petgraph::{graph::NodeIndex, visit::EdgeRef};

use crate::{
    flow_graph::{component::ComponentKind, edge_kind::EdgeKind, flow_graph_impl::GraphType},
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveHardwiredSelectsPass {}

fn get_constant(graph: &GraphType, node: NodeIndex) -> Option<bool> {
    let weight = &graph[node];
    if let ComponentKind::Constant(value) = weight.kind {
        return Some(value);
    }
    None
}

fn is_constant(graph: &GraphType, node: NodeIndex) -> bool {
    let weight = &graph[node];
    if let ComponentKind::Constant(_) = weight.kind {
        return true;
    }
    false
}

fn get_select_control_node(graph: &GraphType, node: NodeIndex) -> Option<NodeIndex> {
    graph
        .edges_directed(node, petgraph::Incoming)
        .find_map(|edge| match edge.weight() {
            EdgeKind::Selector(0) => Some(edge.source()),
            _ => None,
        })
}

fn get_select_data_node(graph: &GraphType, node: NodeIndex, select: bool) -> Option<NodeIndex> {
    graph
        .edges_directed(node, petgraph::Incoming)
        .find_map(|edge| match edge.weight() {
            EdgeKind::True if select => Some(edge.source()),
            EdgeKind::False if !select => Some(edge.source()),
            _ => None,
        })
}

fn is_select_with_hardwired_control(graph: &GraphType, node: NodeIndex) -> bool {
    let weight = &graph[node];
    if let ComponentKind::Select = weight.kind {
        if let Some(control_node) = get_select_control_node(graph, node) {
            return is_constant(graph, control_node);
        }
    }
    false
}

impl Pass for RemoveHardwiredSelectsPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let target_nodes = graph
            .node_indices()
            .filter(|node| is_select_with_hardwired_control(&graph, *node))
            .collect::<Vec<_>>();
        let mut remap: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        for target in target_nodes {
            // Get the control node
            if let Some(control_node) = get_select_control_node(&graph, target) {
                // Get the control value
                if let Some(control_value) = get_constant(&graph, control_node) {
                    if let Some(data_node) = get_select_data_node(&graph, target, control_value) {
                        remap.insert(target, data_node);
                    }
                }
            }
        }
        graph.retain_edges(|frozen, edge| {
            let (_source, target) = frozen.edge_endpoints(edge).unwrap();
            !remap.contains_key(&target)
        });
        for (target, replacement) in remap {
            graph.node_weight_mut(target).unwrap().kind = ComponentKind::Buffer("opt_sel".into());
            graph.add_edge(replacement, target, EdgeKind::Arg(0));
        }
        Ok(FlowGraph { graph, ..input })
    }
}
