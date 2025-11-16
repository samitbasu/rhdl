//! # Support for multiplication via `*`
//!
//! By default, all multiply operations are wrapping.  Use the `*` operator as usual.  Wrapping multiplication is
//! what you experience with, e.g. `u32` in normal Rust.  When two `u32` values are multiplied, the result is also a `u32`,
//! and if the mathematical result does not fit in 32 bits, only the lower 32 bits are kept (when using wrapping multiplication).
//!
//! Here are a simple example of multiplying 2 8-bit unsigned values:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 20.into();
//! let b: Bits<8> = 10.into();
//! let c = a * b; // 200
//! assert_eq!(c, b8(200));
//! ```
//!
//! We can convert them to [DynBits] and multiply them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 20.into();
//! # let b: Bits<8> = 10.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a * b; // 200
//! assert_eq!(c.as_bits::<8>(), b8(200));
//! ```
//!
//! When working with signed values, remember to put parentheses around negative literals:
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a : SignedBits<8> = 12.into();
//! let b : SignedBits<8> = (-10).into();
//! let c = a * b; // -120
//! assert_eq!(c, s8(-120));
//! ```
//!
//! Finally, here is a signed dynamic example:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = 12.into();
//! # let b : SignedBits<8> = (-10).into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a * b; // -120
//! assert_eq!(c.as_signed_bits::<8>(), s8(-120));
//! ```
//!
//! Note that you cannot mix unsigned and signed types, any more than you can multiply `i8` and `u8` in normal Rust.
//!

use std::ops::Mul;

use super::dyn_bits::DynBits;
use super::signed_dyn_bits::SignedDynBits;
use super::{BitWidth, Bits, SignedBits, bits_impl::bits_masked, signed_bits_impl::signed_wrapped};

impl_binop!(Mul, mul, u128::wrapping_mul);
impl_signed_binop!(Mul, mul, i128::wrapping_mul);

#[cfg(test)]
mod tests {
    use crate::bits;
    use crate::bits_impl::bits_masked;

    #[test]
    fn test_muls() {
        for i in 0..=255 {
            for j in 0..=255 {
                test_binop!(*, u128::wrapping_mul, i, j);
            }
        }
    }

    #[test]
    fn test_mul() {
        let a = bits::<32>(0x1234_5678);
        let b = bits::<32>(0x8765_4321);
        let c = a * b;
        assert_eq!(
            c,
            bits::<32>(0x1234_5678_u32.wrapping_mul(0x8765_4321) as u128)
        );
    }
}
