use std::ops::Add;
use std::ops::Sub;

use rhdl_typenum::*;

use crate::bits_impl::Bits;
use crate::signed;
use crate::signed_bits_impl::SignedBits;

impl<N, M> Sub<Bits<M>> for Bits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = SignedBits<Sum<Maximum<N, M>, W1>>;
    fn sub(self, rhs: Bits<M>) -> Self::Output {
        signed((self.val as i128).wrapping_sub(rhs.val as i128))
    }
}

// To subtract 2 N-bit unsigned, numbers, imagine they are 4 bit
// numbers.  Then each is in the range 0..15.  The difference is
// thus in the range -15..15.

impl<N, M> Sub<SignedBits<M>> for SignedBits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = SignedBits<Sum<Maximum<N, M>, W1>>;
    fn sub(self, rhs: SignedBits<M>) -> Self::Output {
        signed(self.val - rhs.val)
    }
}

impl<N> Sub<Bits<N>> for u128
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn sub(self, rhs: Bits<N>) -> Self::Output {
        assert!(self <= Bits::<N>::mask().val);
        signed(self as i128 - rhs.val as i128)
    }
}

impl<N> Sub<Bits<N>> for i32
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn sub(self, rhs: Bits<N>) -> Self::Output {
        signed(self as i128 - rhs.val as i128)
    }
}

impl<N> Sub<i32> for Bits<N>
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn sub(self, rhs: i32) -> Self::Output {
        signed(self.val as i128 - rhs as i128)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sub_bits() {
        let bits: Bits<W8> = 0b1101_1010.into();
        let result = bits - bits;
        assert_eq!(result.val, 0_i128);
        let bits: SignedBits<W8> = 0b0101_1010.into();
        let result = bits - bits - bits;
        assert_eq!(result.val, -bits.val);
        let mut bits: Bits<W126> = 0.into();
        bits = crate::test::set_bit(bits, 125, true);
        let result = bits - bits;
        assert_eq!(result.val, 0_i128);
        let bits: Bits<W54> = 0b1101_1010.into();
        let x = bits.val;
        let result = bits - 1;
        let bits_m_1: SignedBits<W55> = 0b1101_1001.into();
        assert_eq!(result, bits_m_1);
        let result: SignedBits<W55> = 1 - bits;
        // The 2s complement equivalent of 1 - x is 1 + (x::mask() - x) + 1
        // which is 2 + (x::mask() - x)
        assert_eq!(result.val, 1 - (x as i128),);
    }

    #[test]
    fn test_subassign_bits() {
        let mut bits: Bits<W8> = 0b1101_1010.into();
        let bits_m_1: Bits<W8> = 0b1101_1001.into();
        bits = (bits - bits_m_1).resize().as_unsigned();
        assert_eq!(bits.val, 1_u128);
        let mut bits: Bits<W8> = 0b1101_1010.into();
        bits = (bits - 1).resize().as_unsigned();
        assert_eq!(bits.val, 0b1101_1001_u128);
    }

    #[test]
    fn test_subtraction_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let signed_i: SignedBits<W8> = (i as i128).into();
                let signed_j: SignedBits<W8> = (j as i128).into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i as i128 - j as i128;
                assert_eq!(signed_k.val, built_in_k);
            }
        }
    }

    #[test]
    fn test_subtraction_i128() {
        let min = SignedBits::<W127>::min_value();
        let max = SignedBits::<W127>::max_value();
        for i in [min, -1, 0, 1, max] {
            for j in [min, -1, 0, 1, max] {
                let signed_i: SignedBits<W127> = i.into();
                let signed_j: SignedBits<W127> = j.into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i - j;
                assert_eq!(signed_k.val, built_in_k);
            }
        }
    }

    #[test]
    fn test_subassign() {
        let mut x = SignedBits::<W8>::from(1);
        x = (x - SignedBits::<W8>::from(-2)).resize();
        assert_eq!(x.val, 3);
    }
}
