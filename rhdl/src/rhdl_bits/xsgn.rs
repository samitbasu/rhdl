use std::ops::Add;

use crate::rhdl_typenum::prelude::*;

use super::{signed, BitWidth, Bits, SignedBits};

pub trait XSgn {
    type Output;

    fn xsgn(self) -> Self::Output;
}

impl<N> XSgn for Bits<N>
where
    N: Add<U1> + BitWidth,
    op!(N + U1): BitWidth,
{
    type Output = SignedBits<op!(N + U1)>;

    fn xsgn(self) -> Self::Output {
        signed(self.raw() as i128)
    }
}
