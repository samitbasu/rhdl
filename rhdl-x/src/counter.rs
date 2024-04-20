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
use rhdl_core::HDLDescriptor;
use rhdl_core::HDLKind;
use rhdl_macro::{kernel, Digital};

use crate::{clock::Clock, dff::DFF};

// Next a counter with an enable signal
#[derive(Default, Clone)]
pub struct Counter<const N: usize> {
    count: DFF<Bits<N>>,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct CounterI {
    pub clock: Clock,
    pub enable: bool,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct CounterQ<const N: usize> {
    pub count: <DFF<Bits<N>> as CircuitIO>::O,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct CounterD<const N: usize> {
    pub count: <DFF<Bits<N>> as CircuitIO>::I,
}

impl<const N: usize> CircuitIO for Counter<N> {
    type I = CounterI;
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
