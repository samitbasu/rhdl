use super::{BitWidth, Bits, dyn_bits::DynBits};
use crate::{W, signed_dyn_bits::SignedDynBits};

pub trait XSgn {
    type Output;

    fn xsgn(self) -> Self::Output;
}

impl<const N: usize> XSgn for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;

    fn xsgn(self) -> Self::Output {
        assert!(N < 128);
        self.dyn_bits().xsgn()
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
