use std::ops::Add;

use rhdl_typenum::*;

use crate::{signed, Bits, SignedBits};

pub trait XSgn {
    type Output;

    fn xsgn(self) -> Self::Output;
}

impl<N> XSgn for Bits<N>
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;

    fn xsgn(self) -> Self::Output {
        signed(self.raw() as i128)
    }
}
