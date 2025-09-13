use std::ops::Add;

use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits, signed};
use crate::signed_dyn_bits::SignedDynBits;
use rhdl_typenum::prelude::*;

pub trait XNeg {
    type Output;
    fn xneg(self) -> Self::Output;
}

impl<N> XNeg for Bits<N>
where
    N: Add<U1> + BitWidth,
    op!(N + U1): BitWidth,
{
    type Output = SignedBits<op!(N + U1)>;
    fn xneg(self) -> Self::Output {
        signed((self.raw() as i128).wrapping_neg())
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

impl<N> XNeg for SignedBits<N>
where
    N: Add<U1> + BitWidth,
    op!(N + U1): BitWidth,
{
    type Output = SignedBits<op!(N + U1)>;
    fn xneg(self) -> Self::Output {
        signed(self.val.wrapping_neg())
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
