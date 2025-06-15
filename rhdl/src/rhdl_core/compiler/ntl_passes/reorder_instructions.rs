use std::collections::HashMap;

use crate::{
    prelude::RHDLError,
    rhdl_core::{
        ast::source::source_location::SourceLocation,
        error::rhdl_error,
        ntl::{
            error::{NetListError, NetListICE},
            remap::{visit_operands, Sense},
            spec::RegisterId,
            Object,
        },
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ReorderInstructions {}

enum WriteSource {
    Input,
    OpCode(usize),
}

fn raise_cycle_error(input: &Object, location: Option<SourceLocation>) -> RHDLError {
    rhdl_error(NetListError {
        cause: NetListICE::LogicLoop,
        src: input.code.source(),
        elements: location
            .map(|loc| input.code.span(loc).into())
            .into_iter()
            .collect(),
    })
}

fn make_reg_map(input: &Object) -> HashMap<RegisterId, WriteSource> {
    let mut reg_map: HashMap<RegisterId, WriteSource> = HashMap::default();
    // Pass 1
    for (ndx, lop) in input.ops.iter().enumerate() {
        visit_operands(&lop.op, |sense, operand| {
            if let Some(reg) = operand.reg() {
                if sense == Sense::Write {
                    reg_map.insert(reg, WriteSource::OpCode(ndx));
                }
            }
        });
    }
    reg_map.extend(
        input
            .inputs
            .iter()
            .flatten()
            .map(|r| (*r, WriteSource::Input)),
    );
    reg_map
}

fn make_dep_graph(
    input: &Object,
    reg_map: &HashMap<RegisterId, WriteSource>,
) -> petgraph::graph::DiGraph<WriteSource, ()> {
    let mut g = petgraph::graph::DiGraph::default();
    // Add a node for the input source
    let input_node = g.add_node(WriteSource::Input);
    // Add a node for each opcode.
    let op_ndx = (0..input.ops.len())
        .map(|ndx| g.add_node(WriteSource::OpCode(ndx)))
        .collect::<Vec<_>>();
    // For each opcode, scan the inputs.  For each input,
    // add an edge to the graph from that input's write source to
    // the current opcode
    for (ndx, lop) in input.ops.iter().enumerate() {
        let target = op_ndx[ndx];
        visit_operands(&lop.op, |sense, operand| {
            if let Some(reg) = operand.reg() {
                if sense == Sense::Read {
                    let source = match reg_map[&reg] {
                        WriteSource::Input => input_node,
                        WriteSource::OpCode(ndx) => op_ndx[ndx],
                    };
                    g.add_edge(source, target, ());
                }
            }
        });
    }
    g
}

impl Pass for ReorderInstructions {
    /// This passes reorders the operands so as to have them in
    /// executable order (write then read).  The function is organized
    /// into 3 passes:
    ///    1. For each register, the source is either an input of the NTL,
    ///       an opcode, or nothing.  Nothing -> error (undriven input).
    ///    2. Build a petgraph of the dependencies.  Add the input as a node
    ///       to the graph, and then each of the opcodes.  For each opcode,
    ///       add an edge to the opcode or input for each of it's inputs.
    ///    3. Perform a topological sort of the dependency graph.  Reorder
    ///       the opcodes based on the topological order.  If cycles exist,
    ///       raise an error.
    fn run(input: Object) -> Result<Object, RHDLError> {
        // Pass 1 - make a map from register to the source of where it is
        // written.
        let reg_map = make_reg_map(&input);
        // Pass 2 - make a graph of the write sources.
        let dep_graph = make_dep_graph(&input, &reg_map);
        // Pass 3 perform a topo sort of the graph
        match petgraph::algo::toposort(&dep_graph, None) {
            Ok(order) => {
                for elt in order {
                    match &dep_graph[elt] {
                        WriteSource::Input => eprintln!("ord: input"),
                        WriteSource::OpCode(x) => eprintln!("ord: {:?}", input.ops[*x].op),
                    }
                }
            }
            Err(cycle) => {
                let source_location = match &dep_graph[cycle.node_id()] {
                    WriteSource::Input => None,
                    WriteSource::OpCode(ndx) => input.ops[*ndx].loc,
                };
                return Err(raise_cycle_error(&input, source_location));
            }
        }
        Ok(input)
    }

    fn description() -> &'static str {
        "Reorder instructions to create legal dataflow"
    }
}
