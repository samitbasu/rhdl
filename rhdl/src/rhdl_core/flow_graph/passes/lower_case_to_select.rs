use petgraph::{graph::EdgeIndex, visit::EdgeRef};

use crate::rhdl_core::{
    flow_graph::{
        component::{CaseEntry, ComponentKind},
        edge_kind::EdgeKind,
        flow_graph_impl::{FlowIx, GraphType},
    },
    FlowGraph, RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerCaseToSelectPass {}

struct Replacement {
    original: FlowIx,
    true_value: EdgeIndex,
    false_value: EdgeIndex,
}

fn is_select_like_case(node: FlowIx, graph: &GraphType) -> Option<Replacement> {
    let ComponentKind::Case(case) = &graph[node].kind else {
        return None;
    };
    if case.entries.len() != 2 {
        return None;
    }
    // We can get a select from a case in one of 2 ways.
    // The first is if the discriminant is a bit.  In that
    // case, we only need to find the true path and the false path.
    if case.discriminant_width.len() != 1 {
        return None;
    }
    let arg0 = &case.entries[0];
    let arg1 = &case.entries[1];
    let arg0_path = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .find(|edge| matches!(edge.weight(), EdgeKind::ArgBit(0, 0)))?;
    let arg1_path = graph
        .edges_directed(node, petgraph::Direction::Incoming)
        .find(|edge| matches!(edge.weight(), EdgeKind::ArgBit(1, 0)))?;
    match (arg0, arg1) {
        (CaseEntry::Literal(l0), CaseEntry::Literal(l1)) => {
            if l0.is_ones() && l1.is_zero() {
                Some(Replacement {
                    original: node,
                    true_value: arg0_path.id(),
                    false_value: arg1_path.id(),
                })
            } else if l0.is_zero() && l1.is_ones() {
                Some(Replacement {
                    original: node,
                    true_value: arg1_path.id(),
                    false_value: arg0_path.id(),
                })
            } else {
                None
            }
        }
        (CaseEntry::Literal(l0), CaseEntry::WildCard) => {
            if l0.is_ones() {
                Some(Replacement {
                    original: node,
                    true_value: arg0_path.id(),
                    false_value: arg1_path.id(),
                })
            } else if l0.is_zero() {
                Some(Replacement {
                    original: node,
                    true_value: arg1_path.id(),
                    false_value: arg0_path.id(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

impl Pass for LowerCaseToSelectPass {
    fn description() -> &'static str {
        "Lower case with 2 arms to a select"
    }

    fn run(mut input: FlowGraph) -> Result<FlowGraph, RHDLError> {
        let mut graph = std::mem::take(&mut input.graph);
        let candidates = graph
            .node_indices()
            .filter_map(|node| is_select_like_case(node, &graph))
            .collect::<Vec<_>>();
        for replacement in candidates {
            let node = graph.node_weight_mut(replacement.original).unwrap();
            node.kind = ComponentKind::Select;
            *graph.edge_weight_mut(replacement.true_value).unwrap() = EdgeKind::True;
            *graph.edge_weight_mut(replacement.false_value).unwrap() = EdgeKind::False;
        }
        Ok(FlowGraph { graph, ..input })
    }
}
