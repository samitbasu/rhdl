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

#[kernel]
pub fn msbs<const N: usize, const M: usize>(n: Bits<M>) -> Bits<N> {
    let mut o = Bits::<N>::init();
    for i in 0..N {
        if n & (1 << (M - N + i)) != 0 {
            o |= 1 << i
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msbs_works() {
        let n = 0xDEAD_BEEF_u128;
        let n = b32(n);
        let h = msbs::<16, 32>(n);
        assert_eq!(h, 0xDEAD);
        let l = lsbs::<16, 32>(n);
        assert_eq!(l, 0xBEEF);
    }
}
