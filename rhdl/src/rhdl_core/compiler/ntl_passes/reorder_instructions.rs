use crate::{
    prelude::RHDLError,
    rhdl_core::{
        ast::source::source_location::SourceLocation,
        error::rhdl_error,
        ntl::{
            error::{NetListError, NetListICE},
            graph::{make_net_graph, GraphMode, WriteSource},
            Object,
        },
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ReorderInstructions {}

fn raise_cycle_error(input: &Object, location: Vec<SourceLocation>) -> RHDLError {
    rhdl_error(NetListError {
        cause: NetListICE::LogicLoop,
        src: input.code.source(),
        elements: location
            .iter()
            .map(|&loc| input.code.span(loc).into())
            .collect(),
    })
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
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let dep = make_net_graph(&input, GraphMode::Asynchronous);
        match petgraph::algo::toposort(&dep.graph, None) {
            Ok(order) => {
                let orig_order = std::mem::take(&mut input.ops);
                for elt in order {
                    if let WriteSource::OpCode(ndx) = &dep.graph[elt] {
                        input.ops.push(orig_order[*ndx].clone());
                    }
                }
            }
            Err(cycle) => {
                log::warn!("cycle node {:?}", &dep.graph[cycle.node_id()]);
                std::fs::write("reorder.txt", format!("{:?}", input)).unwrap();
                let node = cycle.node_id();
                let source_location = if let Some(path) =
                    petgraph::algo::all_simple_paths::<Vec<_>, _>(&dep.graph, node, node, 1, None)
                        .next()
                {
                    path.into_iter()
                        .flat_map(|id| match &dep.graph[id] {
                            WriteSource::OpCode(ndx) => {
                                log::warn!("{:?}", input.ops[*ndx].op);
                                input.ops[*ndx].loc
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                } else {
                    match &dep.graph[cycle.node_id()] {
                        WriteSource::OpCode(ndx) => {
                            log::warn!("{:?}", input.ops[*ndx].op);
                            input.ops[*ndx].loc
                        }
                        _ => None,
                    }
                    .into_iter()
                    .collect()
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
