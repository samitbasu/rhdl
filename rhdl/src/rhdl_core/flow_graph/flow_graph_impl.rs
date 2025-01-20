use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    io::Write,
};

use fnv::FnvHasher;
use petgraph::prelude::StableDiGraph;

use crate::rhdl_core::{
    ast::source::{source_location::SourceLocation, spanned_source_set::SpannedSourceSet},
    hdl::ast::Module,
    rtl::object::RegisterKind,
    Circuit, Digital, HDLDescriptor, RHDLError, Synchronous,
};

use super::{
    component::{BBInput, BBOutput, BitSelect, Component, ComponentKind},
    edge_kind::EdgeKind,
    flow_cost::{compute_flow_cost, FlowCost},
    hdl::generate_hdl,
};

pub type FlowIx = petgraph::graph::NodeIndex;
pub type GraphType = StableDiGraph<Component, EdgeKind>;

#[derive(Clone, Hash, PartialEq, Copy, Debug)]
pub enum BlackBoxMode {
    Synchronous,
    Asynchronous,
}

#[derive(Clone, Hash)]
pub struct BlackBox {
    pub inputs: Vec<Vec<FlowIx>>,
    pub outputs: Vec<FlowIx>,
    pub code: HDLDescriptor,
    pub mode: BlackBoxMode,
}

#[derive(Clone, Default)]
pub struct FlowGraph {
    pub graph: GraphType,
    pub inputs: Vec<Vec<FlowIx>>,
    pub output: Vec<FlowIx>,
    pub code: SpannedSourceSet,
    pub black_boxes: Vec<BlackBox>,
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
    fn bb_input(&mut self, name: &str, len: usize) -> Vec<FlowIx> {
        (0..len)
            .map(|bit_index| {
                self.graph.add_node(Component {
                    kind: ComponentKind::BBInput(BBInput {
                        name: format!("{name}_{bit_index}"),
                        bit_index,
                    }),
                    width: 1,
                    location: None,
                })
            })
            .collect()
    }
    fn bb_output(&mut self, name: &str, len: usize) -> Vec<FlowIx> {
        (0..len)
            .map(|bit_index| {
                self.graph.add_node(Component {
                    kind: ComponentKind::BBOutput(BBOutput {
                        name: format!("{name}_{bit_index}"),
                        bit_index,
                    }),
                    width: 1,
                    location: None,
                })
            })
            .collect()
    }
    pub fn circuit_black_box<C: Circuit>(
        &mut self,
        code: HDLDescriptor,
    ) -> (Vec<FlowIx>, Vec<FlowIx>) {
        let input_len = C::I::BITS;
        let output_len = C::O::BITS;
        let arg0 = self.input(RegisterKind::Unsigned(input_len), 0, "i");
        let out = self.output(RegisterKind::Unsigned(output_len), "o");
        let inputs = self.bb_input(&code.name, input_len);
        let outputs = self.bb_output(&code.name, output_len);
        self.black_boxes.push(BlackBox {
            inputs: vec![inputs.clone()],
            outputs: outputs.clone(),
            code,
            mode: BlackBoxMode::Asynchronous,
        });
        self.zip(arg0.clone().into_iter(), inputs.into_iter());
        self.zip(outputs.into_iter(), out.clone().into_iter());
        (arg0, out)
    }
    pub fn synchronous_black_box<S: Synchronous>(
        &mut self,
        code: HDLDescriptor,
    ) -> (Vec<FlowIx>, Vec<FlowIx>, Vec<FlowIx>) {
        let input_len = S::I::BITS;
        let output_len = S::O::BITS;
        let arg0 = self.input(RegisterKind::Unsigned(2), 0, "clock_reset");
        let arg1 = self.input(RegisterKind::Unsigned(input_len), 1, "i");
        let out = self.output(RegisterKind::Unsigned(output_len), "o");
        let inputs = self.bb_input(&code.name, input_len);
        let outputs = self.bb_output(&code.name, output_len);
        let clock_reset = self.bb_input(&format!("{}_cr", code.name), 2);
        self.black_boxes.push(BlackBox {
            inputs: vec![clock_reset.clone(), inputs.clone()],
            outputs: outputs.clone(),
            code,
            mode: BlackBoxMode::Synchronous,
        });
        self.zip(arg0.clone().into_iter(), clock_reset.into_iter());
        self.zip(arg1.clone().into_iter(), inputs.into_iter());
        self.zip(outputs.into_iter(), out.clone().into_iter());
        (arg0, arg1, out)
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
    pub fn clock(&mut self, source: FlowIx, target: impl Iterator<Item = FlowIx>) {
        target.for_each(|target| {
            self.edge(source, target, EdgeKind::Clock);
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
        self.black_boxes.extend(other.black_boxes.iter().map(|bb| {
            BlackBox {
                inputs: bb
                    .inputs
                    .iter()
                    .map(|ix| ix.iter().map(|ix| remap[ix]).collect())
                    .collect(),
                outputs: bb.outputs.iter().map(|ix| remap[ix]).collect(),
                ..bb.clone()
            }
        }));
        self.code.extend(other.code.sources.clone());
        remap
    }
    pub fn hdl(&self, name: &str) -> Result<Module, RHDLError> {
        generate_hdl(name, self)
    }
    pub fn dot(&self) -> Result<Vec<u8>, RHDLError> {
        let mut s = vec![];
        super::dot::write_dot(self, &mut s)?;
        Ok(s)
    }
    pub fn write_dot(&self, t: impl AsRef<std::path::Path>) -> Result<(), RHDLError> {
        let mut file = std::fs::File::create(t)?;
        let s = self.dot()?;
        file.write_all(&s)?;
        Ok(())
    }
    pub fn hash_value(&self) -> u64 {
        let mut hasher = FnvHasher::default();
        for node in self.graph.node_indices() {
            node.hash(&mut hasher);
            self.graph[node].hash(&mut hasher);
        }
        for edge in self.graph.edge_indices() {
            edge.hash(&mut hasher);
            self.graph[edge].hash(&mut hasher);
        }
        self.inputs.hash(&mut hasher);
        self.output.hash(&mut hasher);
        self.code.hash(&mut hasher);
        hasher.finish()
    }
    pub fn flow_cost<F: Fn(&Component) -> f64>(&self, cost: F) -> FlowCost {
        compute_flow_cost(self, cost)
    }
    pub fn timing_reports<F: Fn(&Component) -> f64>(&self, cost: F) -> Vec<miette::Report> {
        self.flow_cost(cost).timing_reports(self)
    }
}
