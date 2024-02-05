use rhdl_bits::{bits, Bits};
use rhdl_macro::{kernel, Digital};

use crate::{circuit::Circuit, clock::Clock, constant::Constant, dff::DFF};

// Build a strobe
#[derive(Clone)]
pub struct Strobe<const N: usize> {
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Strobe<N> {
    pub fn new(param: Bits<N>) -> Self {
        Self {
            threshold: param.into(),
            counter: DFF::default(),
        }
    }
}

// Can we autoderive something like:
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeQ<const N: usize> {
    threshold: <Constant<Bits<N>> as Circuit>::O,
    counter: <DFF<Bits<N>> as Circuit>::O,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeD<const N: usize> {
    threshold: <Constant<Bits<N>> as Circuit>::I,
    counter: <DFF<Bits<N>> as Circuit>::I,
}

impl<const N: usize>
    From<(
        <Constant<Bits<N>> as Circuit>::O,
        <DFF<Bits<N>> as Circuit>::O,
    )> for StrobeQ<N>
{
    fn from(
        (threshold, counter): (
            <Constant<Bits<N>> as Circuit>::O,
            <DFF<Bits<N>> as Circuit>::O,
        ),
    ) -> Self {
        Self { threshold, counter }
    }
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeI<const N: usize> {
    pub clock: Clock,
    pub enable: bool,
}

impl<const N: usize> Circuit for Strobe<N> {
    type I = StrobeI<N>;

    type O = bool;

    type Q = StrobeQ<N>;

    type D = StrobeD<N>;

    type S = (
        Self::Q,
        <Constant<Bits<N>> as Circuit>::S,
        <DFF<Bits<N>> as Circuit>::S,
    );

    type Update = strobe<N>;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = strobe::<N>;

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        loop {
            let prev_state = state.clone();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            state.0.threshold = self.threshold.sim(internal_inputs.threshold, &mut state.1);
            state.0.counter = self.counter.sim(internal_inputs.counter, &mut state.2);
            if state == &prev_state {
                return outputs;
            }
        }
    }

    fn name(&self) -> &'static str {
        "Strobe"
    }

    fn components(&self) -> impl Iterator<Item = (String, crate::circuit::CircuitDescriptor)> {
        [
            ("threshold".to_string(), self.threshold.descriptor()),
            ("counter".to_string(), self.counter.descriptor()),
        ]
        .into_iter()
    }

    fn child_verilog(self) -> impl Iterator<Item = anyhow::Result<String>> {
        [self.threshold.verilog(), self.counter.verilog()].into_iter()
    }
}

#[kernel]
pub fn strobe<const N: usize>(i: StrobeI<N>, q: StrobeQ<N>) -> (bool, StrobeD<N>) {
    let mut d = StrobeD::<N>::default();
    d.counter.clock = i.clock;
    let counter_next = if i.enable { q.counter + 1 } else { q.counter };
    let strobe = i.enable & (q.counter == q.threshold);
    let counter_next = if strobe {
        bits::<{ N }>(1)
    } else {
        counter_next
    };
    d.counter.data = counter_next;
    (strobe, d)
}
