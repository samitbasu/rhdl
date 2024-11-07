use crate::{
    digital_fn::NoKernel2,
    flow_graph::edge_kind::EdgeKind,
    hdl::ast::{component_instance, connection, id, index, index_bit, Direction, Module},
    rtl::object::RegisterKind,
    types::{kind::Field, signal::signal},
    Circuit, CircuitDQ, CircuitDescriptor, CircuitIO, ClockReset, Digital, DigitalFn, Domain,
    FlowGraph, Kind, RHDLError, Signal, Synchronous, Timed,
};

use super::hdl_backend::maybe_port_wire;
use rhdl_trace_type as rtt;

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

impl<I: Digital, D: Domain> Digital for AdapterInput<I, D> {
    const BITS: usize = <Signal<ClockReset, D> as Digital>::BITS + <Signal<I, D> as Digital>::BITS;
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
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::make_struct(
            "AdapterInput",
            vec![
                rtt::Field {
                    name: "clock_reset".into(),
                    ty: <Signal<ClockReset, D> as Digital>::static_trace_type(),
                },
                rtt::Field {
                    name: "input".into(),
                    ty: <Signal<I, D> as Digital>::static_trace_type(),
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
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

impl<C: Synchronous, D: Domain> CircuitDQ for Adapter<C, D> {
    type D = ();
    type Q = ();
}

impl<C: Synchronous, D: Domain> Circuit for Adapter<C, D> {
    type S = Signal<C::S, D>;

    fn sim(&self, input: AdapterInput<C::I, D>, state: &mut Signal<C::S, D>) -> Signal<C::O, D> {
        let clock_reset = input.clock_reset.val();
        let input = input.input.val();
        let result = self.circuit.sim(clock_reset, input, state.val_mut());
        signal(result)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        // We build a custom flow graph to connect the input to the circuit and the circuit to the output.
        let mut fg = FlowGraph::default();
        // This includes the clock and reset signals
        // It should be [clock, reset, inputs...]
        let input_reg: RegisterKind = <Self::I as Timed>::static_kind().into();
        let input_buffer = fg.input(input_reg, 0, name);
        let output_reg: RegisterKind = <Self::O as Timed>::static_kind().into();
        let output_buffer = fg.output(output_reg, name);
        // Embed the flow graph for the child circuit
        let child_descriptor = self.circuit.descriptor(&format!("{name}_inner"))?;
        let child_fg = &child_descriptor.flow_graph;
        let child_remap = fg.merge(child_fg);
        let child_inputs = child_fg.inputs[0].iter().chain(child_fg.inputs[1].iter());
        let parent_inputs = input_buffer.iter();
        for (parent_input, child_input) in parent_inputs.zip(child_inputs) {
            let child_input = child_remap[child_input];
            fg.edge(*parent_input, child_input, EdgeKind::ArgBit(0, 0));
        }
        for (parent_output, child_output) in output_buffer.iter().zip(&child_fg.output) {
            let child_output = child_remap[child_output];
            fg.edge(child_output, *parent_output, EdgeKind::ArgBit(0, 0));
        }
        fg.inputs = vec![input_buffer];
        fg.output = output_buffer;
        Ok(CircuitDescriptor {
            unique_name: name.into(),
            input_kind: <<Self as CircuitIO>::I as Timed>::static_kind(),
            output_kind: <<Self as CircuitIO>::O as Timed>::static_kind(),
            d_kind: <<Self as CircuitDQ>::D as Timed>::static_kind(),
            q_kind: <<Self as CircuitDQ>::Q as Timed>::static_kind(),
            flow_graph: fg,
            rtl: None,
            children: Default::default(),
        })
    }

    fn description(&self) -> String {
        format!("Asynchronous adaptor for {}", self.circuit.description())
    }

    fn hdl(&self, name: &str) -> Result<crate::HDLDescriptor, RHDLError> {
        let mut module = Module {
            name: name.into(),
            description: self.description(),
            ..Default::default()
        };
        module.ports = [
            maybe_port_wire(Direction::Input, <Self as CircuitIO>::I::bits(), "i"),
            maybe_port_wire(Direction::Output, <Self as CircuitIO>::O::bits(), "o"),
        ]
        .into_iter()
        .flatten()
        .collect();
        let child_name = &format!("{}_inner", name);
        let child = self.circuit.descriptor(child_name)?;
        let clock_connection = Some(connection("clock", index_bit("i", 0)));
        let reset_connection = Some(connection("reset", index_bit("i", 1)));
        let input_connection = (!child.input_kind.is_empty())
            .then(|| connection("i", index("i", 2..(2 + child.input_kind.bits()))));
        let output_connection = Some(connection("o", id("o")));
        let child_decl = component_instance(
            &child.unique_name,
            "c",
            vec![
                clock_connection,
                reset_connection,
                input_connection,
                output_connection,
            ]
            .into_iter()
            .flatten()
            .collect(),
        );
        module.statements.push(child_decl);
        let child_hdl = self.circuit.hdl(child_name)?;
        Ok(crate::HDLDescriptor {
            name: child_name.into(),
            body: module,
            children: [("c".into(), child_hdl)].into(),
        })
    }
}

impl<C: Synchronous, D: Domain> DigitalFn for Adapter<C, D> {}
