use std::ops::Shl;
use std::ops::ShlAssign;

use crate::bits_impl::bits_masked;
use crate::bits_impl::Bits;
use crate::signed_bits_impl::SignedBits;
use rhdl_typenum::*;

// Note! When reviewing this code remember that wrapping is not the same
// as rotate.

impl<N> Shl<u128> for Bits<N>
where
    N: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: u128) -> Self::Output {
        bits_masked(self.val.wrapping_shl(rhs as u32))
    }
}

impl<N> Shl<Bits<N>> for u128
where
    N: BitWidth,
{
    type Output = Bits<N>;
    fn shl(self, rhs: Bits<N>) -> Self::Output {
        bits_masked(self.wrapping_shl(rhs.val as u32))
    }
}

impl<N, M> Shl<Bits<M>> for Bits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: Bits<M>) -> Self::Output {
        bits_masked(u128::wrapping_shl(self.val, rhs.val as u32))
    }
}

impl<N, M> ShlAssign<Bits<M>> for Bits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    fn shl_assign(&mut self, rhs: Bits<M>) {
        *self = *self << rhs;
    }
}

impl<N> ShlAssign<u128> for Bits<N>
where
    N: BitWidth,
{
    fn shl_assign(&mut self, rhs: u128) {
        *self = *self << rhs;
    }
}

impl<N, M> Shl<Bits<M>> for SignedBits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: Bits<M>) -> Self::Output {
        (self.as_unsigned() << rhs).as_signed()
    }
}

impl<N> Shl<u128> for SignedBits<N>
where
    N: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: u128) -> Self::Output {
        self.as_unsigned().shl(rhs).as_signed()
    }
}

impl<N, M> ShlAssign<Bits<M>> for SignedBits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    fn shl_assign(&mut self, rhs: Bits<M>) {
        *self = *self << rhs;
    }
}

impl<N> ShlAssign<u128> for SignedBits<N>
where
    N: BitWidth,
{
    fn shl_assign(&mut self, rhs: u128) {
        *self = *self << rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shl_bits() {
        let bits: Bits<W8> = 0b1101_1010.into();
        let result = bits << 4;
        assert_eq!(result.val, 0b1010_0000_u128);
        let bits: Bits<W16> = 0b0000_0000_1101_1010.into();
        let result = bits << 8;
        assert_eq!(result.val, 0b1101_1010_0000_0000_u128);
        let shift: Bits<W4> = 8.into();
        let result = bits << shift;
        assert_eq!(result.val, 0b1101_1010_0000_0000_u128);
    }

    #[test]
    fn test_shl_signed_bits() {
        let bits: SignedBits<W8> = (-38).into();
        let result = bits << 1;
        assert_eq!(result.val, -76_i128);
        for shift in 0..8 {
            let bits: SignedBits<W8> = (-38).into();
            let result = bits << shift;
            assert_eq!(result.val, ((-38_i128 << shift) as i8) as i128);
            let shift_as_bits: Bits<W3> = shift.into();
            let result = bits << shift_as_bits;
            assert_eq!(result.val, ((-38_i128 << shift) as i8) as i128);
        }
    }

    #[test]
    fn test_shl_assign_signed_bits() {
        let mut bits: SignedBits<W8> = (-38).into();
        bits <<= 1;
        assert_eq!(bits.val, -76_i128);
        for shift in 0..8 {
            let mut bits: SignedBits<W8> = (-38).into();
            bits <<= shift;
            assert_eq!(bits.val, ((-38_i128 << shift) as i8) as i128);
            let shift_as_bits: Bits<W3> = shift.into();
            let mut bits: SignedBits<W8> = (-38).into();
            bits <<= shift_as_bits;
            assert_eq!(bits.val, ((-38_i128 << shift) as i8) as i128);
        }
    }
}
