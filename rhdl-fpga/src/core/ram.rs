use std::collections::HashMap;

use rhdl::prelude::*;

///
/// A simple block ram that stores 2^N values of type T.
/// It has two interfaces for read and writing, and supports
/// two different clocks.  This RAM is meant primarily for
/// FPGAs, as you can specify the initial contents of the
/// RAM.  For ASICs, you should probably assume the contents
/// of the RAM are random on initialization and implement
/// reset mechanism.
///
#[derive(Debug, Clone)]
pub struct U<T: Digital, W: Domain, R: Domain, const N: usize> {
    initial: Vec<T>,
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> U<T, W, R, N> {
    pub fn new(initial: impl IntoIterator<Item = T>) -> Self {
        let len = (1 << N) as usize;
        Self {
            initial: initial.into_iter().take(len).collect(),
            _w: Default::default(),
            _r: Default::default(),
        }
    }
}

/// For the input interface, we have write and read parts.  
/// These are on different clock domains, so we need to split
/// them out.

/// The read input lines contain the current address and the
/// clock signal.
#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub struct ReadI<const N: usize> {
    pub addr: Bits<N>,
    pub clock: Clock,
}

/// The write input lines control the write side of the RAM.
/// It contains the address to write to, the data, and the
/// enable and clock signal.
#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub struct WriteI<const N: usize, T: Digital> {
    pub addr: Bits<N>,
    pub data: T,
    pub enable: bool,
    pub clock: Clock,
}

#[derive(Copy, Clone, Debug, PartialEq, Digital, Timed)]
pub struct I<T: Digital, W: Domain, R: Domain, const N: usize> {
    pub write: Signal<WriteI<N, T>, W>,
    pub read: Signal<ReadI<N>, R>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitDQ for U<T, W, R, N> {
    type D = ();
    type Q = ();
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for U<T, W, R, N> {
    type I = I<T, W, R, N>;
    type O = Signal<T, R>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

#[derive(Debug, Clone)]
pub struct S<T: Digital, const N: usize> {
    write: WriteI<N, T>,
    read: ReadI<N>,
    contents: HashMap<Bits<N>, T>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> Circuit for U<T, W, R, N> {
    type S = S<T, N>;

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        todo!()
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        todo!()
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        todo!()
    }
}
