use crate::circuit::circuit_impl::Tristate;
use crate::diagnostic::dfg::{build_dfg, Component, ComponentKind, DFGType, Link, DFG};
use crate::path::Path;
use crate::types::digital::Digital;
use crate::types::digital_fn::DigitalFn;
use crate::{compile_design, KernelFnKind};
use crate::{util::hash_id, Kind};
use std::collections::HashMap;

use super::circuit_impl::Circuit;

#[derive(Clone, Debug)]
pub struct CircuitDescriptor {
    pub unique_name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub d_kind: Kind,
    pub q_kind: Kind,
    pub num_tristate: usize,
    pub tristate_offset_in_parent: usize,
    pub update_dfg: Option<DFG>,
    pub children: HashMap<String, CircuitDescriptor>,
}

impl CircuitDescriptor {
    pub fn add_child<C: Circuit>(&mut self, name: &str, circuit: &C) {
        self.children.insert(name.into(), circuit.descriptor());
    }
    // This is a drawing of the circuit dfg construction
    //
    //          +--------------------+
    //   -----> In                Out >-------->
    //          |        update      |
    //     +--> Q                   D >-+
    //     |    |                    |  |
    //     |    +--------------------+  |
    //     |                            |
    //     +--< Out   child 0      In <-+
    //     |                            |
    //     +--< Out    child 1     In <-+
    //
    //  We create buffer nodes for the input and output, D and Q
    //  and then connect the update DFG to these node.  The
    //  children DFGs are then connected to the D and Q nodes
    //  using recursion.
    pub fn dfg(&self) -> Option<DFG> {
        // We need to build a DFG for the entire circuit, which includes the children
        // DFGs linked with the update function DFG already stored in the circuit descriptor
        let mut total_dfg = DFG::default();
        // Create a node for the input of the circuit
        let input_node = total_dfg.buffer("input", self.input_kind.clone());
        // Create a node for the output of the circuit
        let output_node = total_dfg.buffer("output", self.output_kind.clone());
        // Create a node for the D node of the circuit
        let d_node = total_dfg.buffer("d", self.d_kind.clone());
        // Create a node for the Q node of the circuit
        let q_node = total_dfg.buffer("q", self.q_kind.clone());
        total_dfg.arguments.push(input_node);
        total_dfg.ret = output_node;
        if let Some(update_dfg) = &self.update_dfg {
            let update_relocation = total_dfg.merge(update_dfg);
            if !update_dfg.arguments.is_empty() {
                // Connect the first argument of the update function to the input buffer
                total_dfg.graph.add_edge(
                    input_node,
                    update_relocation[&update_dfg.arguments[0]],
                    Link::default(),
                );
                // Connect the second argument of the update function to the q node
                total_dfg.graph.add_edge(
                    q_node,
                    update_relocation[&update_dfg.arguments[1]],
                    Link::default(),
                );
            }
            // Connect the first element of the output tuple from the update kernel
            // to the output buffer
            total_dfg.graph.add_edge(
                update_relocation[&update_dfg.ret],
                output_node,
                Link {
                    src: Path::default().index(0),
                    dest: Path::default(),
                },
            );
            // Connect the second element of the output tuple from the update kernel
            // to the d buffer
            total_dfg.graph.add_edge(
                update_relocation[&update_dfg.ret],
                d_node,
                Link {
                    src: Path::default().index(1),
                    dest: Path::default(),
                },
            );
        }
        for (child_name, child_descriptor) in &self.children {
            if let Some(child_dfg) = child_descriptor.dfg() {
                let child_relocation = total_dfg.merge(&child_dfg);
                if child_relocation.is_empty() {
                    continue;
                }
                // Connect the output of the child to the Q node
                total_dfg.graph.add_edge(
                    child_relocation[&child_dfg.ret],
                    q_node,
                    Link {
                        src: Path::default(),
                        dest: Path::default().field(child_name),
                    },
                );
                if child_dfg.arguments.is_empty() {
                    continue;
                }
                // Connect the D node to the input of the child
                total_dfg.graph.add_edge(
                    d_node,
                    child_relocation[&child_dfg.arguments[0]],
                    Link {
                        src: Path::default().field(child_name),
                        dest: Path::default(),
                    },
                );
            }
        }
        Some(total_dfg)
    }
}

fn root_dfg<C: Circuit>() -> Option<DFG> {
    if let Some(KernelFnKind::Kernel(kernel)) = C::Update::kernel_fn() {
        let design = compile_design(kernel).ok()?;
        build_dfg(&design, design.top).ok()
    } else {
        None
    }
}

pub fn root_descriptor<C: Circuit>(circuit: &C) -> CircuitDescriptor {
    CircuitDescriptor {
        unique_name: format!(
            "{}_{:x}",
            circuit.name(),
            hash_id(std::any::TypeId::of::<C>())
        ),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        d_kind: C::D::static_kind(),
        q_kind: C::Q::static_kind(),
        num_tristate: C::Z::N,
        update_dfg: root_dfg::<C>(),
        tristate_offset_in_parent: 0,
        children: Default::default(),
    }
}
