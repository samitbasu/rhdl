use rhdl::prelude::*;

#[kernel]
pub fn lsbs<N: BitWidth, M: BitWidth>(n: Bits<M>) -> Bits<N> {
    let mut o = bits(0);
    for i in 0..N::BITS {
        if n & (1 << i) != 0 {
            o |= 1 << i
        }
    }
    o
}

#[kernel]
pub fn msbs<N: BitWidth, M: BitWidth>(n: Bits<M>) -> Bits<N> {
    let mut o = bits(0);
    for i in 0..N::BITS {
        if n & (1 << (M::BITS - N::BITS + i)) != 0 {
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
        let h = msbs::<W16, W32>(n);
        assert_eq!(h, 0xDEAD);
        let l = lsbs::<W16, W32>(n);
        assert_eq!(l, 0xBEEF);
    }
}
