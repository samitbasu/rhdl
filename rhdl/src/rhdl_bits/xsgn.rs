use std::ops::Add;

use crate::{rhdl_bits::signed_dyn_bits::SignedDynBits, rhdl_typenum::prelude::*};

use super::{dyn_bits::DynBits, signed, BitWidth, Bits, SignedBits};

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

impl XSgn for DynBits {
    type Output = SignedDynBits;

    fn xsgn(self) -> Self::Output {
        assert!(self.bits < 128);
        SignedDynBits {
            val: self.val as i128,
            bits: self.bits + 1,
        }
        .wrapped()
    }
}
