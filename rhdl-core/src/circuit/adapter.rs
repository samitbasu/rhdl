use crate::{
    types::{kind::Field, signal::signal},
    Circuit, CircuitDQ, CircuitIO, Clock, Digital, DigitalFn, Domain, Kind, Notable, NoteKey,
    NoteWriter, Reset, Signal, Synchronous, Timed,
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
    pub clock: Signal<Clock, D>,
    pub reset: Signal<Reset, D>,
    pub input: Signal<I, D>,
}

impl<I: Digital, D: Domain> Timed for AdapterInput<I, D> {}

impl<I: Digital, D: Domain> Notable for AdapterInput<I, D> {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.clock.note((key, "clock"), &mut writer);
        self.reset.note((key, "reset"), &mut writer);
        self.input.note((key, "input"), &mut writer);
    }
}

impl<I: Digital, D: Domain> Digital for AdapterInput<I, D> {
    fn static_kind() -> crate::Kind {
        Kind::make_struct(
            "AdapterInput",
            vec![
                Field {
                    name: "clock".into(),
                    kind: <Signal<Clock, D> as Digital>::static_kind(),
                },
                Field {
                    name: "reset".into(),
                    kind: <Signal<Reset, D> as Digital>::static_kind(),
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
        out.extend(self.clock.bin());
        out.extend(self.reset.bin());
        out.extend(self.input.bin());
        out
    }
    fn uninit() -> Self {
        Self {
            clock: Signal::uninit(),
            reset: Signal::uninit(),
            input: Signal::uninit(),
        }
    }
}

impl<C: Synchronous, D: Domain> CircuitIO for Adapter<C, D> {
    type I = AdapterInput<C::I, D>;
    type O = Signal<C::O, D>;
}

impl<C: Synchronous, D: Domain> CircuitDQ for Adapter<C, D> {
    type D = Signal<C::D, D>;
    type Q = Signal<C::Q, D>;
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
        let clock = input.clock.val();
        let reset = input.reset.val();
        let input = input.input.val();
        let result = self.circuit.sim(clock, reset, input, state.val_mut(), io);
        signal(result)
    }

    fn name(&self) -> &'static str {
        self.circuit.name()
    }

    fn as_hdl(&self, _kind: crate::HDLKind) -> Result<crate::HDLDescriptor, crate::RHDLError> {
        todo!()
    }
}

impl<C: Synchronous, D: Domain> DigitalFn for Adapter<C, D> {}
