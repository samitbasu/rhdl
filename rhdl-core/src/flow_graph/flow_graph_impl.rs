use std::collections::HashMap;
use std::io::Write;

use petgraph::prelude::StableDiGraph;

use crate::{
    ast::{source_location::SourceLocation, spanned_source::SpannedSourceSet},
    hdl::ast::Module,
    rtl::object::RegisterKind,
    RHDLError,
};

use super::{
    component::{BitSelect, Component, ComponentKind, DFFInput, DFFOutput},
    edge_kind::EdgeKind,
    hdl::generate_hdl,
};

pub type FlowIx = petgraph::graph::NodeIndex;
pub type GraphType = StableDiGraph<Component, EdgeKind>;

#[derive(Debug, Clone, Default)]
pub struct DFF {
    pub input: FlowIx,
    pub output: FlowIx,
    pub reset_value: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FlowGraph {
    pub graph: GraphType,
    pub inputs: Vec<Vec<FlowIx>>,
    pub output: Vec<FlowIx>,
    pub code: SpannedSourceSet,
    pub dffs: Vec<DFF>,
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
                    width: 1,
                    location,
                })
            })
            .collect()
    }
    pub fn input(&mut self, kind: RegisterKind, argument_index: usize, name: &str) -> Vec<FlowIx> {
        (0..kind.len())
            .map(|bit_index| {
                self.graph.add_node(Component {
                    kind: ComponentKind::Buffer(format!(
                        "[{}]<-in<{}, {}>",
                        name, argument_index, bit_index
                    )),
                    width: 1,
                    location: None,
                })
            })
            .collect()
    }
    pub fn output(&mut self, kind: RegisterKind, name: &str) -> Vec<FlowIx> {
        (0..kind.len())
            .map(|bit_index| {
                self.graph.add_node(Component {
                    kind: ComponentKind::Buffer(format!("[{}]->out<{}>", name, bit_index)),
                    width: 1,
                    location: None,
                })
            })
            .collect()
    }
    pub fn dff(
        &mut self,
        kind: RegisterKind,
        init: &[bool],
        location: Option<SourceLocation>,
    ) -> (Vec<FlowIx>, Vec<FlowIx>) {
        let dff_input = (0..kind.len())
            .map(|bit_index| {
                self.graph.add_node(Component {
                    kind: ComponentKind::DFFInput(DFFInput { bit_index }),
                    width: 1,
                    location,
                })
            })
            .collect::<Vec<_>>();
        let dff_output = (0..kind.len())
            .map(|bit_index| {
                self.graph.add_node(Component {
                    kind: ComponentKind::DFFOutput(DFFOutput { bit_index }),
                    width: 1,
                    location,
                })
            })
            .collect::<Vec<_>>();
        self.dffs.extend(
            dff_input
                .iter()
                .zip(dff_output.iter())
                .zip(init.iter())
                .map(|((input, output), reset)| DFF {
                    input: *input,
                    output: *output,
                    reset_value: *reset,
                }),
        );
        (dff_input, dff_output)
    }
    pub fn new_component(
        &mut self,
        kind: ComponentKind,
        width: usize,
        location: SourceLocation,
    ) -> FlowIx {
        self.graph.add_node(Component {
            kind,
            width,
            location: Some(location),
        })
    }
    pub fn new_component_with_optional_location(
        &mut self,
        kind: ComponentKind,
        width: usize,
        location: Option<SourceLocation>,
    ) -> FlowIx {
        self.graph.add_node(Component {
            kind,
            width,
            location,
        })
    }
    pub fn bit_select(&mut self, source: FlowIx, bit_index: usize) -> FlowIx {
        let node = self.graph.add_node(Component {
            kind: ComponentKind::BitSelect(BitSelect { bit_index }),
            width: 1,
            location: None,
        });
        self.graph.add_edge(source, node, EdgeKind::ArgBit(0, 0));
        node
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
            self.edge(source, target, EdgeKind::ArgBit(0, 0));
        });
    }
    pub fn merge(&mut self, other: &FlowGraph) -> HashMap<FlowIx, FlowIx> {
        let remap: HashMap<FlowIx, FlowIx> = other
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
            self.graph.add_edge(remap[&src], remap[&dest], kind);
        }
        self.dffs.extend(other.dffs.iter().map(|dff| DFF {
            input: remap[&dff.input],
            output: remap[&dff.output],
            reset_value: dff.reset_value,
        }));
        self.code.extend(other.code.sources.clone());
        remap
    }
    // Create a new, top level FG with sources for the inputs and sinks for the
    // outputs.
    pub fn sealed(self) -> FlowGraph {
        if self.graph.node_weights().any(|node| {
            matches!(
                node.kind,
                ComponentKind::TimingStart | ComponentKind::TimingEnd
            )
        }) {
            return self;
        }
        let mut fg = FlowGraph::default();
        let remap = fg.merge(&self);
        let timing_start =
            fg.new_component_with_optional_location(ComponentKind::TimingStart, 0, None);
        let timing_end = fg.new_component_with_optional_location(ComponentKind::TimingEnd, 0, None);
        // Create sources for all of the inputs of the internal flow graph
        self.inputs.iter().flatten().for_each(|input| {
            fg.edge(timing_start, remap[input], EdgeKind::Virtual);
        });
        self.output.iter().for_each(|output| {
            fg.edge(remap[output], timing_end, EdgeKind::Virtual);
        });
        let sources = fg
            .graph
            .node_indices()
            .filter(|node| matches!(fg.graph[*node].kind, ComponentKind::DFFOutput(_)))
            .collect::<Vec<_>>();
        let sinks = fg
            .graph
            .node_indices()
            .filter(|node| matches!(fg.graph[*node].kind, ComponentKind::DFFInput(_)))
            .collect::<Vec<_>>();
        sources.iter().for_each(|node| {
            fg.edge(timing_start, *node, EdgeKind::Virtual);
        });
        sinks.iter().for_each(|node| {
            fg.edge(*node, timing_end, EdgeKind::Virtual);
        });
        fg.inputs = vec![vec![timing_start]];
        fg.output = vec![timing_end];
        fg
    }
    pub fn hdl(&self, name: &str) -> Result<Module, RHDLError> {
        generate_hdl(name, self)
    }
    pub fn dot(&self) -> Result<Vec<u8>, RHDLError> {
        let mut s = vec![];
        super::dot::write_dot(self, &mut s)?;
        Ok(s)
    }
}
