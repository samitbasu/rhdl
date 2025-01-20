use crate::rhdl_core::{
    flow_graph::component::{ComponentKind, Unary},
    hdl::ast::SignedWidth,
    rtl::spec::AluUnary,
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerAnyWithSingleArgument {}

impl Pass for LowerAnyWithSingleArgument {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let candidates = graph
            .node_indices()
            .filter(|node| {
                matches!(
                    graph[*node].kind,
                    ComponentKind::Unary(Unary {
                        op: AluUnary::Any,
                        arg_len: SignedWidth::Unsigned(1),
                    })
                )
            })
            .collect::<Vec<_>>();
        for node in candidates {
            graph.node_weight_mut(node).unwrap().kind =
                ComponentKind::Buffer(format!("any_tmp_{node:?}"));
        }
        Ok(FlowGraph { graph, ..input })
    }
}
