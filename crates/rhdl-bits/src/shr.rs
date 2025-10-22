//! # Support for right shifting via `>>` and `>>=`
//!
//! Right shift operations are wrapping, meaning that bits shifted out on the right are discarded,
//! and zeros are shifted in on the left if the value is unsigned.  If the value is signed, then sign bits are shifted in.
//! Use the `>>` operator as usual:
//!
//! Here are a simple example of right shifting a 8-bit unsigned value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 0b1101_1010.into();
//! let b = a >> 4; // 0b0000_1101
//! assert_eq!(b, b8(0b0000_1101));
//! ```
//!
//! We can also right shift by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b1101_1010.into();
//! let shift: Bits<4> = 4.into();
//! let b = a >> shift; // 0b0000_1101
//! assert_eq!(b, b8(0b0000_1101));
//! ```
//!
//! We can convert them to [DynBits] and right shift them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b1101_1010.into();
//! let a = a.dyn_bits();
//! let b = a >> 4; // 0b0000_1101
//! assert_eq!(b.as_bits::<8>(), b8(0b0000_1101));
//! ```
//! We can also right shift [DynBits] by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b1101_1010.into();
//! let a = a.dyn_bits();
//! let shift: Bits<4> = 4.into();
//! let b = a >> shift; // 0b0000_1101
//! assert_eq!(b.as_bits::<8>(), b8(0b0000_1101));
//! ```
//!
//! When working with signed values, remember to put parentheses around negative literals, and that
//! the shift amount is always unsigned:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a : SignedBits<8> = (-38).into();
//! let b = a >> 1; // -19
//! assert_eq!(b, s8(-19));
//! ```
//!
//! You can also right shift [SignedBits] by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = (-38).into();
//! let shift: Bits<3> = 1.into();
//! let b = a >> shift; // -19
//! assert_eq!(b, s8(-19));
//! ```
//!
//! You can convert them to [SignedDynBits] and right shift them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = (-38).into();
//! let a = a.dyn_bits();
//! let b = a >> 1; // -19
//! assert_eq!(b.as_signed_bits::<8>(), s8(-19));
//! ```
//!
//! You can also right shift [SignedDynBits] by a [Bits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = (-38).into();
//! let a = a.dyn_bits();
//! let shift: Bits<3> = 1.into();
//! let b = a >> shift; // -19
//! assert_eq!(b.as_signed_bits::<8>(), s8(-19));
//! ```
//!
//! You can also use the `>>=` operator to right shift and assign in place:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let mut a: Bits<8> = 0b1101_1010.into();
//! a >>= 4; // a is now 0b0000_1101
//! assert_eq!(a, b8(0b0000_1101));
//! ```

use std::ops::Shr;
use std::ops::ShrAssign;

use crate::W;
use crate::signed_dyn_bits::SignedDynBits;

use super::BitWidth;
use super::bits_impl::Bits;
use super::bits_impl::bits_masked;
use super::dyn_bits::DynBits;
use super::signed;
use super::signed_bits_impl::SignedBits;

impl<const N: usize> Shr<u128> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: u128) -> Self::Output {
        bits_masked(self.raw().wrapping_shr(rhs as u32))
    }
}

impl<const N: usize> Shr<Bits<N>> for u128
where
    W<N>: BitWidth,
{
    type Output = Bits<N>;
    fn shr(self, rhs: Bits<N>) -> Self::Output {
        bits_masked(self.wrapping_shr(rhs.raw() as u32))
    }
}

impl<const N: usize, const M: usize> Shr<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: Bits<M>) -> Self::Output {
        bits_masked(u128::wrapping_shr(self.raw(), rhs.raw() as u32))
    }
}

impl<const N: usize> Shr<Bits<N>> for DynBits
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn shr(self, rhs: Bits<N>) -> Self::Output {
        assert!(rhs.raw() < self.bits as u128);
        DynBits {
            val: self.raw().wrapping_shr(rhs.raw() as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl<const N: usize> Shr<DynBits> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = Bits<N>;
    fn shr(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < N as u128);
        bits_masked(self.raw().wrapping_shr(rhs.raw() as u32))
    }
}

impl Shr<DynBits> for DynBits {
    type Output = DynBits;
    fn shr(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < self.bits as u128);
        DynBits {
            val: self.raw().wrapping_shr(rhs.raw() as u32),
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
            val: self.raw().wrapping_shr(rhs as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

impl<const N: usize, const M: usize> ShrAssign<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    fn shr_assign(&mut self, rhs: Bits<M>) {
        *self = *self >> rhs;
    }
}

impl<const N: usize> ShrAssign<u128> for Bits<N>
where
    W<N>: BitWidth,
{
    fn shr_assign(&mut self, rhs: u128) {
        *self = *self >> rhs;
    }
}

impl<const N: usize, const M: usize> Shr<Bits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: Bits<M>) -> Self::Output {
        signed(i128::wrapping_shr(self.raw(), rhs.raw() as u32))
    }
}

impl<const N: usize> Shr<u128> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: u128) -> Self::Output {
        signed(self.raw().wrapping_shr(rhs as u32))
    }
}

impl<const N: usize, const M: usize> ShrAssign<Bits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    fn shr_assign(&mut self, rhs: Bits<M>) {
        *self = *self >> rhs;
    }
}

impl<const N: usize> ShrAssign<u128> for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn shr_assign(&mut self, rhs: u128) {
        *self = *self >> rhs;
    }
}

impl<const N: usize> Shr<DynBits> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.raw() < N as u128);
        signed(i128::wrapping_shr(self.raw(), rhs.raw() as u32))
    }
}

impl<const N: usize> Shr<Bits<N>> for SignedDynBits
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn shr(self, rhs: Bits<N>) -> Self::Output {
        assert!(rhs.raw() <= self.bits as u128);
        SignedDynBits {
            val: self.raw().wrapping_shr(rhs.raw() as u32),
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
            val: self.raw().wrapping_shr(rhs.raw() as u32),
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
            val: self.raw().wrapping_shr(rhs as u32),
            bits: self.bits,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shr_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits >> 4;
        assert_eq!(result.raw(), 0b0000_1101_u128);
        let bits: Bits<16> = 0b1101_1010_0000_0000.into();
        let result = bits >> 8;
        assert_eq!(result.raw(), 0b0000_0000_1101_1010_u128);
        let shift: Bits<4> = 8.into();
        let result = bits >> shift;
        assert_eq!(result.raw(), 0b0000_0000_1101_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits >> 8;
        assert_eq!(result.raw(), 0);
        let shift: Bits<8> = 4.into();
        let result = 0b1101_1010_0000 >> shift;
        assert_eq!(result.raw(), 0b1101_1010u128);
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
                let bits: SignedBits<8> = (i as i128).into();
                let result = bits >> (shift as u128);
                assert_eq!(
                    result.raw(),
                    i128::wrapping_shr(i as i128, shift),
                    "i = {i:b}, shift = {shift}"
                );
                let shift_as_bits: Bits<3> = (shift as u128).into();
                let result = bits >> shift_as_bits;
                assert_eq!(result.raw(), i128::wrapping_shr(i as i128, shift));
            }
        }
    }
}
