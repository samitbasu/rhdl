use super::circuit_impl::Circuit;
use crate::flow_graph::flow_graph_impl::FlowGraph;
use crate::rhif::spec::Member;
use crate::schematic::builder::build_schematic;
use crate::schematic::components::{
    ComponentKind, FieldPin, IndexComponent, KernelComponent, StructComponent,
};
use crate::schematic::schematic_impl::Schematic;
use crate::types::digital::Digital;
use crate::types::path::Path;
use crate::types::tristate::Tristate;
use crate::{compile_design, RHDLError, Synchronous};
use crate::{util::hash_id, Kind};
use std::collections::HashMap;

// A few notes on the circuit descriptor struct
// The idea here is to capture the details on the circuit in such
// a way that it can be manipulated at run time.  This means that
// information encoded in the type system must be lifted into the
// runtime description.  And the repository for that information
// is the CircuitDescriptor struct.  We cannot, for example, iterate
// over the types that make up our children.
#[derive(Clone, Debug)]
pub struct CircuitDescriptor {
    pub unique_name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub d_kind: Kind,
    pub q_kind: Kind,
    pub num_tristate: usize,
    pub tristate_offset_in_parent: usize,
    pub update_schematic: Option<Schematic>,
    pub update_flow_graph: FlowGraph,
    pub children: HashMap<String, CircuitDescriptor>,
}

impl CircuitDescriptor {
    pub fn add_child<C: Circuit>(&mut self, name: &str, circuit: &C) {
        self.children.insert(name.into(), circuit.descriptor());
    }
    pub fn add_synchronous<S: Synchronous>(
        &mut self,
        name: &str,
        circuit: &S,
    ) -> Result<(), RHDLError> {
        self.children.insert(name.into(), circuit.descriptor()?);
        Ok(())
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
        let (input_buffer_in, input_buffer_out) =
            schematic.make_buffer(self.input_kind.clone(), None);
        let (output_buffer_in, output_buffer_out) =
            schematic.make_buffer(self.output_kind.clone(), None);
        // Next, we create the Q buffer, which holds the outputs of the children, and aggregates them
        // into a single output for feeding into the update function
        let q_output_pin = schematic.make_pin(self.q_kind.clone(), "Q".into(), None);
        let q_fields = self
            .children
            .iter()
            .map(|(name, child)| {
                let pin = schematic.make_pin(child.q_kind.clone(), name.clone(), None);
                FieldPin {
                    pin,
                    member: Member::Named(name.clone()),
                }
            })
            .collect::<Vec<_>>();
        let q_buffer = schematic.make_component(
            ComponentKind::Struct(StructComponent {
                kind: self.q_kind.clone(),
                fields: q_fields.clone(),
                output: q_output_pin,
                rest: None,
            }),
            None,
        );
        schematic.pin_mut(q_output_pin).parent(q_buffer);
        q_fields.iter().for_each(|f| {
            schematic.pin_mut(f.pin).parent(q_buffer);
        });
        // Now, create the update kernel component, and wire it to the input buffer and the output of the
        // Q buffer
        let update_output_kind =
            Kind::make_tuple(vec![self.output_kind.clone(), self.d_kind.clone()]);
        let update_input_pin =
            schematic.make_pin(self.input_kind.clone(), "update_in".into(), None);
        let update_output_pin =
            schematic.make_pin(update_output_kind.clone(), "update_out".into(), None);
        let update_q_pin = schematic.make_pin(self.q_kind.clone(), "update_q".into(), None);
        let update_component = schematic.make_component(
            ComponentKind::Kernel(KernelComponent {
                name: "update".into(),
                args: vec![update_input_pin, update_q_pin],
                sub_schematic: self.update_schematic.clone()?,
                output: update_output_pin,
            }),
            None,
        );
        schematic.pin_mut(update_input_pin).parent(update_component);
        schematic.pin_mut(update_q_pin).parent(update_component);
        schematic
            .pin_mut(update_output_pin)
            .parent(update_component);
        schematic.wire(input_buffer_out, update_input_pin);
        schematic.wire(q_output_pin, update_q_pin);
        // Next, we split the output of the update kernel to feed the output of the circuit
        let outfeed_in_pin =
            schematic.make_pin(update_output_kind.clone(), "outfeed_in".into(), None);
        let outfeed_out_pin =
            schematic.make_pin(self.output_kind.clone(), "outfeed_out".into(), None);
        let outfeed_component = schematic.make_component(
            ComponentKind::Index(IndexComponent {
                arg: outfeed_in_pin,
                path: Path::default().index(0),
                output: outfeed_out_pin,
                dynamic: vec![],
                kind: update_output_kind.clone(),
            }),
            None,
        );
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
                let index_to_child_pin =
                    schematic.make_pin(child.input_kind.clone(), name.clone(), None);
                let index_from_update_pin =
                    schematic.make_pin(update_output_kind.clone(), "child_outfeed_in".into(), None);
                let index_component = schematic.make_component(
                    ComponentKind::Index(IndexComponent {
                        arg: index_from_update_pin,
                        path: Path::default().index(1).field(name),
                        output: index_to_child_pin,
                        dynamic: vec![],
                        kind: child.input_kind.clone(),
                    }),
                    None,
                );
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
                schematic.make_pin(child_descriptor.input_kind.clone(), name.clone(), None);
            let child_output_pin =
                schematic.make_pin(child_descriptor.output_kind.clone(), name.clone(), None);
            let sub_schematic = child_descriptor.schematic()?;
            let child_component = schematic.make_component(
                ComponentKind::Kernel(KernelComponent {
                    name: name.clone(),
                    args: vec![child_input_pin],
                    sub_schematic,
                    output: child_output_pin,
                }),
                None,
            );
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
}

fn root_schematic<C: Circuit>() -> Option<Schematic> {
    let module =
        compile_design::<C::Update>(crate::compiler::driver::CompilationMode::Asynchronous).ok()?;
    let schematic = build_schematic(&module, module.top).ok()?;
    Some(schematic)
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
        update_schematic: root_schematic::<C>(),
        update_flow_graph: FlowGraph::default(),
        tristate_offset_in_parent: 0,
        children: Default::default(),
    }
}
