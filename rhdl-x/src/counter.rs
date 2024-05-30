use anyhow::Result;
use rhdl_bits::Bits;
use rhdl_core::note;
use rhdl_core::note_pop_path;
use rhdl_core::note_push_path;
use rhdl_core::root_descriptor;
use rhdl_core::root_hdl;
use rhdl_core::Circuit;
use rhdl_core::CircuitDescriptor;
use rhdl_core::CircuitIO;
use rhdl_core::Clk;
use rhdl_core::Clock;
use rhdl_core::HDLDescriptor;
use rhdl_core::HDLKind;
use rhdl_core::Kind;
use rhdl_core::Sig;
use rhdl_core::Timed;
use rhdl_macro::Timed;
use rhdl_macro::{kernel, Digital};

use crate::dff::DFF;

// Next a counter with an enable signal
#[derive(Default, Clone)]
pub struct Counter<C: Clock, const N: usize> {
    count: DFF<Bits<N>, C>,
}

#[derive(Debug, Clone, PartialEq, Copy, Timed)]
pub struct CounterI<C: Clock> {
    pub clock: Sig<Clk, C>,
    pub enable: Sig<bool, C>,
}

#[derive(Debug, Clone, PartialEq, Default, Copy, Timed)]
pub struct CounterQ<C: Clock, const N: usize> {
    pub count: <DFF<Bits<N>, C> as CircuitIO>::O,
}

#[derive(Debug, Clone, PartialEq, Timed, Default, Copy)]
pub struct CounterD<C: Clock, const N: usize> {
    pub count: <DFF<Bits<N>, C> as CircuitIO>::I,
}

impl<C: Clock, const N: usize> CircuitIO for Counter<C, N> {
    type I = CounterI<C>;
    type O = Bits<N>;
}

impl<const N: usize> Circuit for Counter<N> {
    type Q = CounterQ<N>;

    type D = CounterD<N>;

    type Z = ();

    type Update = counter<N>;
    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = counter::<N>;

    type S = (Self::Q, <DFF<Bits<N>> as Circuit>::S);

    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O {
        note("input", input);
        loop {
            let prev_state = *state;
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            note_push_path("count");
            state.0.count = self.count.sim(internal_inputs.count, &mut state.1, io);
            note_pop_path();
            if state == &prev_state {
                note("outputs", outputs);
                return outputs;
            }
        }
    }

    fn name(&self) -> &'static str {
        "Counter"
    }

    fn descriptor(&self) -> CircuitDescriptor {
        let mut ret = root_descriptor(self);
        ret.children
            .insert("count".to_string(), self.count.descriptor());
        ret
    }

    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor> {
        let mut ret = root_hdl(self, kind)?;
        ret.children
            .insert("count".to_string(), self.count.as_hdl(kind)?);
        Ok(ret)
    }
}

#[kernel]
pub fn counter<const N: usize>(i: CounterI, q: CounterQ<N>) -> (Bits<N>, CounterD<N>) {
    let mut d = CounterD::<N>::default();
    d.count.clock = i.clock;
    d.count.data = q.count;
    if i.enable {
        d.count.data = q.count + 1;
    }
    (q.count, d)
}
