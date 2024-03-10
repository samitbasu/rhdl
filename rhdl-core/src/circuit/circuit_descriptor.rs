use crate::circuit::circuit_impl::Tristate;
use crate::diagnostic::dfg::{build_dfg, Link, DFG};
use crate::path::Path;
use crate::rhif::spec::Member;
use crate::schematic::builder::build_schematic;
use crate::schematic::components::{
    BlackBoxComponent, ComponentKind, FieldPin, IndexComponent, KernelComponent, StructComponent,
};
use crate::schematic::schematic_impl::Schematic;
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
    pub update_schematic: Option<Schematic>,
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

    // Create a schematic of the circuit.  It is modified by adding
    // a Q buffer and a D buffer.
    //          +--------------------+
    //   *in ---> In                Out >------*out
    //          |        update      |
    //     +--> Q                   D >-+
    //     |    |                    |  |
    //     |    +--------------------+  |
    //     |                            |
    //     +--< Out   child 0      In <-+
    //     |                            |
    //     +--< Out    child 1     In <-+
    pub fn schematic(&self) -> Option<Schematic> {
        let mut schematic = Schematic::default();
        // The input and output buffers hold the pins that enter and leave the schematic
        let (input_buffer_in, input_buffer_out) = schematic.make_buffer(self.input_kind.clone());
        let (output_buffer_in, output_buffer_out) = schematic.make_buffer(self.output_kind.clone());
        // Next, we create the Q buffer, which holds the outputs of the children, and aggregates them
        // into a single output for feeding into the update function
        let q_output_pin = schematic.make_pin(self.q_kind.clone(), "Q".into());
        let q_fields = self
            .children
            .iter()
            .map(|(name, child)| {
                let pin = schematic.make_pin(child.q_kind.clone(), name.clone());
                FieldPin {
                    pin,
                    member: Member::Named(name.clone()),
                }
            })
            .collect::<Vec<_>>();
        let q_buffer = schematic.make_component(ComponentKind::Struct(StructComponent {
            kind: self.q_kind.clone(),
            fields: q_fields.clone(),
            output: q_output_pin,
            rest: None,
        }));
        schematic.pin_mut(q_output_pin).parent(q_buffer);
        q_fields.iter().for_each(|f| {
            schematic.pin_mut(f.pin).parent(q_buffer);
        });
        // Now, create the update kernel component, and wire it to the input buffer and the output of the
        // Q buffer
        let update_output_kind =
            Kind::make_tuple(vec![self.output_kind.clone(), self.d_kind.clone()]);
        let update_input_pin = schematic.make_pin(self.input_kind.clone(), "update_in".into());
        let update_output_pin = schematic.make_pin(update_output_kind.clone(), "update_out".into());
        let update_q_pin = schematic.make_pin(self.q_kind.clone(), "update_q".into());
        let update_component = schematic.make_component(ComponentKind::Kernel(KernelComponent {
            name: "update".into(),
            args: vec![update_input_pin, update_q_pin],
            sub_schematic: self.update_schematic.clone()?,
            output: update_output_pin,
        }));
        schematic.pin_mut(update_input_pin).parent(update_component);
        schematic.pin_mut(update_q_pin).parent(update_component);
        schematic
            .pin_mut(update_output_pin)
            .parent(update_component);
        schematic.wire(input_buffer_out, update_input_pin);
        schematic.wire(q_output_pin, update_q_pin);
        // Next, we split the output of the update kernel to feed the output of the circuit
        let outfeed_in_pin = schematic.make_pin(update_output_kind.clone(), "outfeed_in".into());
        let outfeed_out_pin = schematic.make_pin(self.output_kind.clone(), "outfeed_out".into());
        let outfeed_component = schematic.make_component(ComponentKind::Index(IndexComponent {
            arg: outfeed_in_pin,
            path: Path::default().index(0),
            output: outfeed_out_pin,
            dynamic: vec![],
        }));
        schematic.pin_mut(outfeed_in_pin).parent(outfeed_component);
        schematic.pin_mut(outfeed_out_pin).parent(outfeed_component);
        schematic.wire(update_output_pin, outfeed_in_pin);
        schematic.wire(outfeed_out_pin, output_buffer_in);
        // We also create a set of index components to split the output for each
        // child
        let child_outfeed_ins = self
            .children
            .iter()
            .map(|(name, child)| {
                let index_to_child_pin = schematic.make_pin(child.input_kind.clone(), name.clone());
                let index_from_update_pin =
                    schematic.make_pin(update_output_kind.clone(), "child_outfeed_in".into());
                let index_component =
                    schematic.make_component(ComponentKind::Index(IndexComponent {
                        arg: index_from_update_pin,
                        path: Path::default().index(1).field(name),
                        output: index_to_child_pin,
                        dynamic: vec![],
                    }));
                schematic
                    .pin_mut(index_to_child_pin)
                    .parent(index_component);
                schematic
                    .pin_mut(index_from_update_pin)
                    .parent(index_component);
                schematic.wire(update_output_pin, index_from_update_pin);
                (name, index_to_child_pin)
            })
            .collect::<HashMap<_, _>>();
        // Now, we embed each of the children schematics
        for (name, child_descriptor) in &self.children {
            let child_input_pin =
                schematic.make_pin(child_descriptor.input_kind.clone(), name.clone());
            let child_output_pin =
                schematic.make_pin(child_descriptor.output_kind.clone(), name.clone());
            let child_component = if let Some(sub_schematic) = child_descriptor.schematic() {
                schematic.make_component(ComponentKind::Kernel(KernelComponent {
                    name: name.clone(),
                    args: vec![child_input_pin],
                    sub_schematic,
                    output: child_output_pin,
                }))
            } else {
                schematic.make_component(ComponentKind::BlackBox(BlackBoxComponent {
                    name: name.clone(),
                    args: vec![child_input_pin],
                    output: child_output_pin,
                }))
            };
            schematic.pin_mut(child_input_pin).parent(child_component);
            schematic.pin_mut(child_output_pin).parent(child_component);
            schematic.wire(child_outfeed_ins[name], child_input_pin);
            let q_pin = q_fields
                .iter()
                .find(|f| f.member == Member::Named(name.to_string()))
                .unwrap()
                .pin;
            schematic.wire(child_output_pin, q_pin);
        }
        schematic.inputs = vec![input_buffer_in];
        schematic.output = output_buffer_out;
        Some(schematic)
    }

    pub fn dfg(&self) -> Option<DFG> {
        // We need to build a DFG for the entire circuit, which includes the children
        // DFGs linked with the update function DFG already stored in the circuit descriptor
        let mut total_dfg = DFG::default();
        // Create a node for the input of the circuit
        let input_node = total_dfg.buffer("input", self.input_kind.clone());
        // Create a node for the output of the circuit
        let output_node = total_dfg.buffer("output", self.output_kind.clone());
        // Create a node for the D node of the circuit
        let d_node = if self.d_kind.is_empty() {
            None
        } else {
            Some(total_dfg.buffer("d", self.d_kind.clone()))
        };
        // Create a node for the Q node of the circuit
        let q_node = if self.q_kind.is_empty() {
            None
        } else {
            Some(total_dfg.buffer("q", self.q_kind.clone()))
        };
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
                if let Some(q_node) = q_node {
                    total_dfg.graph.add_edge(
                        q_node,
                        update_relocation[&update_dfg.arguments[1]],
                        Link::default(),
                    );
                }
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
            if let Some(d_node) = d_node {
                total_dfg.graph.add_edge(
                    update_relocation[&update_dfg.ret],
                    d_node,
                    Link {
                        src: Path::default().index(1),
                        dest: Path::default(),
                    },
                );
            }
        }
        for (child_name, child_descriptor) in &self.children {
            if let Some(child_dfg) = child_descriptor.dfg() {
                let child_relocation = total_dfg.merge(&child_dfg);
                if child_relocation.is_empty() {
                    continue;
                }
                if let Some(q_node) = q_node {
                    // Connect the output of the child to the Q node
                    total_dfg.graph.add_edge(
                        child_relocation[&child_dfg.ret],
                        q_node,
                        Link {
                            src: Path::default(),
                            dest: Path::default().field(child_name),
                        },
                    );
                }
                if child_dfg.arguments.is_empty() {
                    continue;
                }
                if let Some(d_node) = d_node {
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

fn root_schematic<C: Circuit>() -> Option<Schematic> {
    if let Some(KernelFnKind::Kernel(kernel)) = C::Update::kernel_fn() {
        let module = compile_design(kernel).ok()?;
        let schematic = build_schematic(&module, module.top).ok()?;
        Some(schematic)
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
        update_schematic: root_schematic::<C>(),
        tristate_offset_in_parent: 0,
        children: Default::default(),
    }
}
