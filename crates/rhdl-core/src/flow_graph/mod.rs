use std::{collections::HashMap, sync::Arc};

use crate::{
    Kind, RHDLError, TypedBits,
    ast::spanned_source::SpannedSourceSet,
    error::rhdl_error,
    flow_graph::errors::{FlowGraphICE, LogicLoopError, LoopStepError},
    rhif::{Object, object::LocatedOpCode, spec::Slot},
    types::path::Path,
};
use petgraph::prelude::*;

pub mod black_box;
pub mod builder;
pub mod circuit_builder;
pub mod errors;
pub mod rhif_builder;
pub mod synchronous_builder;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PathRef {
    pub object: Arc<Object>,
    pub slot: Slot,
    pub path: Path,
}

impl std::fmt::Debug for PathRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}{:?}", self.object.name, self.slot, self.path)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct BufferRef {
    pub name: String,
    pub kind: Kind,
    pub path: Path,
}

impl std::fmt::Debug for BufferRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{:?}", self.name, self.path)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConstantRef {
    pub name: String,
    pub value: TypedBits,
    pub path: Path,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OpCodeRef {
    pub opcode: LocatedOpCode,
    pub object: Arc<Object>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KernelDToChildInput {
    pub parent: &'static str,
    pub child: &'static str,
    pub path: Path,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ChildOutputKernelQ {
    pub parent: &'static str,
    pub child: &'static str,
    pub path: Path,
}

#[derive(Clone, Hash, PartialEq, Debug, Eq)]
pub enum EdgeKind {
    OpCode(OpCodeRef),
    InputToKernel,
    ChildOutputKernelQ(ChildOutputKernelQ),
    KernelToOutput,
    KernelDToChildInput(KernelDToChildInput),
    ClockResetToKernel,
    InputForwardToChild,
    OutputForwardFromChild,
    ChildToChild,
}

#[derive(Clone, Hash, PartialEq, Debug, Eq)]
pub enum NodeKind {
    Slot(PathRef),
    Buffer(BufferRef),
    Constant(ConstantRef),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InputRef {
    pub index: usize,
    pub kind: Kind,
    pub path: Path,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OutputRef {
    pub kind: Kind,
    pub path: Path,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum NodePort {
    Input(usize, Path),
    Output(Path),
}

#[derive(Clone, Debug)]
pub struct FlowGraph {
    /// The name of the thing this flow graph represents
    pub name: String,
    pub graph: DiGraph<NodeKind, EdgeKind>,
    pub ports: HashMap<NodePort, NodeIndex>,
    pub source: SpannedSourceSet,
}

impl FlowGraph {
    pub fn input_port(&self, index: usize, path: &Path) -> Result<NodeIndex, RHDLError> {
        self.ports
            .get(&NodePort::Input(index, path.clone()))
            .copied()
            .ok_or_else(|| {
                FlowGraphICE::MissingInputPortForIndexAndPath {
                    index,
                    path: path.clone(),
                }
                .into()
            })
    }
    pub fn output_port(&self, path: &Path) -> Result<NodeIndex, RHDLError> {
        self.ports
            .get(&NodePort::Output(path.clone()))
            .copied()
            .ok_or_else(|| {
                FlowGraphICE::MissingOutputPortForPath {
                    path: path.clone(),
                    available: self
                        .ports
                        .keys()
                        .filter_map(|port| {
                            if let NodePort::Output(p) = port {
                                Some(p.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                }
                .into()
            })
    }
    pub fn dot(&self) -> String {
        format!("{:?}", petgraph::dot::Dot::new(&self.graph))
    }
    pub fn loop_checked(self) -> Result<Self, RHDLError> {
        let fg = &self.graph;
        if let Err(cycle) = petgraph::algo::toposort(&fg, None) {
            let cycle_node = cycle.node_id();
            let components = petgraph::algo::kosaraju_scc(&fg);
            for component in components {
                if component.contains(&cycle_node) {
                    let mut elements = vec![];
                    for nodes in component.windows(2) {
                        let from = nodes[0];
                        let to = nodes[1];
                        if let Some(edge) = fg.find_edge(from, to) {
                            let edge = &fg[edge];
                            elements.push(LoopStepError {
                                from: fg[from].clone(),
                                edge: edge.clone(),
                                to: fg[to].clone(),
                                pool: Some(self.source.source()),
                            });
                        }
                    }
                    if let Some(edge) = fg.find_edge(*component.last().unwrap(), component[0]) {
                        let edge = &fg[edge];
                        elements.push(LoopStepError {
                            from: fg[*component.last().unwrap()].clone(),
                            edge: edge.clone(),
                            to: fg[component[0]].clone(),
                            pool: Some(self.source.source()),
                        });
                    }
                    return Err(rhdl_error(LogicLoopError { errors: elements }));
                }
            }
        }
        Ok(self)
    }
}
