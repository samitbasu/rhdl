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
    pub inputs: Vec<Vec<FlowIx>>,
    pub output: Vec<FlowIx>,
}

impl FlowGraph {
    pub fn buffer(
        &mut self,
        kind: RegisterKind,
        name: &str,
        location: Option<SourceLocation>,
    ) -> Vec<FlowIx> {
        (0..kind.len())
            .map(|bit| {
                let name = format!("{}[{}]", name, bit);
                self.graph.add_node(Component {
                    kind: ComponentKind::Buffer(name),
                    location,
                })
            })
            .collect()
    }
    pub fn source(
        &mut self,
        kind: RegisterKind,
        name: &str,
        location: Option<SourceLocation>,
    ) -> Vec<FlowIx> {
        (0..kind.len())
            .map(|bit| {
                let name = format!("{}[{}]", name, bit);
                self.graph.add_node(Component {
                    kind: ComponentKind::Source(name),
                    location,
                })
            })
            .collect()
    }
    pub fn sink(
        &mut self,
        kind: RegisterKind,
        name: &str,
        location: Option<SourceLocation>,
    ) -> Vec<FlowIx> {
        (0..kind.len())
            .map(|bit| {
                let name = format!("{}[{}]", name, bit);
                self.graph.add_node(Component {
                    kind: ComponentKind::Sink(name),
                    location,
                })
            })
            .collect()
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
    pub fn edge(&mut self, source: FlowIx, target: FlowIx, kind: EdgeKind) {
        self.graph.add_edge(source, target, kind);
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
