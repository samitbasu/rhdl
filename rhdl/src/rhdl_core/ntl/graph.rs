use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::rhdl_core::{
    common::symtab::RegisterId,
    ntl::{
        Object,
        spec::{OpCode, WireKind},
        visit::visit_wires,
    },
};

#[derive(Debug)]
/// A graph representation of the netlist,
/// in which each node represents the source of
/// a register value, and each edge a dependency.
pub struct NetGraph {
    pub reg_map: HashMap<RegisterId<WireKind>, WriteSource>,
    pub graph: petgraph::graph::DiGraph<WriteSource, ()>,
    pub input_node: NodeIndex,
    pub op_nodes: Vec<NodeIndex>,
}

#[derive(Debug, Clone, Copy)]
pub enum WriteSource {
    Input,
    ClockReset,
    OpCode(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum GraphMode {
    Synchronous,
    Asynchronous,
}

fn make_reg_map(input: &Object, mode: GraphMode) -> HashMap<RegisterId<WireKind>, WriteSource> {
    let mut reg_map: HashMap<RegisterId<WireKind>, WriteSource> = HashMap::default();
    // Pass 1
    for (ndx, lop) in input.ops.iter().enumerate() {
        visit_wires(&lop.op, |sense, operand| {
            if let Some(reg) = operand.reg() {
                if sense.is_write() {
                    reg_map.insert(reg, WriteSource::OpCode(ndx));
                }
            }
        });
    }
    match mode {
        GraphMode::Asynchronous => {
            reg_map.extend(
                input
                    .inputs
                    .iter()
                    .flatten()
                    .map(|r| (*r, WriteSource::Input)),
            );
        }
        GraphMode::Synchronous => {
            reg_map.extend(
                input.inputs[0]
                    .iter()
                    .map(|r| (*r, WriteSource::ClockReset)),
            );
            reg_map.extend(
                input
                    .inputs
                    .iter()
                    .skip(1)
                    .flatten()
                    .map(|r| (*r, WriteSource::Input)),
            );
        }
    }
    reg_map
}

pub fn make_net_graph(input: &Object, mode: GraphMode) -> NetGraph {
    // Pass 1 - make a map from register to the source of where it is
    // written.
    let reg_map = make_reg_map(input, mode);
    // Pass 2 - make a graph of the write sources.
    let mut graph = petgraph::graph::DiGraph::default();
    // Add a node for the input source
    let input_node = graph.add_node(WriteSource::Input);
    // Add a node for each opcode.
    let op_nodes = (0..input.ops.len())
        .map(|ndx| graph.add_node(WriteSource::OpCode(ndx)))
        .collect::<Vec<_>>();
    // For each opcode, scan the inputs.  For each input,
    // add an edge to the graph from that input's write source to
    // the current opcode
    for (ndx, lop) in input.ops.iter().enumerate() {
        if matches!(lop.op, OpCode::BlackBox(_)) {
            continue;
        }
        let target = op_nodes[ndx];
        visit_wires(&lop.op, |sense, operand| {
            if let Some(reg) = operand.reg() {
                if sense.is_read() {
                    if let Some(source) = match reg_map[&reg] {
                        WriteSource::Input => Some(input_node),
                        WriteSource::OpCode(ndx) => Some(op_nodes[ndx]),
                        WriteSource::ClockReset => {
                            // For the clock and reset, we don't bother adding edges.
                            None
                        }
                    } {
                        graph.add_edge(source, target, ());
                    }
                }
            }
        });
    }

    NetGraph {
        reg_map,
        graph,
        input_node,
        op_nodes,
    }
}
