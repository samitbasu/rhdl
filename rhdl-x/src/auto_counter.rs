use crate::counter;
use rhdl::prelude::*;

#[derive(Clone, Debug, Default, Synchronous)]
#[rhdl(kernel=auto_counter::<{N}>)]
#[rhdl(auto_dq)]
pub struct U<const N: usize> {
    counter: counter::U<N>,
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = ();
    type O = Bits<N>;
}

#[kernel]
pub fn auto_counter<const N: usize>(_cr: ClockReset, _i: (), q: Q<N>) -> (Bits<N>, D<N>) {
    let mut d = D::<{ N }>::init();
    d.counter.enable = true;
    (q.counter, d)
}
