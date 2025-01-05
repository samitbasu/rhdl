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
        bits(self.val + rhs)
    }
}

impl<N> Add<Bits<N>> for u128
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = Bits<Sum<N, W1>>;
    fn add(self, rhs: Bits<N>) -> Self::Output {
        bits(self + rhs.val)
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

impl<N: BitWidth> AddAssign<Bits<N>> for Bits<N> {
    fn add_assign(&mut self, rhs: Bits<N>) {
        self.val = self.val.wrapping_add(rhs.val);
    }
}

impl<N: BitWidth> AddAssign<u128> for Bits<N> {
    fn add_assign(&mut self, rhs: u128) {
        self.val = self.val.wrapping_add(rhs);
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

impl<N> AddAssign<i128> for SignedBits<N>
where
    N: BitWidth,
{
    fn add_assign(&mut self, rhs: i128) {
        self.val = self.val.wrapping_add(rhs);
    }
}

impl<N> AddAssign<SignedBits<N>> for SignedBits<N>
where
    N: BitWidth,
{
    fn add_assign(&mut self, rhs: SignedBits<N>) {
        self.val = self.val.wrapping_add(rhs.val);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rhdl_typenum::*;

    #[test]
    fn test_add_bits() {
        let bits: Bits<W8> = 0b1101_1010.into();
        let result = bits + bits;
        assert_eq!(result.val, 180_u128);
        let bits: Bits<W8> = 0b1101_1010.into();
        let result = bits + bits + bits;
        assert_eq!(result.val, 142_u128);
        let mut bits: Bits<W128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        bits += bits;
        let result = bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<W54> = 0b1101_1010.into();
        let result = bits + 1;
        assert_eq!(result.val, 219_u128);
        let result = 1 + bits;
        assert_eq!(result.val, 219_u128);
    }

    #[test]
    fn test_add_assign_bits() {
        let mut bits: Bits<W8> = 0b1101_1010.into();
        bits += bits;
        assert_eq!(bits.val, 436);
        let mut bits: Bits<W8> = 0b1101_1010.into();
        bits += bits;
        bits += bits;
        assert_eq!(bits.val, ((218 * 4) as u128) & 0xff);
        let mut bits: Bits<W128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        bits += bits;
        assert_eq!(bits.val, 0_u128);
        let mut bits: Bits<W54> = 0b1101_1010.into();
        bits += 1;
        assert_eq!(bits.val, 219_u128);
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let i_as_signed = SignedBits::<W8>::from(i as i128);
                let j_as_signed = SignedBits::<W8>::from(j as i128);
                let k_as_signed = i_as_signed + j_as_signed;
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
                let k_as_signed = i_as_signed + j_as_signed;
                let k = i64::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i128() {
        for i in [i128::MIN, -1, 0, 1, i128::MAX] {
            for j in [i128::MIN, -1, 0, 1, i128::MAX] {
                let i_as_signed = SignedBits::<W128>::from(i);
                let j_as_signed = SignedBits::<W128>::from(j);
                let mut k_as_signed = i_as_signed;
                k_as_signed += j_as_signed;
                let k = i128::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k);
            }
        }
    }

    #[test]
    fn test_add_assign_signed() {
        let mut x = SignedBits::<W8>::from(1);
        x += SignedBits::<W8>::from(-2);
        assert_eq!(x.val, -1);
        x += 7;
        assert_eq!(x.val, 6);
    }
}
