use std::collections::{HashMap, HashSet};

use petgraph::{graph::EdgeIndex, visit::EdgeRef, Direction::Incoming};

use crate::{
    flow_graph::{
        component::{Binary, ComponentKind, Unary},
        edge_kind::EdgeKind,
        flow_graph_impl::{FlowIx, GraphType},
    },
    hdl::ast::SignedWidth,
    rhif::runtime_ops::{binary, unary},
    types::bit_string::BitString,
    util::binary_string,
    FlowGraph, RHDLError, TypedBits,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ConstantPropagationPass {}

fn collect_argument<T: Fn(&EdgeKind) -> Option<usize>>(
    graph: &GraphType,
    node: FlowIx,
    width: SignedWidth,
    filter: T,
) -> Option<BitString> {
    let arg_incoming = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .filter_map(|x| filter(x.weight()).map(|ndx| (ndx, x.source())))
        .collect::<Vec<_>>();
    let arg_bits = (0..width.len())
        .map(|bit| {
            arg_incoming
                .iter()
                .find_map(|(b, ndx)| if *b == bit { Some(*ndx) } else { None })
        })
        .collect::<Option<Vec<_>>>()?;
    let arg_values: Vec<bool> = arg_bits
        .iter()
        .flat_map(|ndx| {
            if let ComponentKind::Constant(value) = &graph[*ndx].kind {
                Some(*value)
            } else {
                None
            }
        })
        .collect();
    Some(if width.is_signed() {
        BitString::Signed(arg_values)
    } else {
        BitString::Unsigned(arg_values)
    })
}

fn arg_fun(index: usize, edge: &EdgeKind) -> Option<usize> {
    match edge {
        EdgeKind::ArgBit(ndx, bit) if *ndx == index => Some(*bit),
        _ => None,
    }
}

fn compute_binary(bin: &Binary, node: FlowIx, graph: &GraphType) -> Option<BitString> {
    let arg0: TypedBits = collect_argument(graph, node, bin.left_len, |x| arg_fun(0, x))?.into();
    let arg1: TypedBits = collect_argument(graph, node, bin.right_len, |x| arg_fun(1, x))?.into();
    let result = binary(bin.op, arg0.clone(), arg1.clone()).ok()?;
    eprintln!(
        "Constant prop {arg0:?} {op:?} {arg1:?} -> {result:?}",
        op = bin.op
    );
    Some(result.into())
}

fn compute_unary(uny: &Unary, node: FlowIx, graph: &GraphType) -> Option<BitString> {
    let arg0: TypedBits = collect_argument(graph, node, uny.arg_len, |x| arg_fun(0, x))?.into();
    let result = unary(uny.op, arg0).ok()?;
    Some(result.into())
}

fn compute_constant_equivalent(node: FlowIx, graph: &GraphType) -> Option<BitString> {
    match &graph[node].kind {
        ComponentKind::Binary(binary) => compute_binary(binary, node, graph),
        ComponentKind::Unary(unary) => compute_unary(unary, node, graph),
        _ => None,
    }
}

impl Pass for ConstantPropagationPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let all_inputs_constant = |node| {
            graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .all(|edge| matches!(graph[edge.source()].kind, ComponentKind::Constant(_)))
        };
        let candidates = graph
            .node_indices()
            .filter(|node| all_inputs_constant(*node))
            .collect::<Vec<_>>();
        let values: HashMap<FlowIx, BitString> = candidates
            .iter()
            .filter_map(|node| {
                let value = compute_constant_equivalent(*node, &graph)?;
                Some((*node, value))
            })
            .collect();
        // Collect the edges that lead to the nodes that will be rewritten
        let edges_to_drop = values
            .keys()
            .flat_map(|node| graph.edges_directed(*node, Incoming).map(|edge| edge.id()))
            .collect::<HashSet<EdgeIndex>>();
        // Rewrite the nodes to replace them with BitString components
        // containing the constant value.
        for (node, value) in values {
            if value.len() == 1 {
                graph.node_weight_mut(node).unwrap().kind =
                    ComponentKind::Constant(value.is_all_true());
            } else {
                graph.node_weight_mut(node).unwrap().kind = ComponentKind::BitString(value);
            }
        }
        graph.retain_edges(|_, edge| !edges_to_drop.contains(&edge));
        Ok(FlowGraph { graph, ..input })
    }
}
