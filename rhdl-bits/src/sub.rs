use std::ops::Sub;
use std::ops::SubAssign;

use crate::bits_impl::Bits;
use crate::signed_bits_impl::SignedBits;

impl<const N: usize> Sub<Bits<N>> for Bits<N> {
    type Output = Self;
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(u128::wrapping_sub(self.0, rhs.0) & Self::mask().0)
    }
}

impl<const N: usize> Sub<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn sub(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) - rhs
    }
}

impl<const N: usize> Sub<u128> for Bits<N> {
    type Output = Self;
    fn sub(self, rhs: u128) -> Self::Output {
        self - Bits::<N>::from(rhs)
    }
}

impl<const N: usize> SubAssign<u128> for Bits<N> {
    fn sub_assign(&mut self, rhs: u128) {
        *self = *self - rhs;
    }
}

impl<const N: usize> SubAssign<Bits<N>> for Bits<N> {
    fn sub_assign(&mut self, rhs: Bits<N>) {
        *self = *self - rhs;
    }
}

impl<const N: usize> Sub<SignedBits<N>> for SignedBits<N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        (self.as_unsigned() - rhs.as_unsigned()).as_signed()
    }
}

impl<const N: usize> Sub<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn sub(self, rhs: SignedBits<N>) -> Self::Output {
        SignedBits::<N>::from(self) - rhs
    }
}

impl<const N: usize> Sub<i128> for SignedBits<N> {
    type Output = Self;
    fn sub(self, rhs: i128) -> Self::Output {
        self - SignedBits::<N>::from(rhs)
    }
}

impl<const N: usize> SubAssign<i128> for SignedBits<N> {
    fn sub_assign(&mut self, rhs: i128) {
        *self = *self - rhs;
    }
}

impl<const N: usize> SubAssign<SignedBits<N>> for SignedBits<N> {
    fn sub_assign(&mut self, rhs: SignedBits<N>) {
        *self = *self - rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::num::Wrapping;

    #[test]
    fn test_sub_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits - bits;
        assert_eq!(result.0, 0_u128);
        let x: std::num::Wrapping<u8> = Wrapping(0b1101_1010);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits - bits - bits;
        assert_eq!(Wrapping(result.0 as u8), x - x - x);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = bits - bits;
        assert_eq!(result.0, 0_u128);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits - 1;
        let bits_m_1: Bits<54> = 0b1101_1001.into();
        assert_eq!(result, bits_m_1);
        let result = 1 - bits;
        // The 2s complement equivalent of 1 - x is 1 + (x::mask() - x) + 1
        // which is 2 + (x::mask() - x)
        assert_eq!(result.0, 2 + (Bits::<54>::mask().0 - bits.0));
    }

    #[test]
    fn test_subassign_bits() {
        let mut bits: Bits<8> = 0b1101_1010.into();
        let bits_m_1: Bits<8> = 0b1101_1001.into();
        bits -= bits_m_1;
        assert_eq!(bits.0, 1_u128);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits -= 1;
        assert_eq!(bits.0, 0b1101_1001_u128);
    }

    #[test]
    fn test_subtraction_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let signed_i: SignedBits<8> = (i as i128).into();
                let signed_j: SignedBits<8> = (j as i128).into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i8::wrapping_sub(i, j) as i128;
                assert_eq!(signed_k.0, built_in_k);
            }
        }
    }

    #[test]
    fn test_subtraction_i128() {
        for i in [i128::MIN, -1, 0, 1, i128::MAX] {
            for j in [i128::MIN, -1, 0, 1, i128::MAX] {
                let signed_i: SignedBits<128> = i.into();
                let signed_j: SignedBits<128> = j.into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i.wrapping_sub(j);
                assert_eq!(signed_k.0, built_in_k);
            }
        }
    }

    #[test]
    fn test_subassign() {
        let mut x = SignedBits::<8>::from(1);
        x -= SignedBits::<8>::from(-2);
        assert_eq!(x.0, 3);
    }
}
