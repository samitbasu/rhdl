//! Extending Addition (XAdd) trait and implementations for [Bits], [SignedBits], [DynBits], and [SignedDynBits]
//!
//! The `XAdd` trait provides a way to perform addition operations that preserve the bit width of the operands,
//! resulting in an output that is one bit larger than the larger of the two inputs. This is particularly useful
//! in scenarios where overflow needs to be avoided.
//!
//! Because of limitations in stable Rust, the application of applying the `.xadd()` method to a [Bits] or [SignedBits]
//! value will always return a [DynBits] or [SignedDynBits] value, respectively.  This is because the output
//! size is not expressible at compile time using const generics.
//!
//!```
//!# use rhdl_bits::*;
//!# use rhdl_bits::alias::*;
//! let a: Bits<8> = 200.into();
//! let b: Bits<8> = 100.into();
//! let c = a.xadd(b); // 300, which fits in 9 bits
//! assert_eq!(c.as_bits::<9>(), b9(300)); // c is a DynBits with 9 bits
//!```
//!
//! Note that the sizes of the inputs do not need to be the same.  The output size is always
//! one bit larger than the larger of the two input sizes:
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 200.into();
//! let b: Bits<7> = 100.into();
//! let c = a.xadd(b); // 300, which fits in 9 bits
//! assert_eq!(c.as_bits::<9>(), b9(300)); // c is a DynBits with 9 bits
//! ```
//!
//! If the arguments are already [DynBits] the output will also be [DynBits]:
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 200.into();
//! # let b: Bits<7> = 100.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a.xadd(b); // 300, which fits in 9 bits
//! assert_eq!(c, b9(300).dyn_bits()); // c is a DynBits with 9 bits
//! ```
//!
//! The same applies to signed values:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: SignedBits<8> = (-50).into();
//! let b: SignedBits<7> = 30.into();
//! let c = a.xadd(b); // -20, which fits in 9 bits
//! assert_eq!(c.as_signed_bits::<9>(), s9(-20)); // c is a SignedDynBits with 9 bits
//!```
//!
//! If the arguments are already [SignedDynBits] the output will also be [SignedDynBits]:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: SignedBits<8> = (-50).into();
//! # let b: SignedBits<7> = 30.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a.xadd(b); // -20, which fits in 9 bits
//! assert_eq!(c, s9(-20).dyn_bits()); // c is a SignedDynBits with 9 bits
//!```
//!
//! Note that you cannot mix signed and unsigned types, any more than you can add `i8` and `u8` in normal Rust.

use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits, signed_dyn_bits::SignedDynBits};
use crate::bitwidth::W;

/// Extended addition trait.  Represents a bit-preserving addition
/// of two values (either signed or unsigned) where the output
/// size is one bit larger than the larger of the two inputs.
/// This is useful for avoiding overflow in addition operations.
pub trait XAdd<Rhs = Self> {
    /// The output type of the addition.
    type Output;
    /// Perform the extended addition operation.
    fn xadd(self, rhs: Rhs) -> Self::Output;
}

impl<const N: usize, const M: usize> XAdd<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = DynBits;
    fn xadd(self, rhs: Bits<M>) -> Self::Output {
        assert!(N.max(M) < 128);
        self.dyn_bits().xadd(rhs)
    }
}

impl<const N: usize> XAdd<DynBits> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xadd(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.bits.max(N) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: N.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize> XAdd<Bits<N>> for DynBits
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xadd(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits.max(N) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(N) + 1,
        }
        .wrapped()
    }
}

impl XAdd<DynBits> for DynBits {
    type Output = DynBits;
    fn xadd(self, rhs: DynBits) -> Self::Output {
        assert!(self.bits.max(rhs.bits) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize, const M: usize> XAdd<SignedBits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedBits<M>) -> Self::Output {
        self.dyn_bits().xadd(rhs)
    }
}

impl<const N: usize> XAdd<SignedDynBits> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedDynBits) -> Self::Output {
        assert!(rhs.bits.max(N) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: N.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize> XAdd<SignedBits<N>> for SignedDynBits
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits.max(N) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(N) + 1,
        }
        .wrapped()
    }
}

impl XAdd<SignedDynBits> for SignedDynBits {
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedDynBits) -> Self::Output {
        assert!(self.bits.max(rhs.bits) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bits, signed};

    #[test]
    fn test_xadd() {
        let a = bits::<32>(0x1234_5678);
        let b = bits::<32>(0x8765_4321);
        let c = a.xadd(b).as_bits();
        assert_eq!(c, bits::<33>(0x1234_5678 + 0x8765_4321));
        let a = signed::<32>(-456);
        let b = signed::<32>(123);
        let c = a.xadd(b).as_signed_bits();
        assert_eq!(c, signed::<33>(-456 + 123));
        let a = bits::<8>(255);
        let b = bits::<8>(255);
        let c = a.xadd(b).as_bits();
        assert_eq!(c, bits::<9>(255 + 255));
    }
}
