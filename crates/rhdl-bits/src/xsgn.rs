use super::{BitWidth, Bits, dyn_bits::DynBits};
use crate::{W, signed_dyn_bits::SignedDynBits};

/// Promote an unsigned value to a signed value with all bits
/// preserved.  The output size is one bit larger than the input size.
/// This is useful for avoiding overflow in sign promotion operations.
pub trait XSgn {
    /// The output type of the sign promotion.
    type Output;
    /// Perform the sign promotion operation.
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
