use petgraph::{
    algo::{all_simple_paths, feedback_arc_set, is_cyclic_directed},
    visit::EdgeRef,
};

use crate::rhdl_core::{
    error::rhdl_error,
    flow_graph::{
        dot,
        error::{FlowGraphError, FlowGraphICE},
        flow_graph_impl::FlowIx,
    },
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct CheckForLogicLoops {}

fn raise_loop_error(input: &FlowGraph, nodes: &[FlowIx]) -> RHDLError {
    let mut elements = vec![];
    for node in nodes {
        if let Some(location) = input.graph[*node].location {
            elements.push(input.code.span(location).into());
        }
    }
    rhdl_error(FlowGraphError {
        cause: FlowGraphICE::LogicLoop,
        src: input.code.source(),
        elements,
    })
}

impl Pass for CheckForLogicLoops {
    fn description() -> &'static str {
        "Check for logic loops"
    }
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        // Print the loop to a report
        use miette::SourceCode;
        use std::io::Write;
        let mut file = std::fs::File::create("report.txt").unwrap();
        let graph = &input.graph;
        let contains_cycles = is_cyclic_directed(graph);
        if contains_cycles {
            let mut feedback = feedback_arc_set::greedy_feedback_arc_set(graph);
            let Some(edge) = feedback.next() else {
                return Err(raise_loop_error(&input, &[]));
            };
            let mut round_trip =
                all_simple_paths::<Vec<_>, _>(graph, edge.target(), edge.source(), 1, Some(100));
            let Some(first_round_trip) = round_trip.next() else {
                return Err(raise_loop_error(&input, &[edge.source(), edge.target()]));
            };
            for node in &first_round_trip {
                writeln!(file, "Node {:?}", node).unwrap();
                let source = input.code.source();
                if let Some(location) = input.graph[*node].location {
                    let span: miette::SourceSpan = input.code.span(location).into();
                    let text = source.read_span(&span, 0, 0).unwrap();
                    file.write_all(text.data()).unwrap();
                }
            }
            return Err(raise_loop_error(&input, &first_round_trip));
        }
        Ok(input)
    }
}
