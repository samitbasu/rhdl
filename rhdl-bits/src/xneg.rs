use std::ops::Add;

use typenum::{op, U1};

use crate::{signed, BitWidth, Bits, SignedBits};

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
