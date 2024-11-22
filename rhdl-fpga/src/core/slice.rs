use rhdl::prelude::*;

#[kernel]
pub fn lsbs<const N: usize, const M: usize>(n: Bits<M>) -> Bits<N> {
    let mut o = Bits::<N>::init();
    for i in 0..N {
        if n & (1 << i) != 0 {
            o |= 1 << i
        }
    }
    o
}
