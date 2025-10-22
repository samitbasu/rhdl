//! # Negation via the `-` operator
//!
//! You can use the unary `-` operator to negate a [SignedBits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: SignedBits<8> = 38.into();
//! let b = -a; // -38
//! assert_eq!(b, s8(-38));
//! ```
//!
//! You can also negate a [SignedDynBits] value:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: SignedBits<8> = (-38).into();
//! let a = a.dyn_bits();
//! let b = -a; // 38
//! assert_eq!(b.as_signed_bits::<8>(), s8(38));
//! ```
//!
//! Note that negation wraps around on overflow, just like normal Rust integer types:
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: SignedBits<8> = (-128).into();
//! let b = -a; // -128 (not 128, which is out of range for 8-bit signed)
//! assert_eq!(b, s8(-128));
//! ```
//!
//! If you want to perform negation without wrapping, use the [XNeg](crate::xneg::XNeg) operator instead.
use crate::bitwidth::W;

use super::signed_bits_impl::signed_wrapped;

use super::signed_dyn_bits::SignedDynBits;
use super::{BitWidth, signed_bits_impl::SignedBits};
use std::ops::Neg;

impl<const N: usize> Neg for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedBits<N>;
    fn neg(self) -> Self::Output {
        signed_wrapped(-self.raw())
    }
}

impl Neg for SignedDynBits {
    type Output = SignedDynBits;
    fn neg(self) -> Self::Output {
        SignedDynBits {
            val: -self.val,
            bits: self.bits,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod test {
    use crate::signed_bits_impl::SignedBits;

    #[test]
    fn test_neg_wrapping() {
        let x = i8::MIN;
        let y = x.wrapping_neg();
        assert_eq!(x, y);
    }

    #[test]
    fn test_neg_operator() {
        for i in i8::MIN..i8::MAX {
            let x = i;
            let y = x.wrapping_neg() as i16;
            let x_signed = SignedBits::<8>::from(x as i128);
            let y_signed = -x_signed;
            assert_eq!(y_signed.raw(), y as i128);
        }
    }
}
