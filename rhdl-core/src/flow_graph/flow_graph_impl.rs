use std::collections::HashMap;

use petgraph::{Directed, Graph};

use crate::{
    rhif::object::SourceLocation,
    rtl::object::{BitString, RegisterKind},
};

use super::{
    component::{Component, ComponentKind},
    edge_kind::EdgeKind,
};

pub type FlowIx = petgraph::graph::NodeIndex;

#[derive(Debug, Clone, Default)]
pub struct FlowGraph {
    pub graph: Graph<Component, EdgeKind, Directed>,
    pub inputs: Vec<Option<FlowIx>>,
    pub output: FlowIx,
}

impl FlowGraph {
    pub fn buffer(
        &mut self,
        kind: RegisterKind,
        name: &str,
        location: Option<SourceLocation>,
    ) -> FlowIx {
        self.graph.add_node(Component {
            kind: ComponentKind::Buffer(Buffer {
                kind,
                name: name.into(),
            }),
            location,
        })
    }
    pub fn source(
        &mut self,
        kind: RegisterKind,
        name: &str,
        location: Option<SourceLocation>,
    ) -> FlowIx {
        self.graph.add_node(Component {
            kind: ComponentKind::Source(Buffer {
                kind,
                name: name.into(),
            }),
            location,
        })
    }
    pub fn sink(
        &mut self,
        kind: RegisterKind,
        name: &str,
        location: Option<SourceLocation>,
    ) -> FlowIx {
        self.graph.add_node(Component {
            kind: ComponentKind::Sink(Buffer {
                kind,
                name: name.into(),
            }),
            location,
        })
    }
    pub fn new_component(&mut self, kind: ComponentKind, location: SourceLocation) -> FlowIx {
        self.graph.add_node(Component {
            kind,
            location: Some(location),
        })
    }
    pub fn new_component_with_optional_location(
        &mut self,
        kind: ComponentKind,
        location: Option<SourceLocation>,
    ) -> FlowIx {
        self.graph.add_node(Component { kind, location })
    }
    pub fn lhs(&mut self, component: FlowIx, lhs: FlowIx) {
        self.graph.add_edge(component, lhs, EdgeKind::Arg(0));
    }
    pub fn arg(&mut self, component: FlowIx, arg: FlowIx, index: usize) {
        self.graph.add_edge(arg, component, EdgeKind::Arg(index));
    }
    pub fn offset(&mut self, component: FlowIx, offset: FlowIx) {
        self.graph
            .add_edge(offset, component, EdgeKind::DynamicOffset);
    }
    pub fn edge(&mut self, component: FlowIx, source: FlowIx, kind: EdgeKind) {
        self.graph.add_edge(source, component, kind);
    }
    pub fn case_literal(&mut self, component: FlowIx, case: FlowIx, literal: BitString) {
        self.graph
            .add_edge(case, component, EdgeKind::CaseLiteral(literal));
    }
    pub fn case_wild(&mut self, component: FlowIx, case: FlowIx) {
        self.graph.add_edge(case, component, EdgeKind::CaseWild);
    }
    pub fn merge(&mut self, other: &FlowGraph) -> HashMap<FlowIx, FlowIx> {
        let ret: HashMap<FlowIx, FlowIx> = other
            .graph
            .node_indices()
            .map(|old_node| {
                let new_node = self.graph.add_node(other.graph[old_node].clone());
                (old_node, new_node)
            })
            .collect();
        for edge in other.graph.edge_indices() {
            let kind = other.graph[edge].clone();
            let (src, dest) = other.graph.edge_endpoints(edge).unwrap();
            self.graph.add_edge(ret[&src], ret[&dest], kind);
        }
        ret
    }
}
