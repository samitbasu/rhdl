use rhdl::prelude::*;

#[kernel]
pub fn doubler<const N: usize>(_cr: ClockReset, i: Bits<N>) -> Bits<N> {
    trace("i", &i);
    let o = i << 1;
    trace("o", &o);
    i << 1
}
