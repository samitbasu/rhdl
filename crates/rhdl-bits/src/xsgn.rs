//! Signed promotion without wrap around
//!
//! This module provides the [XSgn] trait, which defines the `xsgn` method for performing
//! sign promotion operations that preserve the bit width of the operand, resulting in an output
//! that is one bit larger than the input. This is particularly useful in scenarios
//! where overflow needs to be avoided.
//!
//! In general, when converting an unsigned value to a signed value, it is important to ensure that
//! the output type has enough bits to represent all possible values of the input type. The `xsgn`
//! method takes care of this by always producing a signed value with one additional bit of width.
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 200.into();
//! let b = a.xsgn(); // 200, which fits in 9 bits, but not in an 8 bit signed type!
//! assert_eq!(b.as_signed_bits::<9>(), s9(200)); // b is a SignedDynBits with 9 bits
//! ```
//!
//! You can also use `xsgn` on [DynBits]:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 200.into();
//! let a = a.dyn_bits();
//! let b = a.xsgn(); // 200, which fits in 9 bits
//! assert_eq!(b.as_signed_bits::<9>(), s9(200)); // b is a SignedDynBits with 9 bits
//! ```
//!
use super::{BitWidth, Bits, dyn_bits::DynBits};
use crate::{W, signed_dyn_bits::SignedDynBits};

/// Promote an unsigned value to a signed value with all bits
/// preserved.  The output size is one bit larger than the input size.
/// This is useful for avoiding overflow in sign promotion operations.
pub trait XSgn {
    /// The output type of the sign promotion.
    type Output;
    /// Perform the sign promotion operation.
    fn xsgn(self) -> Self::Output;
}

impl<const N: usize> XSgn for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;

    fn xsgn(self) -> Self::Output {
        assert!(N < 128);
        self.dyn_bits().xsgn()
    }
}

impl XSgn for DynBits {
    type Output = SignedDynBits;

    fn xsgn(self) -> Self::Output {
        assert!(self.bits < 128);
        SignedDynBits {
            val: self.val as i128,
            bits: self.bits + 1,
        }
        .wrapped()
    }
}
