use std::collections::HashSet;

use petgraph::visit::EdgeRef;

use crate::rhdl_core::{
    bitx::BitX,
    flow_graph::{
        component::{ComponentKind, Unary},
        edge_kind::EdgeKind,
        flow_graph_impl::{FlowIx, GraphType},
    },
    hdl::ast::unsigned_width,
    rtl::spec::AluUnary,
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveZerosFromAnyPass {}

#[derive(Debug)]
enum VarOrConstant {
    Variable,
    Constant(BitX),
}

#[derive(Debug)]
struct Replacement {
    node: FlowIx,
    args: Vec<FlowIx>,
}

fn get_source_bit(node: FlowIx, graph: &GraphType) -> VarOrConstant {
    match graph[node].kind {
        ComponentKind::Constant(value) => VarOrConstant::Constant(value),
        _ => VarOrConstant::Variable,
    }
}

fn rewrite_any_operation(orig_node: FlowIx, graph: &GraphType) -> Option<Replacement> {
    let ComponentKind::Unary(Unary {
        op: AluUnary::Any,
        arg_len: _,
    }) = &graph[orig_node].kind
    else {
        return None;
    };
    // Collect the input arguments
    let arg_incoming = graph
        .edges_directed(orig_node, petgraph::Direction::Incoming)
        .filter_map(|x| match x.weight() {
            EdgeKind::ArgBit(ndx, _) if *ndx == 0 => Some(x.source()),
            _ => None,
        })
        .map(|node| (node, get_source_bit(node, graph)))
        .collect::<Vec<_>>();
    // If any of the arguments are true, we can drop all other arguments
    if let Some((true_node, _)) = arg_incoming
        .iter()
        .find(|(_, voc)| matches!(voc, VarOrConstant::Constant(BitX::One)))
    {
        return Some(Replacement {
            node: orig_node,
            args: vec![*true_node],
        });
    }
    // Remove any arguments that are false
    let new_args = arg_incoming
        .iter()
        .filter_map(|(node, voc)| match voc {
            VarOrConstant::Constant(BitX::Zero) => None,
            _ => Some(*node),
        })
        .collect::<Vec<_>>();
    // if we made no improvement, return None
    if new_args.len() == arg_incoming.len() {
        return None;
    }
    Some(Replacement {
        node: orig_node,
        args: new_args,
    })
}

impl Pass for RemoveZerosFromAnyPass {
    fn description() -> &'static str {
        "Remove zeros from .any() inputs"
    }
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let candidates = graph
            .node_indices()
            .filter_map(|node| rewrite_any_operation(node, &graph))
            .collect::<Vec<_>>();
        // First remove all of the edges to any node being replaced.
        let edges_to_drop: HashSet<_> = candidates
            .iter()
            .flat_map(|candidate| {
                graph
                    .edges_directed(candidate.node, petgraph::Direction::Incoming)
                    .map(|edge| edge.id())
            })
            .collect();
        graph.retain_edges(|_, edge| !edges_to_drop.contains(&edge));
        // Now rewire the nodes to the new arguments
        for candidate in candidates {
            graph.node_weight_mut(candidate.node).unwrap().kind = ComponentKind::Unary(Unary {
                op: AluUnary::Any,
                arg_len: unsigned_width(candidate.args.len()),
            });
            for (bit, arg) in candidate.args.iter().enumerate() {
                graph.add_edge(*arg, candidate.node, EdgeKind::ArgBit(0, bit));
            }
        }
        Ok(FlowGraph { graph, ..input })
    }
}
