use std::collections::HashMap;

use miette::{Diagnostic, SourceSpan};
use petgraph::visit::EdgeRef;
use thiserror::Error;

use crate::{
    flow_graph::{
        component::{Component, ComponentKind},
        flow_graph_impl::{FlowGraph, FlowIx},
    },
    SourcePool,
};

use super::edge_kind::EdgeKind;

#[derive(Debug)]
pub struct FlowCost {
    pub node_cost: HashMap<FlowIx, f64>,
}

#[derive(Debug, Error)]
#[error("RHDL Critical Timing Path")]
pub struct CriticalPath {
    pub src: SourcePool,
    pub elements: Vec<SourceSpan>,
}

impl Diagnostic for CriticalPath {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Advice)
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("This is the critical path in the design based on the given cost function.  Consider breaking it into smaller components or adding pipeline stages."))
    }
    fn labels<'a>(
        &'a self,
    ) -> Option<Box<dyn std::iter::Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(self.elements.iter().map(|span| {
            miette::LabeledSpan::new_primary_with_span(None, *span)
        })))
    }
}

impl FlowCost {
    pub fn max(&self) -> f64 {
        self.node_cost.values().cloned().fold(0.0, f64::max)
    }
    fn critical_nodes(&self) -> Vec<FlowIx> {
        let max_cost = self.max();
        self.node_cost
            .iter()
            .filter_map(|(ix, cost)| if *cost == max_cost { Some(*ix) } else { None })
            .collect()
    }
    fn backtrack(&self, graph: &FlowGraph, start: FlowIx) -> Vec<FlowIx> {
        let mut result = vec![start];
        let mut current = start;
        while let Some(edge) = graph
            .graph
            .edges_directed(current, petgraph::Direction::Incoming)
            .filter(|edge| !matches!(edge.weight(), EdgeKind::Clock | EdgeKind::Reset))
            .max_by_key(|edge| {
                let source = edge.source();
                let cost = self.node_cost[&source];
                (cost * 1e6) as i64
            })
        {
            let source = edge.source();
            result.push(source);
            current = source;
        }
        result
    }
    fn timing_report(&self, graph: &FlowGraph, start: FlowIx) -> CriticalPath {
        let path = self.backtrack(graph, start);
        let elements = path
            .iter()
            .filter_map(|ix| graph.graph[*ix].location)
            .map(|x| graph.code.span(x).into())
            .collect();
        CriticalPath {
            src: graph.code.source(),
            elements,
        }
    }
    pub fn timing_reports(&self, graph: &FlowGraph) -> Vec<miette::Report> {
        self.critical_nodes()
            .iter()
            .map(|ix| self.timing_report(graph, *ix))
            .map(miette::Report::new)
            .collect()
    }
}

// A trivial cost model that just assumes all non-trivial operations have equal cost.
// We use a small cost for trivial operations to break ties.
pub fn trivial_cost(component: &Component) -> f64 {
    match &component.kind {
        ComponentKind::Binary(_)
        | ComponentKind::Case(_)
        | ComponentKind::DynamicIndex(_)
        | ComponentKind::DynamicSplice(_)
        | ComponentKind::Select
        | ComponentKind::Unary(_) => 1.0,
        _ => 1.0 / 1024.0,
    }
}

pub(crate) fn compute_flow_cost<F: Fn(&Component) -> f64>(fg: &FlowGraph, cost: F) -> FlowCost {
    // Visit the graph in topological order - we will start at the timing source
    let mut cost_map = HashMap::new();
    {
        let mut topo = petgraph::visit::Topo::new(&fg.graph);
        while let Some(ix) = topo.next(&fg.graph) {
            // Get the cost for this node
            let component = &fg.graph[ix];
            let max_incoming = fg
                .graph
                .edges_directed(ix, petgraph::Direction::Incoming)
                .map(|edge| cost_map[&edge.source()])
                .fold(0.0, f64::max);
            let node_cost = cost(component) + max_incoming;
            cost_map.insert(ix, node_cost);
        }
    }
    FlowCost {
        node_cost: cost_map,
    }
}
