use rhdl_typenum::*;
use std::ops::{Add, Mul};

use crate::{bits_impl::bits_masked, signed, signed_bits_impl::signed_wrapped, Bits, SignedBits};

impl<N: BitWidth> Mul for Bits<N> {
    type Output = Bits<N>;
    fn mul(self, rhs: Bits<N>) -> Self::Output {
        bits_masked(self.val.wrapping_mul(rhs.val))
    }
}

impl<N: BitWidth> Mul for SignedBits<N> {
    type Output = SignedBits<N>;
    fn mul(self, rhs: SignedBits<N>) -> Self::Output {
        signed_wrapped(self.val.wrapping_mul(rhs.val))
    }
}

#[cfg(test)]
mod tests {
    use crate::{alias::*, bits};

    use super::*;

    #[test]
    fn test_mul() {
        let a = bits::<W32>(0x1234_5678);
        let b = bits::<W32>(0x8765_4321);
        let c = a * b;
        assert_eq!(
            c,
            bits::<W32>(0x1234_5678_u32.wrapping_mul(0x8765_4321) as u128)
        );
    }
}
