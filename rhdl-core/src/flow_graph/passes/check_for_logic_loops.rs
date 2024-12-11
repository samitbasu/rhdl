use petgraph::{
    algo::{feedback_arc_set, is_cyclic_directed},
    visit::EdgeRef,
};

use crate::{
    error::rhdl_error,
    flow_graph::error::{FlowGraphError, FlowGraphICE},
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct CheckForLogicLoops {}

impl Pass for CheckForLogicLoops {
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let graph = &input.graph;
        let contains_cycles = is_cyclic_directed(graph);
        if contains_cycles {
            let feedback = feedback_arc_set::greedy_feedback_arc_set(graph);

            let mut elements = vec![];
            for edge in feedback {
                if let Some(location) = input.graph[edge.source()].location {
                    elements.push(input.code.span(location).into());
                }
                if let Some(location) = input.graph[edge.target()].location {
                    elements.push(input.code.span(location).into());
                }
            }
            return Err(rhdl_error(FlowGraphError {
                cause: FlowGraphICE::LogicLoop,
                src: input.code.source(),
                elements,
            }));
        }
        Ok(input)
    }
}
