use petgraph::visit::EdgeRef;

use crate::{flow_graph::component::ComponentKind, FlowGraph, RHDLError};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ConstantBufferEliminationPass {}

impl Pass for ConstantBufferEliminationPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        // If we have a constant node A that drives a buffer node B, we can
        // replace the buffer B with a constant and drop the constant node A.
        graph.retain_nodes(|mut frozen, node| {
            let component = frozen.node_weight(node).unwrap().clone();
            if let ComponentKind::Constant(value) = component.kind {
                let mut must_retain = false;
                let downstreams = frozen
                    .neighbors_directed(node, petgraph::Outgoing)
                    .collect::<Vec<_>>();
                for target_node in downstreams {
                    frozen.node_weight_mut(target_node).map(|c| {
                        if let ComponentKind::Buffer(_) = &c.kind {
                            c.kind = ComponentKind::Constant(value);
                        } else {
                            must_retain = true;
                        }
                    });
                    /*
                    if let Some(ComponentKind::Buffer(_)) = target_component.map(|c| &c.kind) {
                        target_component.unwrap().kind = ComponentKind::Constant(value);
                    } else {
                        must_retain = true;
                    }
                    */
                }
                must_retain
            } else {
                true
            }
        });
        Ok(FlowGraph { graph, ..input })
    }
}
