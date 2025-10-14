use crate::bitwidth::W;
use crate::signed_bits_impl::signed_wrapped;

use super::{BitWidth, SignedBits, dyn_bits::DynBits};

/// A signed bit vector whose size is determined at runtime.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignedDynBits {
    /// The raw value of the bits.  Represented as 2s complement
    /// integer.  Only the lower `bits` bits are valid.
    pub(crate) val: i128,
    /// The number of bits in this value.  Must be in the range 1..=128.
    pub(crate) bits: usize,
}

impl std::fmt::Debug for SignedDynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{val}_ds{bits}", bits = self.bits, val = self.val)
    }
}

impl SignedDynBits {
    /// The largest positive value that can be represented
    /// with this number of bits.
    pub const fn max_value(self) -> i128 {
        i128::MAX >> (128 - self.bits)
    }
    /// The smallest (most negative) value that can be represented
    /// with this number of bits.
    pub const fn min_value(self) -> i128 {
        i128::MIN >> (128 - self.bits)
    }
    /// Convert to an unsigned [DynBits] value.
    /// The output size is the same as the input size.
    /// The value is reinterpreted as an unsigned value.
    pub const fn as_unsigned(self) -> DynBits {
        DynBits {
            val: self.val as u128,
            bits: self.bits,
        }
        .masked()
    }
    /// Extract the raw i128 value.
    pub const fn raw(self) -> i128 {
        self.val
    }
    /// Returns true if any bit is set.
    /// Can be called in a synthesizable context
    pub const fn any(self) -> bool {
        self.val != 0
    }
    /// Returns true if all bits are set.
    /// Can be called in a synthesizable context
    pub const fn all(self) -> bool {
        self.val == -1
    }
    /// Returns true if the number of set bits is odd.
    /// Can be called in a synthesizable context
    pub const fn xor(self) -> bool {
        let mut x = self.val as u128;
        x ^= x >> 1;
        x ^= x >> 2;
        x ^= x >> 4;
        x ^= x >> 8;
        x ^= x >> 16;
        x ^= x >> 32;
        x ^= x >> 64;
        x & 1 == 1
    }
    /// Sign extend the value by the given number of bits, returning a [SignedDynBits] value.
    /// The output size is the input size plus the extension amount.
    ///
    /// # Panics
    /// Panics if the output size would be greater than 128 bits.
    pub const fn xext<const M: usize>(self) -> SignedDynBits {
        assert!(self.bits + M <= 128);
        SignedDynBits {
            val: self.val,
            bits: M + self.bits,
        }
    }
    /// Shift right by a constant amount, returning a [SignedDynBits] value.
    /// The output size is the input size minus the shift amount.
    /// # Panics
    /// Panics if the output size would be zero or negative.
    pub const fn xshr<const M: usize>(self) -> SignedDynBits {
        assert!(self.bits > M);
        SignedDynBits {
            val: self.val >> M,
            bits: self.bits - M,
        }
    }
    /// Shift left by a constant amount, returning a [SignedDynBits] value.
    /// The output size is the input size plus the shift amount.
    /// # Panics
    /// Panics if the output size would be greater than 128 bits.
    pub const fn xshl<const M: usize>(self) -> SignedDynBits {
        assert!(self.bits + M <= 128);
        SignedDynBits {
            val: self.val << M,
            bits: self.bits + M,
        }
    }
    /// Resize the [SignedDynBits] value to a different number of bits.
    /// If the new size is larger than the current size, then sign
    /// extension is performed.  If the new size is smaller than the
    /// current size, then the value is truncated to fit in the
    /// smaller size.
    pub const fn resize<const M: usize>(self) -> SignedDynBits {
        if M > self.bits {
            SignedDynBits {
                val: self.val,
                bits: M,
            }
        } else {
            self.as_unsigned().resize::<M>().as_signed()
        }
    }
    /// Wrap the value to fit in the current number of bits.
    /// This is done by converting to unsigned, wrapping, and
    /// converting back to signed.
    pub const fn wrapped(self) -> SignedDynBits {
        self.as_unsigned().wrapped().as_signed()
    }
    /// Convert to a [SignedBits] value of the given size.
    /// # Panics
    /// Panics if the size does not match the current size.
    pub const fn as_signed_bits<const N: usize>(self) -> SignedBits<N>
    where
        W<N>: BitWidth,
    {
        assert!(N == self.bits);
        signed_wrapped(self.val)
    }
    /// Returns the number of bits in this value.
    pub const fn bits(self) -> usize {
        self.bits
    }
}
