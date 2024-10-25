use rhdl::prelude::*;

#[derive(Clone, Debug, Default, Synchronous)]
#[rhdl(kernel=doubler::<{N}>)]
#[rhdl(auto_dq)]
pub struct U<const N: usize> {}

impl<const N: usize> SynchronousIO for U<N> {
    type I = Bits<N>;
    type O = Bits<N>;
}

#[kernel]
pub fn doubler<const N: usize>(_cr: ClockReset, i: Bits<N>, _q: Q<N>) -> (Bits<N>, D<N>) {
    note("i", i);
    let o = i << 1;
    note("o", o);
    (i << 1, D::<{ N }>::init())
}
