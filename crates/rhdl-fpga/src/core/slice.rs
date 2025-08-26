//! Functions to slice bits
//!
//! This module provides some synthesizable functions to
//! extract either the MSBs or LSBs of a bitvector.  The
//! code may not look efficient, but it optimizes away
//! when generating HDL.
use rhdl::prelude::*;

#[kernel]
/// Return the `N` LSBs of a bitvector of length `M`.  If
/// `N >= M`, then the upper bits will be zero filled.
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
/// Return the `N` MSBs of a bitvector of length `M`.  If
/// `N >= M`, then the lower bits of the output will be
/// zero filled.
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
        let h = msbs::<U16, U32>(n);
        assert_eq!(h, 0xDEAD);
        let l = lsbs::<U16, U32>(n);
        assert_eq!(l, 0xBEEF);
    }
}
