//! # Bitwise NOT operation via `!`
//!
//! You can use the unary `!` operator to compute the bitwise NOT of a [Bits] or [SignedBits] values.
//!
//!```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 0b1101_1010.into();
//! let b = !a; // 0b0010_0101
//! assert_eq!(b, b8(0b0010_0101));
//!```
//!
//! We can also compute the bitwise NOT of [SignedBits]:
//!```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: SignedBits<8> = (-4).into(); // 0b1111_1100
//! let b = !a; // 0b0000_0011
//! assert_eq!(b, s8(3));
//!```
//!
use std::ops::Not;

use crate::bitwidth::W;

use super::{BitWidth, bits_impl::Bits, signed_bits_impl::SignedBits};

impl<const N: usize> Not for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn not(self) -> Self::Output {
        Self {
            val: !self.val & Self::mask().val,
        }
    }
}

impl<const N: usize> Not for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = Self;
    fn not(self) -> Self::Output {
        self.as_unsigned().not().as_signed()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_not_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = !bits;
        assert_eq!(result.val, 0b0010_0101_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = !bits;
        assert_eq!(result.val, !0_u128 - (1 << 127));
        let bits: Bits<14> = 0b1101_1010.into();
        let result = !bits;
        assert_eq!(result.val, 0b0011_1111_0010_0101_u128);
    }

    #[test]
    fn test_not_on_i8() {
        let x = -4_i8;
        let result = !x;
        assert_eq!(result, 3_i8);
    }

    #[test]
    fn test_not_on_signed_bits() {
        let x = SignedBits::<8>::from(-4);
        let result = !x;
        assert_eq!(result.val, 3_i128);
    }

    #[test]
    fn test_not_on_signed_does_not_overflow() {
        let x = SignedBits::<128>::from(-1);
        let result = !x;
        assert_eq!(result.val, 0_i128);
    }
}
