use anyhow::bail;
use anyhow::Result;
use rhdl_bits::Bits;
use rhdl_core::Digital;
use rhdl_macro::{kernel, Digital};

use crate::circuit::no_link;
use crate::circuit::root_descriptor;
use crate::circuit::root_hdl;
use crate::circuit::CircuitLinkFn;
use crate::circuit::NoLink;
use crate::{
    circuit::{Circuit, CircuitDescriptor},
    clock::Clock,
    dff::{DFF, DFFI},
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

    type IO = ();

    type Q = (Bits<N>,);

    type D = (DFFI<Bits<N>>,);

    type C = ();

    type Update = counter<N>;
    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = counter::<N>;

    type Link = NoLink;
    const LINK: CircuitLinkFn<Self> = no_link::<Self>;

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

    fn descriptor(&self) -> CircuitDescriptor {
        let mut ret = root_descriptor(self);
        ret.children
            .insert("count".to_string(), self.count.descriptor());
        ret
    }

    fn as_hdl(&self, kind: crate::circuit::HDLKind) -> Result<crate::circuit::HDLDescriptor> {
        let mut ret = root_hdl(self, kind)?;
        ret.children
            .insert("count".to_string(), self.count.as_hdl(kind)?);
        Ok(ret)
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
