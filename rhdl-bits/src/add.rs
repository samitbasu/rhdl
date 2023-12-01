use std::ops::Add;
use std::ops::AddAssign;

use crate::bits_impl::Bits;
use crate::signed_bits_impl::SignedBits;

impl<const N: usize> Add<u128> for Bits<N> {
    type Output = Self;
    fn add(self, rhs: u128) -> Self::Output {
        self + Bits::<N>::from(rhs)
    }
}

impl<const N: usize> Add<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn add(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) + rhs
    }
}

impl<const N: usize> Add<Bits<N>> for Bits<N> {
    type Output = Self;
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(u128::wrapping_add(self.0, rhs.0) & Self::mask().0)
    }
}

impl<const N: usize> AddAssign<Bits<N>> for Bits<N> {
    fn add_assign(&mut self, rhs: Bits<N>) {
        *self = *self + rhs;
    }
}

impl<const N: usize> AddAssign<u128> for Bits<N> {
    fn add_assign(&mut self, rhs: u128) {
        *self = *self + rhs;
    }
}

impl<const N: usize> Add<i128> for SignedBits<N> {
    type Output = Self;
    fn add(self, rhs: i128) -> Self::Output {
        self + SignedBits::<N>::from(rhs)
    }
}

impl<const N: usize> Add<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn add(self, rhs: SignedBits<N>) -> Self::Output {
        SignedBits::<N>::from(self) + rhs
    }
}

impl<const N: usize> Add<SignedBits<N>> for SignedBits<N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        // Signed addition is the same as unsigned addition.
        // But the result needs to be reinterpreted as a signed value.
        (self.as_unsigned() + rhs.as_unsigned()).as_signed()
    }
}

impl<const N: usize> AddAssign<i128> for SignedBits<N> {
    fn add_assign(&mut self, rhs: i128) {
        *self = *self + rhs;
    }
}

impl<const N: usize> AddAssign<SignedBits<N>> for SignedBits<N> {
    fn add_assign(&mut self, rhs: SignedBits<N>) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits + bits;
        assert_eq!(result.0, 180_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits + bits + bits;
        assert_eq!(result.0, 142_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = bits + bits;
        assert_eq!(result.0, 0_u128);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits + 1;
        assert_eq!(result.0, 219_u128);
        let result = 1 + bits;
        assert_eq!(result.0, 219_u128);
    }

    #[test]
    fn test_add_assign_bits() {
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits += bits;
        assert_eq!(bits.0, 180_u128);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits += bits;
        bits += bits;
        assert_eq!(bits.0, ((218 * 4) as u128) & 0xff);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        bits += bits;
        assert_eq!(bits.0, 0_u128);
        let mut bits: Bits<54> = 0b1101_1010.into();
        bits += 1;
        assert_eq!(bits.0, 219_u128);
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let i_as_signed = SignedBits::<8>::from(i as i128);
                let j_as_signed = SignedBits::<8>::from(j as i128);
                let k_as_signed = i_as_signed + j_as_signed;
                let k = i8::wrapping_add(i, j);
                assert_eq!(k_as_signed.0, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i64() {
        for i in [i64::MIN, -1, 0, 1, i64::MAX] {
            for j in [i64::MIN, -1, 0, 1, i64::MAX] {
                let i_as_signed = SignedBits::<64>::from(i as i128);
                let j_as_signed = SignedBits::<64>::from(j as i128);
                let k_as_signed = i_as_signed + j_as_signed;
                let k = i64::wrapping_add(i, j);
                assert_eq!(k_as_signed.0, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i128() {
        for i in [i128::MIN, -1, 0, 1, i128::MAX] {
            for j in [i128::MIN, -1, 0, 1, i128::MAX] {
                let i_as_signed = SignedBits::<128>::from(i);
                let j_as_signed = SignedBits::<128>::from(j);
                let k_as_signed = i_as_signed + j_as_signed;
                let k = i128::wrapping_add(i, j);
                assert_eq!(k_as_signed.0, k);
            }
        }
    }

    #[test]
    fn test_add_assign_signed() {
        let mut x = SignedBits::<8>::from(1);
        x += SignedBits::<8>::from(-2);
        assert_eq!(x.0, -1);
        x += 7;
        assert_eq!(x.0, 6);
    }
}
