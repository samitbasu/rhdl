use std::collections::BTreeMap;

use crate::{
    digital_fn::DigitalFn3,
    flow_graph::edge_kind::EdgeKind,
    hdl::ast::{
        component_instance, connection, id, unsigned_width, Declaration, Direction, HDLKind, Module,
    },
    note_pop_path, note_push_path,
    rtl::object::RegisterKind,
    CircuitDescriptor, ClockReset, Digital, DigitalFn, FlowGraph, HDLDescriptor, Kind, Synchronous,
    SynchronousDQ, SynchronousIO, Tristate,
};

use super::{hdl_backend::maybe_port_wire, synchronous::SynchronousKernel};

#[derive(Clone)]
pub struct Chain<A, B> {
    a: A,
    b: B,
}

impl<A, B> Chain<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Synchronous, B: Synchronous> SynchronousIO for Chain<A, B> {
    type I = <A as SynchronousIO>::I;
    type O = <B as SynchronousIO>::O;
}

impl<A: Synchronous, B: Synchronous> SynchronousDQ for Chain<A, B> {
    type D = ();

    type Q = ();
}

impl<A: Synchronous, B: Synchronous> DigitalFn for Chain<A, B> {}

impl<A: Synchronous, B: Synchronous> DigitalFn3 for Chain<A, B> {
    type A0 = ClockReset;
    type A1 = <Self as SynchronousIO>::I;
    type A2 = ();
    type O = (<Self as SynchronousIO>::O, ());

    fn func() -> fn(Self::A0, Self::A1, Self::A2) -> Self::O {
        unimplemented!("Chain::func")
    }
}

impl<A: Synchronous, B: Synchronous> SynchronousKernel for Chain<A, B> {
    type Kernel = Self;
}

