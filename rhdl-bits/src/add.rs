use std::ops::Add;

use rhdl_typenum::BitWidth;
use rhdl_typenum::Max;
use rhdl_typenum::Maximum;
use rhdl_typenum::Sum;
use rhdl_typenum::W1;

use crate::bits;
use crate::bits_impl::bits_masked;
use crate::bits_impl::Bits;
use crate::signed;
use crate::signed_bits_impl::SignedBits;

impl<N> Add<u128> for Bits<N>
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = Bits<Sum<N, W1>>;
    fn add(self, rhs: u128) -> Self::Output {
        assert!(rhs <= Self::MASK.val);
        bits(self.val.wrapping_add(rhs))
    }
}

impl<N> Add<Bits<N>> for u128
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = Bits<Sum<N, W1>>;
    fn add(self, rhs: Bits<N>) -> Self::Output {
        assert!(self <= Bits::<N>::MASK.val);
        bits(self.wrapping_add(rhs.val))
    }
}

impl<N, M> Add<Bits<M>> for Bits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = Bits<Sum<Maximum<N, M>, W1>>;
    fn add(self, rhs: Bits<M>) -> Self::Output {
        bits_masked(u128::wrapping_add(self.val, rhs.val))
    }
}

impl<N> Add<i128> for SignedBits<N>
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn add(self, rhs: i128) -> Self::Output {
        signed(self.val + rhs)
    }
}

impl<N> Add<SignedBits<N>> for i128
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn add(self, rhs: SignedBits<N>) -> Self::Output {
        signed(self + rhs.val)
    }
}

impl<N, M> Add<SignedBits<M>> for SignedBits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = SignedBits<Sum<Maximum<N, M>, W1>>;
    fn add(self, rhs: SignedBits<M>) -> Self::Output {
        signed(self.val.wrapping_add(rhs.val))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rhdl_typenum::*;

    #[test]
    fn test_add_bits() {
        let bits: Bits<W8> = 0b1101_1010.into();
        let b_val = bits.val;
        let result = bits + bits;
        assert_eq!(result.val, 2 * b_val);
        let result = bits + bits + bits;
        assert_eq!(result.val, 3 * b_val);
        let mut bits: Bits<W124> = 0.into();
        bits = crate::test::set_bit(bits, 123, true);
        bits = (bits + bits).resize();
        let result = bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<W54> = 0b1101_1010.into();
        let result = bits + 1;
        assert_eq!(result.val, 219_u128);
        let result = 1 + bits;
        assert_eq!(result.val, 219_u128);
        let mut bits: Bits<W8> = 0b1101_1010.into();
        bits = (bits + bits).resize();
        assert_eq!(bits.val, 180);
        let mut bits: Bits<W8> = 0b1101_1010.into();
        bits = (bits + bits).resize();
        bits = (bits + bits).resize();
        assert_eq!(bits.val, ((218 * 4) as u128) & 0xff);
        let mut bits: Bits<W126> = 0.into();
        bits = crate::test::set_bit(bits, 125, true);
        bits = (bits + bits).resize();
        assert_eq!(bits.val, 0_u128);
        let mut bits: Bits<W54> = 0b1101_1010.into();
        bits = (bits + 1).resize();
        assert_eq!(bits.val, 219_u128);
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let i_as_signed = SignedBits::<W8>::from(i as i128);
                let j_as_signed = SignedBits::<W8>::from(j as i128);
                let k_as_signed: SignedBits<W8> = (i_as_signed + j_as_signed).resize();
                let k = i8::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i64() {
        for i in [i64::MIN, -1, 0, 1, i64::MAX] {
            for j in [i64::MIN, -1, 0, 1, i64::MAX] {
                let i_as_signed = SignedBits::<W64>::from(i as i128);
                let j_as_signed = SignedBits::<W64>::from(j as i128);
                let k_as_signed = (i_as_signed + j_as_signed).resize::<W64>();
                let k = i64::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_range() {
        eprintln!(
            "signed range {}..{}",
            SignedBits::<W9>::min_value(),
            SignedBits::<W9>::max_value()
        );
        assert_eq!(SignedBits::<W9>::min_value(), -256);
        assert_eq!(SignedBits::<W9>::max_value(), 255);
    }

    #[test]
    fn test_add_assign_signed() {
        let mut x = SignedBits::<W8>::from(1);
        x = (x + SignedBits::<W8>::from(-2)).resize();
        assert_eq!(x.val, -1);
        let z = x + 7;
        x = z.resize();
        assert_eq!(x.val, 6);
    }
}
