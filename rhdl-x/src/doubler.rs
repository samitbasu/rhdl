use rhdl::prelude::*;

#[kernel]
pub fn doubler<const N: usize>(_cr: ClockReset, i: Bits<N>) -> Bits<N> {
    note("i", i);
    let o = i << 1;
    note("o", o);
    i << 1
}
