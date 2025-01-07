use std::ops::Shr;
use std::ops::ShrAssign;

use crate::bits_impl::bits_masked;
use crate::bits_impl::Bits;
use crate::signed;
use crate::signed_bits_impl::SignedBits;
use rhdl_typenum::*;

impl<N> Shr<u128> for Bits<N>
where
    N: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: u128) -> Self::Output {
        bits_masked(self.val.wrapping_shr(rhs as u32))
    }
}

impl<N> Shr<Bits<N>> for u128
where
    N: BitWidth,
{
    type Output = Bits<N>;
    fn shr(self, rhs: Bits<N>) -> Self::Output {
        bits_masked(self.wrapping_shr(rhs.val as u32))
    }
}

impl<N, M> Shr<Bits<M>> for Bits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: Bits<M>) -> Self::Output {
        bits_masked(u128::wrapping_shr(self.val, rhs.val as u32))
    }
}

impl<N, M> ShrAssign<Bits<M>> for Bits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    fn shr_assign(&mut self, rhs: Bits<M>) {
        *self = *self >> rhs;
    }
}

impl<N> ShrAssign<u128> for Bits<N>
where
    N: BitWidth,
{
    fn shr_assign(&mut self, rhs: u128) {
        *self = *self >> rhs;
    }
}

impl<N, M> Shr<Bits<M>> for SignedBits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: Bits<M>) -> Self::Output {
        signed(i128::wrapping_shr(self.val, rhs.val as u32))
    }
}

impl<N> Shr<u128> for SignedBits<N>
where
    N: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: u128) -> Self::Output {
        signed(self.val.wrapping_shr(rhs as u32))
    }
}

impl<N, M> ShrAssign<Bits<M>> for SignedBits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    fn shr_assign(&mut self, rhs: Bits<M>) {
        *self = *self >> rhs;
    }
}

impl<N> ShrAssign<u128> for SignedBits<N>
where
    N: BitWidth,
{
    fn shr_assign(&mut self, rhs: u128) {
        *self = *self >> rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shr_bits() {
        let bits: Bits<W8> = 0b1101_1010.into();
        let result = bits >> 4;
        assert_eq!(result.val, 0b0000_1101_u128);
        let bits: Bits<W16> = 0b1101_1010_0000_0000.into();
        let result = bits >> 8;
        assert_eq!(result.val, 0b0000_0000_1101_1010_u128);
        let shift: Bits<W4> = 8.into();
        let result = bits >> shift;
        assert_eq!(result.val, 0b0000_0000_1101_1010_u128);
        let bits: Bits<W8> = 0b1101_1010.into();
        let result = bits >> 8;
        assert_eq!(result.val, 0);
        let shift: Bits<W8> = 4.into();
        let result = 0b1101_1010_0000 >> shift;
        assert_eq!(result.val, 0b1101_1010u128);
    }

    #[test]
    fn test_shr_signed_i8_sane() {
        let i = -128_i8;
        let j = i >> 1;
        assert_eq!(j, -64_i8);
        let j = i8::wrapping_shr(i, 1);
        assert_eq!(j, -64_i8);
    }

    #[test]
    fn test_shr_signed() {
        for i in i8::MIN..i8::MAX {
            for shift in 0..8_u32 {
                let bits: SignedBits<W8> = (i as i128).into();
                let result = bits >> (shift as u128);
                assert_eq!(
                    result.val,
                    i128::wrapping_shr(i as i128, shift),
                    "i = {:b}, shift = {}",
                    i,
                    shift
                );
                let shift_as_bits: Bits<W3> = (shift as u128).into();
                let result = bits >> shift_as_bits;
                assert_eq!(result.val, i128::wrapping_shr(i as i128, shift));
            }
        }
    }
}
