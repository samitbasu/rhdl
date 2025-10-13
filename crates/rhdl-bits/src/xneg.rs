use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits};
use crate::{W, signed_dyn_bits::SignedDynBits, xsgn::XSgn};

pub trait XNeg {
    type Output;
    fn xneg(self) -> Self::Output;
}

impl<const N: usize> XNeg for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xneg(self) -> Self::Output {
        self.xsgn().xneg()
    }
}

impl XNeg for DynBits {
    type Output = DynBits;
    fn xneg(self) -> Self::Output {
        assert!(self.bits < 128);
        DynBits {
            val: self.val.wrapping_neg(),
            bits: self.bits + 1,
        }
        .wrapped()
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
