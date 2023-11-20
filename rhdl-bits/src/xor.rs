use std::ops::BitXor;
use std::ops::BitXorAssign;

use crate::bits::Bits;
use crate::signed_bits::SignedBits;

impl<const N: usize> BitXor<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn bitxor(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) ^ rhs
    }
}

impl<const N: usize> BitXor<u128> for Bits<N> {
    type Output = Self;
    fn bitxor(self, rhs: u128) -> Self::Output {
        self ^ Bits::<N>::from(rhs)
    }
}

impl<const N: usize> BitXorAssign<u128> for Bits<N> {
    fn bitxor_assign(&mut self, rhs: u128) {
        *self = *self ^ rhs;
    }
}

impl<const N: usize> BitXor<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn bitxor(self, rhs: SignedBits<N>) -> Self::Output {
        SignedBits::<N>::from(self) ^ rhs
    }
}

impl<const N: usize> BitXor<i128> for SignedBits<N> {
    type Output = Self;
    fn bitxor(self, rhs: i128) -> Self::Output {
        self ^ SignedBits::<N>::from(rhs)
    }
}

impl<const N: usize> BitXorAssign<i128> for SignedBits<N> {
    fn bitxor_assign(&mut self, rhs: i128) {
        *self = *self ^ rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_xor_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits ^ bits;
        assert_eq!(result.0, 0_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits ^ 0b1111_0000;
        assert_eq!(result.0, 0b0010_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = 0b1111_0000 ^ bits;
        assert_eq!(result.0, 0b0010_1010_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = bits ^ bits;
        assert_eq!(result.0, 0_u128);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits ^ 1;
        assert_eq!(result.0, 0b1101_1011_u128);
        let result = 1 ^ bits;
        assert_eq!(result.0, 0b1101_1011_u128);
        let a: Bits<12> = 0b1010_1010_1010.into();
        let b: Bits<12> = 0b0110_0100_0000.into();
        let c: Bits<12> = 0b1100_1110_1010.into();
        assert_eq!(a ^ b, c);
    }
}
