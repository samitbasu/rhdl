//! # Support for left shifting via `<<` and `<<=`
//!
//! Left shift operations are wrapping, meaning that bits shifted out on the left are discarded,
//! and zeros are shifted in on the right.  Use the `<<` operator as usual:
//!
//! Here are a simple example of left shifting a 8-bit unsigned value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 0b0000_1010.into();
//! let b = a << 4; // 0b1010_0000
//! assert_eq!(b, b8(0b1010_0000));
//! ```
//!
//! We can also left shift by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b0000_1010.into();
//! let shift: Bits<4> = 4.into();
//! let b = a << shift; // 0b1010_0000
//! assert_eq!(b, b8(0b1010_0000));
//! ```
//!
//! We can convert them to [DynBits] and left shift them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b0000_1010.into();
//! let a = a.dyn_bits();
//! let b = a << 4; // 0b1010_0000
//! assert_eq!(b.as_bits::<8>(), b8(0b1010_0000));
//! ```
//! We can also left shift [DynBits] by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b0000_1010.into();
//! let a = a.dyn_bits();
//! let shift: Bits<4> = 4.into();
//! let b = a << shift; // 0b1010_0000
//! assert_eq!(b.as_bits::<8>(), b8(0b1010_0000));
//! ```
//!
//! When working with signed values, remember to put parentheses around negative literals, and that
//! the shift amount is always unsigned:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a : SignedBits<8> = (-38).into();
//! let b = a << 1; // -76
//! assert_eq!(b, s8(-76));
//! ```
//!
//! You can also left shift [SignedBits] by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = (-38).into();
//! let shift: Bits<3> = 1.into();
//! let b = a << shift; // -76
//! assert_eq!(b, s8(-76));
//! ```
//!
//! You can convert them to [SignedDynBits] and left shift them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = (-38).into();
//! let a = a.dyn_bits();
//! let b = a << 1; // -76
//! assert_eq!(b.as_signed_bits::<8>(), s8(-76));
//! ```
//!
//! You can also left shift [SignedDynBits] by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = (-38).into();
//! let a = a.dyn_bits();
//! let shift: Bits<3> = 1.into();
//! let b = a << shift; // -76
//! assert_eq!(b.as_signed_bits::<8>(), s8(-76));
//! ```
//!
//! You can also use the `<<=` operator to left shift and assign in place:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let mut a: Bits<8> = 0b0000_1010.into();
//! a <<= 4; // a is now 0b1010_0000
//! assert_eq!(a, b8(0b1010_0000));
//! ```
use std::ops::Shl;
use std::ops::ShlAssign;

use crate::bitwidth::W;

use super::BitWidth;
use super::bits_impl::Bits;
use super::bits_impl::bits_masked;
use super::dyn_bits::DynBits;
use super::signed_bits_impl::SignedBits;
use super::signed_dyn_bits::SignedDynBits;

// Note! When reviewing this code remember that wrapping is not the same
// as rotate.

impl<const N: usize> Shl<u128> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: u128) -> Self::Output {
        bits_masked(self.raw().wrapping_shl(rhs as u32))
    }
}

impl<const N: usize> Shl<Bits<N>> for u128
where
    W<N>: BitWidth,
{
    type Output = Bits<N>;
    fn shl(self, rhs: Bits<N>) -> Self::Output {
        bits_masked(self.wrapping_shl(rhs.raw() as u32))
    }
}

impl<const N: usize, const M: usize> Shl<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: Bits<M>) -> Self::Output {
        bits_masked(u128::wrapping_shl(self.raw(), rhs.raw() as u32))
    }
}