impl<A: Synchronous, B: Synchronous, P: Digital> Synchronous for Chain<A, B>
where
    A: SynchronousIO<O = P>,
    B: SynchronousIO<I = P>,
{
    type Z = (A::Z, B::Z);

    type S = (A::S, B::S);

    fn sim(
        &self,
        clock_reset: crate::ClockReset,
        input: Self::I,
        state: &mut Self::S,
        io: &mut Self::Z,
    ) -> Self::O {
        note_push_path("chain");
        note_push_path("a");
        let p = self.a.sim(clock_reset, input, &mut state.0, &mut io.0);
        note_pop_path();
        note_push_path("b");
        let o = self.b.sim(clock_reset, p, &mut state.1, &mut io.1);
        note_pop_path();
        note_pop_path();
        o
    }

    fn description(&self) -> String {
        format!(
            "series synchronous circuit of {} and {}",
            self.a.description(),
            self.b.description()
        )
    }

    fn descriptor(&self, name: &str) -> Result<crate::CircuitDescriptor, crate::RHDLError> {
        let a_name = format!("{name}_a");
        let b_name = format!("{name}_b");
        let desc_a = self.a.descriptor(&a_name)?;
        let desc_b = self.b.descriptor(&b_name)?;
        let fg_a = &desc_a.flow_graph;
        let fg_b = &desc_b.flow_graph;
        let mut fg = FlowGraph::default();
        let a_remap = fg.merge(fg_a);
        let b_remap = fg.merge(fg_b);
        // Create a buffer to hold the clock and reset signals
        let cr_buffer = fg.buffer(RegisterKind::Unsigned(2), "cr", None);
        // Connect the clock and reset signals to the input of the first circuit
        for (cr, a_input) in cr_buffer.iter().zip(&fg_a.inputs[0]) {
            fg.edge(*cr, a_remap[a_input], EdgeKind::ArgBit(0, 0));
        }
        // Connect the clock and reset signals to the input of the second circuit
        for (cr, b_input) in cr_buffer.iter().zip(&fg_b.inputs[0]) {
            fg.edge(*cr, b_remap[b_input], EdgeKind::ArgBit(0, 0));
        }
        let input_kind: RegisterKind = <A as SynchronousIO>::I::static_kind().into();
        // Allocate the input buffer for the chain
        let input_buffer = fg.input(input_kind, 0, name);
        // Connect the input buffer to the input of the first circuit
        for (parent_input, child_input) in input_buffer.iter().zip(&fg_a.inputs[1]) {
            fg.edge(*parent_input, a_remap[child_input], EdgeKind::ArgBit(0, 0));
        }
        // Connect the output of the first circuit to the input of the second circuit
        for (a_output, b_input) in fg_a.output.iter().zip(&fg_b.inputs[1]) {
            fg.edge(a_remap[a_output], b_remap[b_input], EdgeKind::ArgBit(0, 0));
        }
        // Allocate the output buffer for the chain
        let output_kind: RegisterKind = <B as SynchronousIO>::O::static_kind().into();
        let output_buffer = fg.output(output_kind, name);
        // Connect the output of the second circuit to the output buffer
        for (b_output, parent_output) in fg_b.output.iter().zip(output_buffer.iter()) {
            fg.edge(b_remap[b_output], *parent_output, EdgeKind::ArgBit(0, 0));
        }
        fg.inputs = vec![cr_buffer, input_buffer];
        fg.output = output_buffer;
        let desc = CircuitDescriptor {
            unique_name: name.into(),
            input_kind: desc_a.input_kind.clone(),
            output_kind: desc_b.output_kind.clone(),
            q_kind: Kind::Empty,
            d_kind: Kind::Empty,
            num_tristate: Self::Z::N,
            tristate_offset_in_parent: 0,
            flow_graph: fg,
            rtl: None,
            children: BTreeMap::from_iter(vec![(a_name, desc_a), (b_name, desc_b)]),
        };
        Ok(desc)
    }

    fn hdl(&self, name: &str) -> Result<crate::HDLDescriptor, crate::RHDLError> {
        let mut module = Module {
            name: name.into(),
            description: self.description(),
            ..Default::default()
        };
        let input_kind = <A as SynchronousIO>::I::static_kind();
        let pipe_kind = <A as SynchronousIO>::O::static_kind();
        module.ports = [
            maybe_port_wire(Direction::Input, 2, "clock_reset"),
            maybe_port_wire(Direction::Input, <A as SynchronousIO>::I::bits(), "i"),
            maybe_port_wire(Direction::Output, <B as SynchronousIO>::O::bits(), "o"),
            maybe_port_wire(Direction::Inout, Self::Z::N, "io"),
        ]
        .into_iter()
        .flatten()
        .collect();
        module.declarations.push(Declaration {
            kind: HDLKind::Wire,
            name: "pipe".into(),
            width: unsigned_width(pipe_kind.bits()),
            alias: None,
        });
        let a_name = &format!("{name}_a");
        let b_name = &format!("{name}_b");
        // Add the two child components.
        let a_input_binding = if input_kind.is_empty() {
            None
        } else {
            Some(connection("i", id("i")))
        };
        let cr_binding = Some(connection("clock_reset", id("clock_reset")));
        let b_output_binding = Some(connection("o", id("o")));
        let a_p_binding = Some(connection("o", id("pipe")));
        let b_i_binding = Some(connection("i", id("pipe")));
        let a_instance = component_instance(
            a_name,
            "a",
            [cr_binding.clone(), a_input_binding.clone(), a_p_binding]
                .into_iter()
                .flatten()
                .collect(),
        );
        let b_instance = component_instance(
            b_name,
            "b",
            [cr_binding, b_i_binding, b_output_binding]
                .into_iter()
                .flatten()
                .collect(),
        );
        let a_hdl = self.a.hdl(a_name)?;
        let b_hdl = self.b.hdl(b_name)?;
        module.statements.extend([a_instance, b_instance]);
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: BTreeMap::from_iter(vec![(a_name.into(), a_hdl), (b_name.into(), b_hdl)]),
        })
    }
}
