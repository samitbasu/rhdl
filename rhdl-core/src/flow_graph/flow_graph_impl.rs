use std::collections::HashMap;

use petgraph::{Directed, Graph};

use crate::{
    ast::{source_location::SourceLocation, spanned_source::SpannedSourceSet},
    rtl::object::RegisterKind,
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
    pub code: SpannedSourceSet,
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
    pub fn zip(
        &mut self,
        source: impl Iterator<Item = FlowIx>,
        target: impl Iterator<Item = FlowIx>,
    ) {
        source.zip(target).for_each(|(source, target)| {
            self.edge(source, target, EdgeKind::Arg(0));
        });
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
        self.code.extend(other.code.sources.clone());
        ret
    }
    // Create a new, top level FG with sources for the inputs and sinks for the
    // outputs.
    pub fn sealed(self) -> FlowGraph {
        let mut fg = FlowGraph::default();
        let remap = fg.merge(&self);
        let timing_start =
            fg.new_component_with_optional_location(ComponentKind::TimingStart, None);
        let timing_end = fg.new_component_with_optional_location(ComponentKind::TimingEnd, None);
        // Create sources for all of the inputs of the internal flow graph
        self.inputs.iter().flatten().for_each(|input| {
            fg.edge(timing_start, remap[input], EdgeKind::Virtual);
        });
        self.output.iter().for_each(|output| {
            fg.edge(remap[output], timing_end, EdgeKind::Virtual);
        });
        // Create links from all of the internal sources to the timing start node
        for node in fg.graph.node_indices() {
            if matches!(fg.graph[node].kind, ComponentKind::Source(_)) {
                fg.edge(timing_start, node, EdgeKind::Virtual);
            }
            if matches!(fg.graph[node].kind, ComponentKind::Sink(_)) {
                fg.edge(node, timing_end, EdgeKind::Virtual);
            }
        }
        fg.inputs = vec![vec![timing_start]];
        fg.output = vec![timing_end];
        fg
    }
}
