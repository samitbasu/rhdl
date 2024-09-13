// Every node in the graph should be driven unless:
// 1. It is a timing source
// 2. It is constant

use crate::{
    flow_graph::{component::ComponentKind, error::FlowGraphICE, flow_graph_impl::FlowGraph},
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct CheckForUndrivenPass {}

impl Pass for CheckForUndrivenPass {
    fn run(input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        for node in input.graph.node_indices() {
            let component = input.graph.node_weight(node).unwrap();
            let no_drive_needed = matches!(
                component.kind,
                ComponentKind::TimingStart | ComponentKind::Constant(_)
            );
            if !no_drive_needed {
                let incoming_count = input
                    .graph
                    .edges_directed(node, petgraph::Direction::Incoming)
                    .count();
                if incoming_count == 0 {
                    return Err(Self::raise_ice(
                        &input,
                        FlowGraphICE::UndrivenNode,
                        component.location,
                    ));
                }
            }
        }
        Ok(input)
    }
}
