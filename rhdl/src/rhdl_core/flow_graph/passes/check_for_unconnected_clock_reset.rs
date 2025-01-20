use petgraph::visit::EdgeRef;

use crate::rhdl_core::{
    ast::source::source_location::SourceLocation, flow_graph::flow_graph_impl::FlowIx, FlowGraph,
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct CheckForUnconnectedClockReset {}

fn _walk_incoming(graph: &FlowGraph, node: FlowIx, locations: &mut Vec<SourceLocation>) {
    if let Some(loc) = graph.graph[node].location {
        locations.push(loc);
    }
    for edge in graph
        .graph
        .edges_directed(node, petgraph::Direction::Incoming)
    {
        let source = edge.source();
        _walk_incoming(graph, source, locations);
    }
}

impl Pass for CheckForUnconnectedClockReset {
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        /*         let need_connection = input.graph.node_indices().filter(|node| {
                   let component = input.graph.node_weight(*node).unwrap();
                   matches!(
                       component.kind,
                       ComponentKind::Sink(Sink::Clock) | ComponentKind::Sink(Sink::Reset)
                   )
               });
               if input.inputs.len() != 1
                   || input.inputs[0].len() != 1
                   || !matches!(
                       input.graph[input.inputs[0][0]].kind,
                       ComponentKind::TimingStart,
                   )
               {
                   return Err(Self::raise_ice(
                       &input,
                       FlowGraphICE::UnSealedFlowGraph,
                       None,
                   ));
               }
               let timing_start = input.inputs[0][0];
               for node in need_connection {
                   if !petgraph::algo::has_path_connecting(&input.graph, timing_start, node, None) {
                       // Collect the chain of nodes upstream from the node
                       let mut spans = vec![];
                       walk_incoming(&input, node, &mut spans);
                       return Err(rhdl_error(FlowGraphError {
                           cause: FlowGraphICE::UnconnectedClockReset,
                           src: input.code.source(),
                           elements: spans
                               .into_iter()
                               .map(|span| input.code.span(span).into())
                               .collect(),
                       }));
                   }
               }
        */
        Ok(input)
    }
}
