use anyhow::bail;
use anyhow::Result;
use rhdl_bits::Bits;
use rhdl_core::Digital;
use rhdl_macro::{kernel, Digital};

use crate::{
    circuit::{Circuit, CircuitDescriptor},
    clock::Clock,
    dff::{DFF, DFFI},
    translator::Translator,
};

// Next a counter with an enable signal
#[derive(Default, Clone)]
pub struct Counter<const N: usize> {
    count: DFF<Bits<N>>,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct CounterI<const N: usize> {
    pub clock: Clock,
    pub enable: bool,
}

impl<const N: usize> Circuit for Counter<N> {
    type I = CounterI<N>;

    type O = Bits<N>;

    type Q = (Bits<N>,);

    type D = (DFFI<Bits<N>>,);

    type Update = counter<N>;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = counter::<N>;

    type S = (Self::Q, <DFF<Bits<N>> as Circuit>::S);

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        loop {
            let prev_state = state.clone();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            let o0 = self.count.sim(internal_inputs.0, &mut state.1);
            state.0 = (o0,);
            if state == &prev_state {
                return outputs;
            }
        }
    }

    fn name(&self) -> &'static str {
        "Counter"
    }

    fn components(&self) -> impl Iterator<Item = (String, CircuitDescriptor)> {
        std::iter::once(("count".to_string(), self.count.descriptor()))
    }

    fn translate<T: Translator>(&self, name: &str, translator: &mut T) -> Result<()> {
        translator.translate(name, self)?;
        translator.push()?;
        self.count.translate("count".into(), translator)?;
        translator.pop()?;
        Ok(())
    }
}

#[kernel]
pub fn counter<const N: usize>(
    i: CounterI<N>,
    (count_q,): (Bits<N>,),
) -> (Bits<N>, (DFFI<Bits<N>>,)) {
    let count_next = if i.enable { count_q + 1 } else { count_q };
    (
        count_q,
        (DFFI::<Bits<{ N }>> {
            clock: i.clock,
            data: count_next,
        },),
    )
}
