use std::collections::{HashMap, HashSet};

use petgraph::{graph::EdgeIndex, visit::EdgeRef, Direction::Incoming};

use crate::rhdl_core::{
    bitx::BitX,
    flow_graph::{
        component::{
            Binary, BitSelect, Case, CaseEntry, ComponentKind, DynamicIndex, DynamicSplice, Unary,
        },
        edge_kind::EdgeKind,
        flow_graph_impl::{FlowIx, GraphType},
    },
    hdl::ast::{unsigned_width, SignedWidth},
    rtl::runtime_ops::{binary, unary},
    types::bit_string::BitString,
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
    let arg_values: Vec<BitX> = arg_bits
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
    Some(result.into())
}

fn compute_unary(uny: &Unary, node: FlowIx, graph: &GraphType) -> Option<BitString> {
    let arg0: TypedBits = collect_argument(graph, node, uny.arg_len, |x| arg_fun(0, x))?.into();
    let result = unary(uny.op, arg0.clone()).ok()?;
    Some(result.into())
}

fn compute_dynamic_index(
    dyn_index: &DynamicIndex,
    lhs_width: usize,
    node: FlowIx,
    graph: &GraphType,
) -> Option<BitString> {
    let arg0: BitString = collect_argument(graph, node, unsigned_width(dyn_index.arg_len), |x| {
        arg_fun(0, x)
    })?;
    let offset: TypedBits = collect_argument(
        graph,
        node,
        unsigned_width(dyn_index.offset_len),
        |x| match x {
            EdgeKind::DynamicOffset(ndx) => Some(*ndx),
            _ => None,
        },
    )?
    .into();
    let offset = offset.as_i64().ok()? as usize;
    let result = arg0
        .bits()
        .iter()
        .skip(offset)
        .take(lhs_width)
        .copied()
        .collect::<Vec<_>>();
    Some(BitString::Unsigned(result))
}

fn compute_dynamic_splice(
    dyn_splice: &DynamicSplice,
    lhs_width: usize,
    node: FlowIx,
    graph: &GraphType,
) -> Option<BitString> {
    let arg: BitString =
        collect_argument(graph, node, unsigned_width(lhs_width), |x| arg_fun(0, x))?;
    let offset: TypedBits = collect_argument(
        graph,
        node,
        unsigned_width(dyn_splice.offset_len),
        |x| match x {
            EdgeKind::DynamicOffset(ndx) => Some(*ndx),
            _ => None,
        },
    )?
    .into();
    let offset = offset.as_i64().ok()? as usize;
    let value = collect_argument(
        graph,
        node,
        unsigned_width(dyn_splice.splice_len),
        |x| match x {
            EdgeKind::Splice(ndx) => Some(*ndx),
            _ => None,
        },
    )?;
    let spliced_value = arg
        .bits()
        .iter()
        .take(offset)
        .chain(value.bits())
        .chain(arg.bits().iter().skip(offset + dyn_splice.splice_len))
        .copied()
        .collect::<Vec<_>>();
    Some(BitString::Unsigned(spliced_value))
}

fn compute_bitselect(bitselect: &BitSelect, node: FlowIx, graph: &GraphType) -> Option<BitString> {
    let arg_incoming = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .filter(|x| matches!(x.weight(), EdgeKind::ArgBit(0, _)))
        .map(|x| x.source())
        .collect::<Vec<_>>();
    if arg_incoming.len() != 1 {
        return None;
    }
    let arg = &graph[arg_incoming[0]];
    let ComponentKind::BitString(arg_value) = &arg.kind else {
        return None;
    };
    if arg_value.len() <= bitselect.bit_index {
        return None;
    }
    let value = arg_value.bits()[bitselect.bit_index];
    Some(BitString::Unsigned(vec![value]))
}

fn compute_case(case: &Case, node: FlowIx, graph: &GraphType) -> Option<BitString> {
    let discriminant = collect_argument(graph, node, case.discriminant_width, |x| match x {
        EdgeKind::Selector(ndx) => Some(*ndx),
        _ => None,
    })?;
    let entry_ndx = case.entries.iter().position(|entry| match entry {
        CaseEntry::Literal(value) => discriminant == *value,
        CaseEntry::WildCard => true,
    })?;
    let input = collect_argument(graph, node, unsigned_width(1), |x| match x {
        EdgeKind::ArgBit(ndx, _) if *ndx == entry_ndx => Some(0),
        _ => None,
    })?;
    Some(input)
}

fn compute_constant_equivalent(node: FlowIx, graph: &GraphType) -> Option<BitString> {
    let component = &graph[node];
    match &component.kind {
        ComponentKind::Binary(binary) => compute_binary(binary, node, graph),
        ComponentKind::BitSelect(bitselect) => compute_bitselect(bitselect, node, graph),
        ComponentKind::Unary(unary) => compute_unary(unary, node, graph),
        ComponentKind::DynamicIndex(dyn_index) => {
            compute_dynamic_index(dyn_index, component.width, node, graph)
        }
        ComponentKind::Case(case) => compute_case(case, node, graph),
        ComponentKind::DynamicSplice(dyn_splice) => {
            compute_dynamic_splice(dyn_splice, component.width, node, graph)
        }
        _ => None,
    }
}

impl Pass for ConstantPropagationPass {
    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let all_inputs_constant = |node| {
            graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .all(|edge| {
                    matches!(
                        graph[edge.source()].kind,
                        ComponentKind::Constant(_) | ComponentKind::BitString(_)
                    )
                })
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
                    ComponentKind::Constant(value.bits()[0]);
            } else {
                graph.node_weight_mut(node).unwrap().kind = ComponentKind::BitString(value);
            }
        }
        graph.retain_edges(|_, edge| !edges_to_drop.contains(&edge));
        Ok(FlowGraph { graph, ..input })
    }
}
