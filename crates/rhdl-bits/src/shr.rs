use std::ops::Shr;
use std::ops::ShrAssign;

use crate::signed_dyn_bits::SignedDynBits;

use super::BitWidth;
use super::bits_impl::Bits;
use super::bits_impl::bits_masked;
use super::dyn_bits::DynBits;
use super::signed;
use super::signed_bits_impl::SignedBits;

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

impl<N> Shr<Bits<N>> for DynBits
where
    N: BitWidth,
{
    type Output = DynBits;
    fn shr(self, rhs: Bits<N>) -> Self::Output {
        assert!(rhs.raw() < self.bits as u128);
        DynBits {
            val: self.val.wrapping_shr(rhs.val as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl<N> Shr<DynBits> for Bits<N>
where
    N: BitWidth,
{
    type Output = Bits<N>;
    fn shr(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < N::BITS as u128);
        bits_masked(self.val.wrapping_shr(rhs.val as u32))
    }
}

impl Shr<DynBits> for DynBits {
    type Output = DynBits;
    fn shr(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < self.bits as u128);
        DynBits {
            val: self.val.wrapping_shr(rhs.val as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl Shr<u128> for DynBits {
    type Output = DynBits;
    fn shr(self, rhs: u128) -> Self::Output {
        assert!(rhs <= self.bits as u128);
        DynBits {
            val: self.val.wrapping_shr(rhs as u32),
            bits: self.bits,
        }
        .wrapped()
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

impl<N> Shr<DynBits> for SignedBits<N>
where
    N: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < N::BITS as u128);
        signed(i128::wrapping_shr(self.val, rhs.val as u32))
    }
}

impl<N> Shr<Bits<N>> for SignedDynBits
where
    N: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: Bits<N>) -> Self::Output {
        assert!(rhs.raw() <= self.bits as u128);
        SignedDynBits {
            val: self.val.wrapping_shr(rhs.val as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl Shr<DynBits> for SignedDynBits {
    type Output = SignedDynBits;
    fn shr(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() <= self.bits as u128);
        SignedDynBits {
            val: self.val.wrapping_shr(rhs.val as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl Shr<u128> for SignedDynBits {
    type Output = SignedDynBits;
    fn shr(self, rhs: u128) -> Self::Output {
        assert!(rhs <= self.bits as u128);
        SignedDynBits {
            val: self.val.wrapping_shr(rhs as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bitwidth::*;

    #[test]
    fn test_shr_bits() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits >> 4;
        assert_eq!(result.val, 0b0000_1101_u128);
        let bits: Bits<U16> = 0b1101_1010_0000_0000.into();
        let result = bits >> 8;
        assert_eq!(result.val, 0b0000_0000_1101_1010_u128);
        let shift: Bits<U4> = 8.into();
        let result = bits >> shift;
        assert_eq!(result.val, 0b0000_0000_1101_1010_u128);
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits >> 8;
        assert_eq!(result.val, 0);
        let shift: Bits<U8> = 4.into();
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
                let bits: SignedBits<U8> = (i as i128).into();
                let result = bits >> (shift as u128);
                assert_eq!(
                    result.val,
                    i128::wrapping_shr(i as i128, shift),
                    "i = {i:b}, shift = {shift}"
                );
                let shift_as_bits: Bits<U3> = (shift as u128).into();
                let result = bits >> shift_as_bits;
                assert_eq!(result.val, i128::wrapping_shr(i as i128, shift));
            }
        }
    }
}
