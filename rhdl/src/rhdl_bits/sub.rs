use std::ops::Sub;
use std::ops::SubAssign;

use super::bits_impl::bits_masked;
use super::bits_impl::Bits;
use super::signed_bits_impl::signed_wrapped;
use super::signed_bits_impl::SignedBits;
use super::BitWidth;

impl<N: BitWidth> Sub for Bits<N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        bits_masked(self.val.wrapping_sub(rhs.val))
    }
}

impl<N: BitWidth> Sub<u128> for Bits<N> {
    type Output = Self;
    fn sub(self, rhs: u128) -> Self::Output {
        assert!(rhs <= Self::mask().val);
        bits_masked(self.val.wrapping_sub(rhs))
    }
}

impl<N: BitWidth> Sub<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn sub(self, rhs: Bits<N>) -> Self::Output {
        assert!(self <= Bits::<N>::mask().val);
        bits_masked(self.wrapping_sub(rhs.val))
    }
}

impl<N: BitWidth> SubAssign for Bits<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<N: BitWidth> SubAssign<u128> for Bits<N> {
    fn sub_assign(&mut self, rhs: u128) {
        *self = *self - rhs;
    }
}

/* impl<N, M> Sub<Bits<M>> for Bits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<U1>,
    Sum<Maximum<N, M>, U1>: BitWidth,
{
    type Output = SignedBits<Sum<Maximum<N, M>, U1>>;
    fn sub(self, rhs: Bits<M>) -> Self::Output {
        signed((self.val as i128).wrapping_sub(rhs.val as i128))
    }
}
 */
// To subtract 2 N-bit unsigned, numbers, imagine they are 4 bit
// numbers.  Then each is in the range 0..15.  The difference is
// thus in the range -15..15.

impl<N: BitWidth> Sub for SignedBits<N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        signed_wrapped(self.val.wrapping_sub(rhs.val))
    }
}

impl<N: BitWidth> Sub<i128> for SignedBits<N> {
    type Output = Self;
    fn sub(self, rhs: i128) -> Self::Output {
        assert!(rhs >= Self::min_value());
        assert!(rhs <= Self::max_value());
        signed_wrapped(self.val.wrapping_sub(rhs))
    }
}

impl<N: BitWidth> Sub<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn sub(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self >= SignedBits::<N>::min_value());
        assert!(self <= SignedBits::<N>::max_value());
        signed_wrapped(self.wrapping_sub(rhs.val))
    }
}

impl<N: BitWidth> SubAssign for SignedBits<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<N: BitWidth> SubAssign<i128> for SignedBits<N> {
    fn sub_assign(&mut self, rhs: i128) {
        *self = *self - rhs;
    }
}

impl<N: BitWidth> SubAssign<SignedBits<N>> for i128 {
    fn sub_assign(&mut self, rhs: SignedBits<N>) {
        *self = *self - rhs.val;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rhdl_bits::bitwidth::*;

    #[test]
    fn test_sub_bits() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits - bits;
        assert_eq!(result.val, 0);
        let bits: SignedBits<U8> = 0b0101_1010.into();
        let result = bits - bits - bits;
        assert_eq!(result.val, -bits.val);
        let mut bits: Bits<U126> = 0.into();
        bits = crate::rhdl_bits::test::set_bit(bits, 125, true);
        let result = bits - bits;
        assert_eq!(result.val, 0);
        let bits: Bits<U54> = 0b1101_1010.into();
        let x = bits.val;
        let result = bits - 1;
        let bits_m_1: Bits<U54> = 0b1101_1001.into();
        assert_eq!(result, bits_m_1);
        let result = 1 - bits;
        // The 2s complement equivalent of 1 - x is 1 + (x::mask() - x) + 1
        // which is 2 + (x::mask() - x)
        assert_eq!(
            result,
            SignedBits::<U54>::from(1 - (x as i128)).as_unsigned()
        );
    }

    #[test]
    fn test_subassign_bits() {
        let mut bits: Bits<U8> = 0b1101_1010.into();
        let bits_m_1: Bits<U8> = 0b1101_1001.into();
        bits -= bits_m_1;
        assert_eq!(bits.val, 1_u128);
        let mut bits: Bits<U8> = 0b1101_1010.into();
        bits -= 1;
        assert_eq!(bits.val, 0b1101_1001_u128);
    }

    #[test]
    fn test_subtraction_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let signed_i: SignedBits<U8> = (i as i128).into();
                let signed_j: SignedBits<U8> = (j as i128).into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i.wrapping_sub(j) as i128;
                assert_eq!(signed_k.val, built_in_k);
            }
        }
    }

    #[test]
    fn test_subtraction_i128() {
        let min = SignedBits::<U128>::min_value();
        let max = SignedBits::<U128>::max_value();
        for i in [min, -1, 0, 1, max] {
            for j in [min, -1, 0, 1, max] {
                let signed_i: SignedBits<U128> = i.into();
                let signed_j: SignedBits<U128> = j.into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i.wrapping_sub(j);
                assert_eq!(signed_k.val, built_in_k);
            }
        }
    }

    #[test]
    fn test_subassign() {
        let mut x = SignedBits::<U8>::from(1);
        x -= -2;
        assert_eq!(x.val, 3);
    }
}