impl<const N: usize> Shl<Bits<N>> for DynBits
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn shl(self, rhs: Bits<N>) -> Self::Output {
        assert!(rhs.raw() < self.bits as u128);
        DynBits {
            val: self.raw().wrapping_shl(rhs.raw() as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl<const N: usize> Shl<DynBits> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = Bits<N>;
    fn shl(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() <= { N } as u128);
        bits_masked(self.raw().wrapping_shl(rhs.raw() as u32))
    }
}

impl Shl<DynBits> for DynBits {
    type Output = DynBits;
    fn shl(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < self.bits as u128);
        DynBits {
            val: self.raw().wrapping_shl(rhs.raw() as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl Shl<u128> for DynBits {
    type Output = DynBits;
    fn shl(self, rhs: u128) -> Self::Output {
        assert!(rhs <= self.bits as u128);
        DynBits {
            val: self.raw().wrapping_shl(rhs as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl<const N: usize, const M: usize> ShlAssign<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    fn shl_assign(&mut self, rhs: Bits<M>) {
        *self = *self << rhs;
    }
}

impl<const N: usize> ShlAssign<u128> for Bits<N>
where
    W<N>: BitWidth,
{
    fn shl_assign(&mut self, rhs: u128) {
        *self = *self << rhs;
    }
}

impl<const N: usize, const M: usize> Shl<Bits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: Bits<M>) -> Self::Output {
        (self.as_unsigned() << rhs).as_signed()
    }
}

impl<const N: usize> Shl<u128> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: u128) -> Self::Output {
        self.as_unsigned().shl(rhs).as_signed()
    }
}

impl<const N: usize> Shl<DynBits> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < { N } as u128);
        self.as_unsigned().shl(rhs).as_signed()
    }
}

impl<const N: usize, const M: usize> ShlAssign<Bits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    fn shl_assign(&mut self, rhs: Bits<M>) {
        *self = *self << rhs;
    }
}

impl<const N: usize> ShlAssign<u128> for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn shl_assign(&mut self, rhs: u128) {
        *self = *self << rhs;
    }
}

impl<const N: usize> Shl<Bits<N>> for SignedDynBits
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shl(self, rhs: Bits<N>) -> Self::Output {
        assert!(rhs.raw() <= self.bits as u128);
        SignedDynBits {
            val: self.raw().wrapping_shl(rhs.raw() as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl Shl<DynBits> for SignedDynBits {
    type Output = SignedDynBits;
    fn shl(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() <= self.bits as u128);
        SignedDynBits {
            val: self.raw().wrapping_shl(rhs.raw() as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl Shl<u128> for SignedDynBits {
    type Output = SignedDynBits;
    fn shl(self, rhs: u128) -> Self::Output {
        assert!(rhs <= self.bits as u128);
        SignedDynBits {
            val: self.raw().wrapping_shl(rhs as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shl_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits << 4;
        assert_eq!(result.raw(), 0b1010_0000_u128);
        let bits: Bits<16> = 0b0000_0000_1101_1010.into();
        let result = bits << 8;
        assert_eq!(result.raw(), 0b1101_1010_0000_0000_u128);
        let shift: Bits<4> = 8.into();
        let result = bits << shift;
        assert_eq!(result.raw(), 0b1101_1010_0000_0000_u128);
    }

    #[test]
    fn test_shl_signed_bits() {
        let bits: SignedBits<8> = (-38).into();
        let result = bits << 1;
        assert_eq!(result.raw(), -76_i128);
        for shift in 0..8 {
            let bits: SignedBits<8> = (-38).into();
            let result = bits << shift;
            assert_eq!(result.raw(), ((-38_i128 << shift) as i8) as i128);
            let shift_as_bits: Bits<3> = shift.into();
            let result = bits << shift_as_bits;
            assert_eq!(result.raw(), ((-38_i128 << shift) as i8) as i128);
        }
    }

    #[test]
    fn test_shl_assign_signed_bits() {
        let mut bits: SignedBits<8> = (-38).into();
        bits <<= 1;
        assert_eq!(bits.raw(), -76_i128);
        for shift in 0..8 {
            let mut bits: SignedBits<8> = (-38).into();
            bits <<= shift;
            assert_eq!(bits.raw(), ((-38_i128 << shift) as i8) as i128);
            let shift_as_bits: Bits<3> = shift.into();
            let mut bits: SignedBits<8> = (-38).into();
            bits <<= shift_as_bits;
            assert_eq!(bits.raw(), ((-38_i128 << shift) as i8) as i128);
        }
    }
}
