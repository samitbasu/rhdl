use rhdl::prelude::*;

use crate::bitz::Bitz;

#[derive(Debug, Clone)]
pub struct U<const N: usize> {}

impl<const N: usize> Default for U<N> {
    fn default() -> Self {
        Self {}
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = Bitz<N>;
    type O = Bitz<N>;
    type Kernel = NoKernel3<ClockReset, Bitz<N>, (), (Bitz<N>, ())>;
}

impl<const N: usize> SynchronousDQZ for U<N> {
    type D = ();
    type Q = ();
    type Z = ();
}

impl<const N: usize> Synchronous for U<N> {
    type S = Bitz<N>;

    fn sim(
        &self,
        _clock_reset: ClockReset,
        input: Self::I,
        state: &mut Self::S,
        _io: &mut Self::Z,
    ) -> Self::O {
        note("input", input);
        *state = input;
        let output = state;
        let bitz = BitZ::<N> {
            value: input.value,
            mask: input.mask,
        };
        note("bus", bitz);
        note("output", *output);
        input
    }

    fn description(&self) -> String {
        format!("BusZ {}", N)
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        panic!("BusZ does not support HDL generation")
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        panic!("BusZ does not support flow graph generation")
    }
}
