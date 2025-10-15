//! Support for extended negation (negation without wrap around)
//!
//! This provides the [XNeg] trait which defines
//! the `xneg` method.  This method performs a negation operation
//! that increases the bit width of the operand by one, thus avoiding
//! overflow issues that can occur with standard negation.
//!
//! ```
//! use rhdl_bits::*;
//! use rhdl_bits::alias::*;
//! let a: Bits<8> = 38.into();
//! let b = a.xneg(); // -38
//! assert_eq!(b.as_signed_bits::<9>(), s9(-38)); // b is a SignedDynBits with 9 bits
//! ```
//!
//! You can also use `xneg` on [DynBits]:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 38.into();
//! let a = a.dyn_bits();
//! let b = a.xneg(); // -38
//! assert_eq!(b.as_signed_bits::<9>(), s9(-38)); // b is a SignedDynBits with 9 bits
//! ```
//!
//! You can use `xneg` on [SignedBits] as well:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: SignedBits<8> = 38.into();
//! let b = a.xneg(); // -38
//! assert_eq!(b.as_signed_bits::<9>(), s9(-38)); // b is a SignedDynBits with 9 bits
//! ```
//!
//! You can also use `xneg` on [SignedDynBits]:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: SignedBits<8> = 38.into();
//! let a = a.dyn_bits();
//! let b = a.xneg(); // -38
//! assert_eq!(b.as_signed_bits::<9>(), s9(-38)); // b is a SignedDynBits with 9 bits
//! ```
//!
use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits};
use crate::{W, signed_dyn_bits::SignedDynBits};

/// Extended negation trait.  Represents a bit-preserving negation
/// of a signed value where the output size is one bit larger
/// than the input.  This is useful for avoiding overflow in
/// negation operations.
pub trait XNeg {
    /// The output type of the negation.
    type Output;
    /// Perform the extended negation operation.
    fn xneg(self) -> Self::Output;
}

impl<const N: usize> XNeg for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xneg(self) -> Self::Output {
        assert!(N < 128);
        let val = (self.val as i128).wrapping_neg();
        SignedDynBits { val, bits: N + 1 }
    }
}

impl XNeg for DynBits {
    type Output = SignedDynBits;
    fn xneg(self) -> Self::Output {
        assert!(self.bits < 128);
        let val = (self.val as i128).wrapping_neg();
        SignedDynBits {
            val,
            bits: self.bits + 1,
        }
    }
}

impl<const N: usize> XNeg for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xneg(self) -> Self::Output {
        self.dyn_bits().xneg()
    }
}

impl XNeg for SignedDynBits {
    type Output = SignedDynBits;
    fn xneg(self) -> Self::Output {
        assert!(self.bits < 128);
        SignedDynBits {
            val: self.val.wrapping_neg(),
            bits: self.bits + 1,
        }
        .wrapped()
    }
}
