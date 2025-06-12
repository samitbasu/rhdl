use crate::rhdl_core::{
    bitx::BitX,
    digital_fn::NoKernel2,
    hdl::ast::{
        component_instance, concatenate, connection, id, index, index_bit, Direction, Module,
    },
    ntl,
    rtl::object::RegisterKind,
    types::{kind::Field, signal::signal},
    Circuit, CircuitDQ, CircuitDescriptor, CircuitIO, ClockReset, Digital, DigitalFn, Domain, Kind,
    RHDLError, Signal, Synchronous, Timed,
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

impl<C: Synchronous + Default, D: Domain> Default for Adapter<C, D> {
    fn default() -> Self {
        Self::new(C::default())
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
    fn static_kind() -> crate::rhdl_core::Kind {
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
    fn bin(self) -> Vec<BitX> {
        let mut out = vec![];
        out.extend(self.clock_reset.bin());
        out.extend(self.input.bin());
        out
    }
    fn dont_care() -> Self {
        Self {
            clock_reset: Signal::dont_care(),
            input: Signal::dont_care(),
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
    type S = C::S;

    fn init(&self) -> Self::S {
        self.circuit.init()
    }

    fn sim(&self, input: AdapterInput<C::I, D>, state: &mut C::S) -> Signal<C::O, D> {
        let clock_reset = input.clock_reset.val();
        let input = input.input.val();
        let result = self.circuit.sim(clock_reset, input, state);
        signal(result)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        // We build a custom flow graph to connect the input to the circuit and the circuit to the output.
        let mut builder = ntl::Builder::new(name);
        let child_descriptor = self.circuit.descriptor(&format!("{name}_inner"))?;
        // This includes the clock and reset signals
        // It should be [clock, reset, inputs...]
        let input_reg: RegisterKind = <Self::I as Timed>::static_kind().into();
        let output_reg: RegisterKind = <Self::O as Timed>::static_kind().into();
        let ti = builder.add_input(input_reg.len());
        let to = builder.allocate_outputs(output_reg.len());
        let child_offset = builder.link(&child_descriptor.ntl);
        let child_inputs = child_descriptor.ntl.inputs.iter().flatten();
        for (&t, c) in ti.iter().zip(child_inputs) {
            builder.copy_from_to(t, c.offset(child_offset));
        }
        for (&t, c) in to.iter().zip(&child_descriptor.ntl.outputs) {
            builder.copy_from_to(c.offset(child_offset), t);
        }
        Ok(CircuitDescriptor {
            unique_name: name.into(),
            input_kind: <<Self as CircuitIO>::I as Timed>::static_kind(),
            output_kind: <<Self as CircuitIO>::O as Timed>::static_kind(),
            d_kind: <<Self as CircuitDQ>::D as Timed>::static_kind(),
            q_kind: <<Self as CircuitDQ>::Q as Timed>::static_kind(),
            ntl: builder.build(),
            rtl: None,
            children: Default::default(),
        })
    }

    fn description(&self) -> String {
        format!("Asynchronous adaptor for {}", self.circuit.description())
    }

    fn hdl(&self, name: &str) -> Result<crate::rhdl_core::HDLDescriptor, RHDLError> {
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
        let clock_reset = concatenate(vec![index_bit("i", 1), index_bit("i", 0)]);
        let cr_connection = Some(connection("clock_reset", clock_reset));
        let input_connection = (!child.input_kind.is_empty())
            .then(|| connection("i", index("i", 2..(2 + child.input_kind.bits()))));
        let output_connection = Some(connection("o", id("o")));
        let child_decl = component_instance(
            &child.unique_name,
            "c",
            vec![cr_connection, input_connection, output_connection]
                .into_iter()
                .flatten()
                .collect(),
        );
        module.statements.push(child_decl);
        let child_hdl = self.circuit.hdl(child_name)?;
        Ok(crate::rhdl_core::HDLDescriptor {
            name: child_name.into(),
            body: module,
            children: [("c".into(), child_hdl)].into(),
        })
    }
}

impl<C: Synchronous, D: Domain> DigitalFn for Adapter<C, D> {}
