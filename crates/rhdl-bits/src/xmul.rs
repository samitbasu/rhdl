//! Extended multiplication (multiplication without wrap around)
//!
//! This module provides the [XMul] trait, which defines the `xmul` method for performing
//! multiplication operations that preserve the bit width of the operands, resulting in an output
//! that is the sum of the sizes of the two inputs. This is particularly useful in scenarios
//! where overflow needs to be avoided.
//!
//! Because of limitations in stable Rust, the application of applying the `.xmul()` method to a [Bits] or [SignedBits]
//! value will always return a [DynBits] or [SignedDynBits] value, respectively.  This is because the output
//! size is not expressible at compile time using const generics.
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 200.into();
//! let b: Bits<8> = 100.into();
//! let c = a.xmul(b); // 20000, which fits in 16 bits
//! assert_eq!(c.as_bits::<16>(), b16(20000)); // c is a DynBits with 16 bits
//! ```
//!
//! Note that the sizes of the inputs do not need to be the same.  The output size is always
//! the sum of the sizes of the two inputs:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 200.into();
//! let b: Bits<4> = 10.into();
//! let c = a.xmul(b); // 2000, which fits in 12 bits
//! assert_eq!(c.as_bits::<12>(), b12(2000)); // c is a DynBits with 12 bits
//! ```
//! If the arguments are already [DynBits] the output will also be [DynBits]:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 200.into();
//! # let b: Bits<4> = 10.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a.xmul(b); // 2000, which fits in 12 bits
//! assert_eq!(c, b12(2000).dyn_bits()); // c is a DynBits with 12 bits
//! ```
//!
//! The same applies to signed values:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: SignedBits<8> = (-50).into();
//! let b: SignedBits<4> = 3.into();
//! let c = a.xmul(b); // -150, which fits in 12 bits
//! assert_eq!(c.as_signed_bits::<12>(), s12(-150)); // c is a SignedDynBits with 12 bits
//!```
//!
//! If the arguments are already [SignedDynBits] the output will also be [SignedDynBits]:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: SignedBits<8> = (-50).into();
//! # let b: SignedBits<4> = 3.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a.xmul(b); // -150, which fits in 12 bits
//! assert_eq!(c, s12(-150).dyn_bits()); // c is a SignedDynBits with 12 bits
//! ```
//!
//! Note that the maximum supported bit width for [Bits], [SignedBits], [DynBits] and [SignedDynBits] is 128 bits.
//! Therefore, the sum of the sizes of the two inputs to the `xmul` method must not exceed 128 bits.
//! Attempting to multiply two values whose combined bit width exceeds this limit will result in a panic at runtime.
//!
//! Also, note that while multipliers are generally synthesizable, exactly how they are implemented can vary
//! greatly between synthesis tools and target technologies. Therefore, it's advisable to consult the documentation
//! for your specific toolchain and target architecture to understand the implications of using extended multiplication.
//!
use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits, signed_dyn_bits::SignedDynBits};
use crate::bitwidth::W;

/// Extended multiplication trait.  Represents a bit-preserving multiplication
/// of two values (either signed or unsigned) where the output
/// size is the sum of the sizes of the two inputs.
/// This is useful for avoiding overflow in multiplication operations.
pub trait XMul<Rhs = Self> {
    /// The output type of the multiplication.
    type Output;
    /// Perform the extended multiplication operation.
    fn xmul(self, rhs: Rhs) -> Self::Output;
}

impl<const N: usize, const M: usize> XMul<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: Bits<M>) -> Self::Output {
        self.dyn_bits().xmul(rhs)
    }
}

impl<const N: usize> XMul<Bits<N>> for DynBits
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits + N <= 128);
        DynBits {
            val: self.val.wrapping_mul(rhs.raw()),
            bits: self.bits + N,
        }
        .wrapped()
    }
}

impl<const N: usize> XMul<DynBits> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: DynBits) -> Self::Output {
        assert!(N + rhs.bits <= 128);
        DynBits {
            val: self.raw().wrapping_mul(rhs.val),
            bits: N + rhs.bits,
        }
        .wrapped()
    }
}

impl XMul<DynBits> for DynBits {
    type Output = DynBits;
    fn xmul(self, rhs: DynBits) -> Self::Output {
        assert!(self.bits + rhs.bits <= 128);
        DynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + rhs.bits,
        }
        .wrapped()
    }
}

impl<const N: usize, const M: usize> XMul<SignedBits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedBits<M>) -> Self::Output {
        self.dyn_bits().xmul(rhs)
    }
}

impl<const N: usize> XMul<SignedBits<N>> for SignedDynBits
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits + N <= 128);
        SignedDynBits {
            val: self.val.wrapping_mul(rhs.raw()),
            bits: self.bits + N,
        }
        .wrapped()
    }
}

impl<const N: usize> XMul<SignedDynBits> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedDynBits) -> Self::Output {
        assert!(N + rhs.bits <= 128);
        SignedDynBits {
            val: self.raw().wrapping_mul(rhs.val),
            bits: N + rhs.bits,
        }
        .wrapped()
    }
}

impl XMul<SignedDynBits> for SignedDynBits {
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedDynBits) -> Self::Output {
        assert!(self.bits + rhs.bits <= 128);
        SignedDynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + rhs.bits,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::alias::*;
    use crate::{bits, signed};

    #[test]
    fn test_xmul() {
        let a = bits::<32>(0x1234_5678);
        let b = bits::<32>(0x8765_4321);
        let c = a.xmul(b);
        assert_eq!(c.as_bits(), bits::<64>(0x1234_5678 * 0x8765_4321));
        let a = signed::<32>(-456);
        let b = signed::<32>(123);
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits(), signed::<64>(-456 * 123));
        let a = bits::<8>(255);
        let b = bits::<8>(255);
        let c = a.xmul(b);
        assert_eq!(c.as_bits(), bits::<16>(255 * 255));
        let a = signed::<8>(-128);
        let b = signed::<8>(-128);
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits(), signed::<16>(-128 * -128));
    }

    #[test]
    fn test_xmul_at_max_sizes() {
        let a = b127::MAX;
        let b = b1::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_bits::<128>(), b127::MAX.resize());
        let a = b125::MAX;
        let b = b3::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_bits::<128>(), bits(a.raw() * b.raw()));
    }

    #[test]
    fn test_signed_mul_at_max_sizes() {
        let a = s126::MAX;
        let b = s2::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits::<128>(), s126::MAX.resize());
        let a = s125::MAX;
        let b = s3::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits::<128>(), signed(a.raw() * b.raw()));
    }
}
