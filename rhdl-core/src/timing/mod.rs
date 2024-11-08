use std::collections::HashMap;

use petgraph::visit::EdgeRef;

use crate::flow_graph::{
    component::ComponentKind,
    flow_graph_impl::{FlowGraph, FlowIx},
};

// Not a realistic model.  Just counts the number of non-trivial ops in the path.
pub fn simplest_cost(node: FlowIx, fg: &FlowGraph, cost_map: &HashMap<FlowIx, f64>) -> f64 {
    let component = &fg.graph[node];
    let max_incoming = fg
        .graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .map(|edge| cost_map[&edge.source()])
        .fold(0.0, f64::max);
    match component.kind {
        ComponentKind::Binary(_)
        | ComponentKind::Case(_)
        | ComponentKind::DynamicIndex(_)
        | ComponentKind::DynamicSplice(_)
        | ComponentKind::Select
        | ComponentKind::Unary(_) => max_incoming + 1.0,
        _ => max_incoming,
    }
}

pub fn compute_node_costs<F: FnMut(FlowIx, &FlowGraph, &HashMap<FlowIx, f64>) -> f64>(
    fg: &mut FlowGraph,
    mut cost: F,
) {
    // Visit the graph in topological order - we will start at the timing source
    let mut cost_map = HashMap::new();
    {
        let mut topo = petgraph::visit::Topo::new(&fg.graph);
        while let Some(ix) = topo.next(&fg.graph) {
            // Get the cost for this node
            let node_cost = cost(ix, fg, &cost_map);
            cost_map.insert(ix, node_cost);
        }
    }
    eprintln!("Cost map: {:?}", cost_map);
}
