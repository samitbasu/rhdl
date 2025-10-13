use crate::{bits_impl::bits_masked, bitwidth::W};

use super::{BitWidth, Bits, signed_dyn_bits::SignedDynBits};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynBits {
    pub val: u128,
    pub bits: usize,
}

impl std::fmt::Debug for DynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{val}_db{bits}", bits = self.bits, val = self.val)
    }
}

impl DynBits {
    pub const fn masked(self) -> DynBits {
        DynBits {
            val: self.val & self.mask(),
            bits: self.bits,
        }
    }
    pub const fn mask(self) -> u128 {
        u128::MAX >> (128 - self.bits)
    }
    pub const fn xext<const M: usize>(self) -> DynBits {
        assert!((M + self.bits) <= 128);
        DynBits {
            val: self.val,
            bits: M + self.bits,
        }
    }
    pub const fn xshr<const M: usize>(self) -> DynBits {
        assert!(self.bits > M);
        DynBits {
            val: self.val >> M,
            bits: self.bits - M,
        }
    }
    pub const fn xshl<const M: usize>(self) -> DynBits {
        assert!((M + self.bits) <= 128);
        DynBits {
            val: self.val << M,
            bits: self.bits + M,
        }
    }
    pub const fn resize<const M: usize>(self) -> DynBits {
        DynBits {
            val: self.val,
            bits: M,
        }
        .masked()
    }
    pub const fn raw(self) -> u128 {
        self.val
    }
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
    pub const fn any(self) -> bool {
        (self.val & self.mask()) != 0
    }
    pub const fn all(self) -> bool {
        (self.val & self.mask()) == self.mask()
    }
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
    pub const fn wrapped(self) -> DynBits {
        DynBits {
            val: self.val & self.mask(),
            bits: self.bits,
        }
    }
    pub const fn as_bits<const N: usize>(self) -> Bits<N>
    where
        W<N>: BitWidth,
    {
        assert!(self.bits == N);
        bits_masked(self.val)
    }
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
