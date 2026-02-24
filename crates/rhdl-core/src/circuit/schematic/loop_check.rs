use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

use crate::{
    RHDLError,
    circuit::schematic::{PortId, Schematic},
};

fn map_schematic_to_graph(
    schematic: &Schematic,
    graph: &mut DiGraph<PortId, ()>,
    map: &mut HashMap<PortId, NodeIndex>,
) {
    for input in schematic.inputs.iter().flatten() {
        let node_index = graph.add_node(input.id);
        map.insert(input.id, node_index);
    }
    for output in &schematic.outputs {
        let node_index = graph.add_node(output.id);
        map.insert(output.id, node_index);
    }
    for child in &schematic.inner {
        map_schematic_to_graph(child, graph, map);
    }
}

fn link_up_graph(
    schematic: &Schematic,
    graph: &mut DiGraph<PortId, ()>,
    map: &HashMap<PortId, NodeIndex>,
) {
    for link in &schematic.links {
        let from_index = map.get(&link.from).unwrap();
        let to_index = map.get(&link.to).unwrap();
        graph.add_edge(*from_index, *to_index, ());
    }
    for child in &schematic.inner {
        link_up_graph(child, graph, map);
    }
}

pub fn loop_check(schematic: &Schematic) -> Result<(), RHDLError> {
    let mut graph = DiGraph::<PortId, ()>::default();
    let mut map = HashMap::<PortId, NodeIndex>::default();
    map_schematic_to_graph(schematic, &mut graph, &mut map);
    link_up_graph(schematic, &mut graph, &map);
    if let Err(cycle) = petgraph::algo::toposort(&graph, None) {
        panic!(
            "Schematic contains a cycle involving port {:?}",
            graph[cycle.node_id()]
        );
    }
    Ok(())
}
