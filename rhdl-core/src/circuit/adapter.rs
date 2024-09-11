use crate::{
    types::kind::Field, CircuitIO, Clock, Digital, Domain, Kind, Notable, NoteKey, NoteWriter,
    Reset, Signal, Synchronous, Timed,
};

use super::circuit_impl::CircuitUpdateFn;

// An adapter allows you to use a Synchronous circuit in an Asynchronous context.
#[derive(Clone)]
pub struct Adapter<C: Synchronous, D: Domain> {
    circuit: C,
    domain: std::marker::PhantomData<D>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct AdapterInput<I: Digital, D: Domain> {
    clock: Signal<Clock, D>,
    reset: Signal<Reset, D>,
    input: Signal<I, D>,
}

impl<I: Digital, D: Domain> Timed for AdapterInput<I, D> {}

impl<I: Digital, D: Domain> Notable for AdapterInput<I, D> {
    fn note(&self, key: impl NoteKey, writer: impl NoteWriter) {
        self.clock.note((key, "clock"), &writer);
        self.reset.note((key, "reset"), &writer);
        self.input.note((key, "input"), &writer);
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
    fn random() -> Self {
        Self {
            clock: Signal::random(),
            reset: Signal::random(),
            input: Signal::random(),
        }
    }
}

struct AdapterOutput<C: Synchronous, D: Domain> {
    output: Signal<C::O, D>,
}

impl<C: Synchronous, D: Domain> CircuitIO for Adapter<C, D> {
    type I = AdapterInput<C::I, D>;
    type O = AdapterOutput<C::O, D>;
}

impl<C: Synchronous, D: Domain> Circuit for Adapter<C, D> {
    type D = Signal<C::D, D>;
    type Q = Signal<C::Q, D>;

    type Update = Self;

    const UPDATE: CircuitUpdateFn<Self> = |_, _| unimplemented!();

    type S = Signal<C::S, D>;

    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O {
        self.circuit.sim(input, state, io)
    }

    fn name(&self) -> &'static str {
        self.circuit.name()
    }
}
