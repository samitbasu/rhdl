use rhdl::prelude::*;

use crate::dff;

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub enable: bool,
}

#[derive(Clone, Debug, Synchronous)]
#[rhdl(kernel=counter::<{N}>)]
#[rhdl(auto_dq)]
pub struct U<const N: usize> {
    count: dff::U<Bits<N>>,
}

#[derive(Clone, Debug, PartialEq, Copy, Digital)]
struct MyD<const N: usize> {
    count: Bits<N>,
}

#[derive(Clone, Debug, PartialEq, Copy, Digital)]
struct MyQ<const N: usize> {
    count: Bits<N>,
}

impl<const N: usize> U<N> {
    pub fn new() -> Self {
        Self {
            count: dff::U::new(Bits::ZERO),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = I;
    type O = Bits<N>;
}

#[kernel]
pub fn counter<const N: usize>(reset: bool, i: I, q: Q<N>) -> (Bits<N>, D<N>) {
    let next_count = if i.enable { q.count + 1 } else { q.count };
    let output = q.count;
    if reset {
        (bits(0), D::<{ N }> { count: bits(0) })
    } else {
        (output, D::<{ N }> { count: next_count })
    }
}
