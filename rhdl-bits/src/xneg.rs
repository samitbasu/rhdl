use std::ops::Add;

use rhdl_typenum::*;

use crate::{signed, Bits, SignedBits};

pub trait XNeg {
    type Output;
    fn xneg(self) -> Self::Output;
}

impl<N> XNeg for Bits<N>
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn xneg(self) -> Self::Output {
        signed((self.raw() as i128).wrapping_neg())
    }
}

impl<N> XNeg for SignedBits<N>
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn xneg(self) -> Self::Output {
        signed(self.val.wrapping_neg())
    }
}
