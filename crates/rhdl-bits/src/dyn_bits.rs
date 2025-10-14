use crate::{bits_impl::bits_masked, bitwidth::W};

use super::{BitWidth, Bits, signed_dyn_bits::SignedDynBits};

/// A bit vector whose size is determined at runtime.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynBits {
    /// The raw value of the bits.  Only the lower `bits` bits are valid.
    pub(crate) val: u128,
    /// The number of bits in this value.  Must be in the range 1..=128.
    pub(crate) bits: usize,
}

impl std::fmt::Debug for DynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{val}_db{bits}", bits = self.bits, val = self.val)
    }
}

impl DynBits {
    pub(crate) const fn masked(self) -> DynBits {
        DynBits {
            val: self.val & self.mask(),
            bits: self.bits,
        }
    }
    /// Returns a mask with the lower `bits` bits set to 1.
    pub const fn mask(self) -> u128 {
        u128::MAX >> (128 - self.bits)
    }
    /// Zero extend the [DynBits] value to a larger size, returning a [DynBits] value.
    /// The output size is increased by M bits.
    ///
    /// # Panics
    /// Panics if the resulting size would be greater than 128 bits.
    pub const fn xext<const M: usize>(self) -> DynBits {
        assert!((M + self.bits) <= 128);
        DynBits {
            val: self.val,
            bits: M + self.bits,
        }
    }
    /// Shift right by a constant amount, returning a [DynBits] value.
    /// The output size is decreased by the shift amount.
    ///
    /// # Panics
    /// Panics if the resulting size would be zero or negative.
    pub const fn xshr<const M: usize>(self) -> DynBits {
        assert!(self.bits > M);
        DynBits {
            val: self.val >> M,
            bits: self.bits - M,
        }
    }
    /// Shift left by a constant amount, returning a [DynBits] value.
    /// The output size is increased by the shift amount.
    ///
    /// # Panics
    /// Panics if the resulting size would be greater than 128 bits.
    pub const fn xshl<const M: usize>(self) -> DynBits {
        assert!((M + self.bits) <= 128);
        DynBits {
            val: self.val << M,
            bits: self.bits + M,
        }
    }
    /// Resize the [DynBits] value to a different size, returning a [DynBits] value.
    /// If the new size is smaller, the value is truncated.  If the new size is larger,
    /// the value is zero-extended.
    ///
    /// # Panics
    /// Panics if the new size is zero or greater than 128 bits.
    pub const fn resize<const M: usize>(self) -> DynBits {
        assert!(M <= 128);
        assert!(M != 0);
        DynBits {
            val: self.val,
            bits: M,
        }
        .masked()
    }
    /// Returns the raw value of the bits, without masking.
    /// Only the lower `bits` bits are valid.
    pub const fn raw(self) -> u128 {
        self.val
    }
    /// Convert to a [SignedDynBits] value, interpreting the value as a signed integer
    /// in two's complement representation.
    /// The output size is the same as the input size.
    pub const fn as_signed(self) -> SignedDynBits {
        if self.val & (1 << (self.bits - 1)) != 0 {
            SignedDynBits {
                val: self.val as i128 | !self.mask() as i128,
                bits: self.bits,
            }
        } else {
            SignedDynBits {
                val: self.val as i128,
                bits: self.bits,
            }
        }
    }
    /// Returns true if any bit is set.
    /// Can be called in a synthesizable context
    pub const fn any(self) -> bool {
        (self.val & self.mask()) != 0
    }
    /// Returns true if all bits are set.
    /// Can be called in a synthesizable context
    pub const fn all(self) -> bool {
        (self.val & self.mask()) == self.mask()
    }
    /// Returns true if the number of set bits is odd.
    /// Can be called in a synthesizable context
    pub const fn xor(self) -> bool {
        let mut x = self.val & self.mask();
        x ^= x >> 1;
        x ^= x >> 2;
        x ^= x >> 4;
        x ^= x >> 8;
        x ^= x >> 16;
        x ^= x >> 32;
        x ^= x >> 64;
        x & 1 == 1
    }
    /// Wrap the value to fit in the specified number of bits, returning a [DynBits] value.
    /// The output size is the same as the input size.
    pub const fn wrapped(self) -> DynBits {
        DynBits {
            val: self.val & self.mask(),
            bits: self.bits,
        }
    }
    /// Convert to a [Bits] value of the specified size.
    /// # Panics
    /// Panics if the size does not match the number of bits in this value.
    /// Can be called in a synthesizable context
    pub const fn as_bits<const N: usize>(self) -> Bits<N>
    where
        W<N>: BitWidth,
    {
        assert!(self.bits == N);
        bits_masked(self.val)
    }
    /// The number of bits in this [DynBits] value.
    pub const fn bits(self) -> usize {
        self.bits
    }
}

impl std::fmt::Display for DynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'d{}", self.bits, self.val)
    }
}

impl std::fmt::LowerHex for DynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'h{:x}", self.bits, self.val)
    }
}

impl std::fmt::UpperHex for DynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'H{:X}", self.bits, self.val)
    }
}

impl std::fmt::Binary for DynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'b{:b}", self.bits, self.val)
    }
}
