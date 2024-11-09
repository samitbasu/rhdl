use rhdl::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct ZDriver<const N: usize> {}

#[derive(Debug, Clone, PartialEq, Copy, Digital)]
pub struct ZDriverIn<const N: usize> {
    pub bus: BitZ<N>,
    pub mask: Bits<N>,
    pub data: Bits<N>,
}

impl<const N: usize> SynchronousDQ for ZDriver<N> {
    type D = ();
    type Q = ();
}

#[derive(Debug, Clone, PartialEq, Copy, Digital)]
pub struct ZDriverOut<const N: usize> {
    pub bus: BitZ<N>,
    pub data: Bits<N>,
}

impl<const N: usize> SynchronousIO for ZDriver<N> {
    type I = ZDriverIn<N>;
    type O = ZDriverOut<N>;
    type Kernel = NoKernel3<ClockReset, Self::I, (), (Self::O, ())>;
}

#[kernel]
pub fn tristate<const N: usize>(bus: BitZ<N>, mask: Bits<N>, data: Bits<N>) -> (BitZ<N>, Bits<N>) {
    let mut out = bus;
    out.mask |= mask;
    out.value |= data & mask;
    (out, out.value)
}

impl<const N: usize> Synchronous for ZDriver<N> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(&self, _clock_reset: ClockReset, input: Self::I, _state: &mut Self::S) -> Self::O {
        trace("input", &input);
        let output = tristate(input.bus, input.mask, input.data);
        trace("output", &output);
        ZDriverOut {
            bus: output.0,
            data: output.1,
        }
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
