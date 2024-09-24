use crate::{
    flow_graph::edge_kind::EdgeKind,
    rtl::object::RegisterKind,
    types::{kind::Field, signal::signal},
    Circuit, CircuitDQ, CircuitDescriptor, CircuitIO, ClockReset, Digital, DigitalFn, Domain,
    FlowGraph, Kind, Notable, NoteKey, NoteWriter, RHDLError, Signal, Synchronous, Timed, Tristate,
};

use super::circuit_impl::CircuitUpdateFn;

// An adapter allows you to use a Synchronous circuit in an Asynchronous context.
#[derive(Clone)]
pub struct Adapter<C: Synchronous, D: Domain> {
    circuit: C,
    domain: std::marker::PhantomData<D>,
}

impl<C: Synchronous, D: Domain> Adapter<C, D> {
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            domain: Default::default(),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AdapterInput<I: Digital, D: Domain> {
    pub clock_reset: Signal<ClockReset, D>,
    pub input: Signal<I, D>,
}

impl<I: Digital, D: Domain> Timed for AdapterInput<I, D> {}

impl<I: Digital, D: Domain> Notable for AdapterInput<I, D> {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.clock_reset.note((key, "clock_reset"), &mut writer);
        self.input.note((key, "input"), &mut writer);
    }
}

impl<I: Digital, D: Domain> Digital for AdapterInput<I, D> {
    fn static_kind() -> crate::Kind {
        Kind::make_struct(
            "AdapterInput",
            vec![
                Field {
                    name: "clock_reset".into(),
                    kind: <Signal<ClockReset, D> as Digital>::static_kind(),
                },
                Field {
                    name: "input".into(),
                    kind: <Signal<I, D> as Digital>::static_kind(),
                },
            ],
        )
    }
    fn bin(self) -> Vec<bool> {
        let mut out = vec![];
        out.extend(self.clock_reset.bin());
        out.extend(self.input.bin());
        out
    }
    fn init() -> Self {
        Self {
            clock_reset: Signal::init(),
            input: Signal::init(),
        }
    }
}

impl<C: Synchronous, D: Domain> CircuitIO for Adapter<C, D> {
    type I = AdapterInput<C::I, D>;
    type O = Signal<C::O, D>;
}

impl<C: Synchronous, D: Domain> CircuitDQ for Adapter<C, D> {
    type D = ();
    type Q = ();
}

impl<C: Synchronous, D: Domain> Circuit for Adapter<C, D> {
    type Z = C::Z;

    type Update = ();

    const UPDATE: CircuitUpdateFn<Self> = |_, _| unimplemented!();

    type S = Signal<C::S, D>;

    fn sim(
        &self,
        input: AdapterInput<C::I, D>,
        state: &mut Signal<C::S, D>,
        io: &mut C::Z,
    ) -> Signal<C::O, D> {
        let clock_reset = input.clock_reset.val();
        let input = input.input.val();
        let result = self.circuit.sim(clock_reset, input, state.val_mut(), io);
        signal(result)
    }

    fn descriptor(&self) -> Result<CircuitDescriptor, RHDLError> {
        // We build a custom flow graph to connect the input to the circuit and the circuit to the output.
        let mut fg = FlowGraph::default();
        // This includes the clock and reset signals
        // It should be [clock, reset, inputs...]
        let input_reg: RegisterKind = <Self::I as Timed>::static_kind().into();
        let input_buffer = fg.buffer(input_reg, "i", None);
        let output_reg: RegisterKind = <Self::O as Timed>::static_kind().into();
        let output_buffer = fg.buffer(output_reg, "o", None);
        // Embed the flow graph for the child circuit
        let child_descriptor = self.circuit.descriptor()?;
        let child_fg = &child_descriptor.flow_graph;
        let child_remap = fg.merge(child_fg);
        let child_inputs = child_fg.inputs[0].iter().chain(child_fg.inputs[1].iter());
        let parent_inputs = input_buffer.iter();
        for (parent_input, child_input) in parent_inputs.zip(child_inputs) {
            let child_input = child_remap[child_input];
            fg.edge(*parent_input, child_input, EdgeKind::Arg(0));
        }
        for (parent_output, child_output) in output_buffer.iter().zip(&child_fg.output) {
            let child_output = child_remap[child_output];
            fg.edge(child_output, *parent_output, EdgeKind::Arg(0));
        }
        fg.inputs = vec![input_buffer];
        fg.output = output_buffer;
        Ok(CircuitDescriptor {
            unique_name: format!("Adapter_{}", self.circuit.name()),
            input_kind: <<Self as CircuitIO>::I as Timed>::static_kind(),
            output_kind: <<Self as CircuitIO>::O as Timed>::static_kind(),
            d_kind: <<Self as CircuitDQ>::D as Timed>::static_kind(),
            q_kind: <<Self as CircuitDQ>::Q as Timed>::static_kind(),
            num_tristate: C::Z::N,
            tristate_offset_in_parent: 0,
            flow_graph: fg,
            rtl: None,
            children: Default::default(),
        })
    }

    fn name(&self) -> String {
        self.circuit.name()
    }

    fn as_hdl(&self, _kind: crate::HDLKind) -> Result<crate::HDLDescriptor, RHDLError> {
        todo!()
    }
}

impl<C: Synchronous, D: Domain> DigitalFn for Adapter<C, D> {}
