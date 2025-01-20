use std::ops::BitXor;
use std::ops::BitXorAssign;

use super::bits_impl::Bits;
use super::signed_bits_impl::SignedBits;
use super::BitWidth;

impl<N: BitWidth> BitXor<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn bitxor(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) ^ rhs
    }
}

impl<N: BitWidth> BitXor<u128> for Bits<N> {
    type Output = Self;
    fn bitxor(self, rhs: u128) -> Self::Output {
        self ^ Bits::<N>::from(rhs)
    }
}

impl<N: BitWidth> BitXorAssign<u128> for Bits<N> {
    fn bitxor_assign(&mut self, rhs: u128) {
        self.val ^= rhs;
    }
}

impl<N: BitWidth> BitXor<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn bitxor(self, rhs: SignedBits<N>) -> Self::Output {
        SignedBits::<N>::from(self) ^ rhs
    }
}

impl<N: BitWidth> BitXor<i128> for SignedBits<N> {
    type Output = Self;
    fn bitxor(self, rhs: i128) -> Self::Output {
        self ^ SignedBits::<N>::from(rhs)
    }
}

impl<N: BitWidth> BitXorAssign<i128> for SignedBits<N> {
    fn bitxor_assign(&mut self, rhs: i128) {
        self.val ^= rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rhdl_typenum::prelude::*;

    #[test]
    fn test_xor_bits() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits ^ bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits ^ 0b1111_0000;
        assert_eq!(result.val, 0b0010_1010_u128);
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = 0b1111_0000 ^ bits;
        assert_eq!(result.val, 0b0010_1010_u128);
        let mut bits: Bits<U128> = 0.into();
        bits = crate::rhdl_bits::test::set_bit(bits, 127, true);
        let result = bits ^ bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<U54> = 0b1101_1010.into();
        let result = bits ^ 1;
        assert_eq!(result.val, 0b1101_1011_u128);
        let result = 1 ^ bits;
        assert_eq!(result.val, 0b1101_1011_u128);
        let a: Bits<U12> = 0b1010_1010_1010.into();
        let b: Bits<U12> = 0b0110_0100_0000.into();
        let c: Bits<U12> = 0b1100_1110_1010.into();
        assert_eq!(a ^ b, c);
    }
}
