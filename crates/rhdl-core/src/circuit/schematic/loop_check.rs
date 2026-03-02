use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

use petgraph::{
    graph::{DiGraph, Graph, NodeIndex},
    unionfind::UnionFind,
    visit::{DfsPostOrder, EdgeRef, NodeIndexable},
};
use rhdl_trace_type::TraceType;

use crate::{
    RHDLError,
    circuit::schematic::{
        Port, PortId, Schematic, SchematicKind, SchematicSet, error::SchematicICE,
    },
    error::rhdl_error,
};

fn map_schematic_to_graph(
    schematic: &Schematic,
    graph: &mut Graph<Port, ()>,
    map: &mut HashMap<PortId, NodeIndex>,
) {
    for input in schematic.inputs.iter().flatten() {
        let node_index = graph.add_node(input.clone());
        map.insert(input.id, node_index);
    }
    for output in &schematic.outputs {
        let node_index = graph.add_node(output.clone());
        map.insert(output.id, node_index);
    }
    for child in &schematic.inner {
        map_schematic_to_graph(child, graph, map);
    }
}

fn link_up_graph(
    schematic: &Schematic,
    graph: &mut Graph<Port, ()>,
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

fn dump_schematic(schematic: &SchematicSet) -> String {
    let fname = tempfile::Builder::new()
        .prefix("schematic-")
        .suffix(".json")
        .tempfile()
        .unwrap();
    let (_, path) = fname.keep().unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(schematic).unwrap()).unwrap();
    path.to_string_lossy().to_string()
}

pub fn loop_check(schematic: &Schematic) -> Result<(), RHDLError> {
    let mut graph = Graph::<Port, ()>::default();
    let mut map = HashMap::<PortId, NodeIndex>::default();
    map_schematic_to_graph(schematic, &mut graph, &mut map);
    link_up_graph(schematic, &mut graph, &map);
    if let Err(cycle) = petgraph::algo::toposort(&graph, None) {
        let cycle_node = cycle.node_id();
        let components = petgraph::algo::kosaraju_scc(&graph);
        for component in components {
            if component.contains(&cycle_node) {
                let set = SchematicSet {
                    schematics: schematic.clone(),
                    loop_ports: component
                        .iter()
                        .map(|node_index| graph[*node_index].id)
                        .collect(),
                };
                let filename = dump_schematic(&set);
                return Err(rhdl_error(SchematicICE::LogicLoop { filename }));
            }
        }
    }
    Ok(())
}

pub fn check_combinatorial_pathways(schematic: &Schematic) -> Result<(), RHDLError> {
    let mut graph = DiGraph::<Port, ()>::default();
    let mut map = HashMap::<PortId, NodeIndex>::default();
    map_schematic_to_graph(schematic, &mut graph, &mut map);
    link_up_graph(schematic, &mut graph, &map);
    let mut visitor = DfsPostOrder::empty(&graph);
    let output_set: HashSet<PortId> = schematic.outputs.iter().map(|output| output.id).collect();
    for input_port in schematic.inputs.iter().flatten() {
        if input_port.ty == TraceType::Reset {
            continue;
        }
        let input_node = map[&input_port.id];
        visitor.move_to(input_node);
        let mut path = vec![input_port.id];
        while let Some(node) = visitor.next(&graph) {
            let port = &graph[node];
            path.push(port.id);
            if output_set.contains(&port.id) {
                let set = SchematicSet {
                    schematics: schematic.clone(),
                    loop_ports: path.clone(),
                };
                let filename = dump_schematic(&set);
                return Err(rhdl_error(SchematicICE::CombinatorialPathway {
                    filename,
                    from: input_port.id,
                    to: port.id,
                }));
            }
        }
    }
    Ok(())
}
