use crate::bitwidth::W;
use crate::signed_bits_impl::signed_wrapped;

use super::{BitWidth, SignedBits, dyn_bits::DynBits};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignedDynBits {
    pub val: i128,
    pub bits: usize,
}

impl std::fmt::Debug for SignedDynBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{val}_ds{bits}", bits = self.bits, val = self.val)
    }
}

impl SignedDynBits {
    pub const fn max_value(self) -> i128 {
        i128::MAX >> (128 - self.bits)
    }
    pub const fn min_value(self) -> i128 {
        i128::MIN >> (128 - self.bits)
    }
    pub const fn as_unsigned(self) -> DynBits {
        DynBits {
            val: self.val as u128,
            bits: self.bits,
        }
        .masked()
    }
    pub const fn raw(self) -> i128 {
        self.val
    }
    pub const fn any(self) -> bool {
        self.val != 0
    }
    pub const fn all(self) -> bool {
        self.val == -1
    }
    pub const fn xor(self) -> bool {
        let mut x = self.val;
        x ^= x >> 1;
        x ^= x >> 2;
        x ^= x >> 4;
        x ^= x >> 8;
        x ^= x >> 16;
        x ^= x >> 32;
        x ^= x >> 64;
        x & 1 == 1
    }
    pub const fn xext<const M: usize>(self) -> SignedDynBits {
        assert!(self.bits + M <= 128);
        SignedDynBits {
            val: self.val,
            bits: M + self.bits,
        }
    }
    pub const fn xshr<const M: usize>(self) -> SignedDynBits {
        assert!(self.bits > M);
        SignedDynBits {
            val: self.val >> M,
            bits: self.bits - M,
        }
    }
    pub const fn xshl<const M: usize>(self) -> SignedDynBits {
        assert!(self.bits + M <= 128);
        SignedDynBits {
            val: self.val << M,
            bits: self.bits + M,
        }
    }
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
    pub const fn wrapped(self) -> SignedDynBits {
        self.as_unsigned().wrapped().as_signed()
    }
    pub const fn as_signed_bits<const N: usize>(self) -> SignedBits<N>
    where
        W<N>: BitWidth,
    {
        assert!(N == self.bits);
        signed_wrapped(self.val)
    }
    pub const fn bits(self) -> usize {
        self.bits
    }
}
