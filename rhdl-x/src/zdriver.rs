use rhdl::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct ZDriver<const N: usize> {}

#[derive(Debug, Clone, PartialEq, Copy, Digital)]
pub struct ZDriverIn<const N: usize> {
    pub mask: Bits<N>,
    pub data: Bits<N>,
}

impl<const N: usize> SynchronousDQZ for ZDriver<N> {
    type D = ();
    type Q = ();
    type Z = BitZ<N>;
}

impl<const N: usize> SynchronousIO for ZDriver<N> {
    type I = ZDriverIn<N>;
    type O = Bits<N>;
    type Kernel = NoKernel3<ClockReset, Self::I, (), (Self::O, ())>;
}

impl<const N: usize> Synchronous for ZDriver<N> {
    type S = ();

    fn sim(
        &self,
        _clock_reset: ClockReset,
        input: Self::I,
        _state: &mut Self::S,
        io: &mut Self::Z,
    ) -> Self::O {
        note("input", input);
        io.value = input.data;
        io.mask = input.mask;
        note("bus", *io);
        let output = input.data;
        note("output", output);
        output
    }

    fn description(&self) -> String {
        format!("ZDriver of width {N}")
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        todo!()
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        todo!()
    }
}
