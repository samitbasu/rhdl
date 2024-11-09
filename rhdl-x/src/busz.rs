use rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct U<const N: usize> {}

impl<const N: usize> Default for U<N> {
    fn default() -> Self {
        Self {}
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = BitZ<N>;
    type O = BitZ<N>;
    type Kernel = NoKernel3<ClockReset, BitZ<N>, (), (BitZ<N>, ())>;
}

impl<const N: usize> SynchronousDQ for U<N> {
    type D = ();
    type Q = ();
}

impl<const N: usize> Synchronous for U<N> {
    type S = BitZ<N>;

    fn init(&self) -> Self::S {
        BitZ::init()
    }

    fn sim(&self, _clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace("input", &input);
        *state = input;
        let output = state;
        let bitz = BitZ::<N> {
            value: input.value,
            mask: input.mask,
        };
        trace("bus", &bitz);
        trace("output", output);
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
