use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

use crate::{
    RHDLError,
    circuit::schematic::{PortId, Schematic, SchematicKind, error::SchematicICE},
    error::rhdl_error,
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

fn dump_schematic(schematic: &Schematic) {
    if matches!(schematic.kind, SchematicKind::Kernel) {
        eprintln!("------------------------------------------");
        eprintln!(
            "Schematic: {:?} {} (id: {:?})",
            schematic.kind, schematic.name, schematic.id
        );
        eprintln!("{}", schematic.debug_text);
    }
    for child in &schematic.inner {
        dump_schematic(child);
    }
}

pub fn loop_check(schematic: &Schematic) -> Result<(), RHDLError> {
    let mut graph = DiGraph::<PortId, ()>::default();
    let mut map = HashMap::<PortId, NodeIndex>::default();
    map_schematic_to_graph(schematic, &mut graph, &mut map);
    link_up_graph(schematic, &mut graph, &map);
    if let Err(cycle) = petgraph::algo::toposort(&graph, None) {
        let cycle_node = cycle.node_id();
        let components = petgraph::algo::kosaraju_scc(&graph);
        for component in components {
            if component.contains(&cycle_node) {
                dump_schematic(schematic);
                let logic_loop = component
                    .iter()
                    .map(|node_index| graph[*node_index])
                    .collect();
                return Err(rhdl_error(SchematicICE::LogicLoop {
                    schematic: std::sync::Arc::new(schematic.clone()),
                    logic_loop,
                }));
            }
        }
    }
    Ok(())
}
