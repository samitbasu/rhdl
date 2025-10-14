use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits};
use crate::{W, signed_dyn_bits::SignedDynBits};

/// Extended negation trait.  Represents a bit-preserving negation
/// of a signed value where the output size is one bit larger
/// than the input.  This is useful for avoiding overflow in
/// negation operations.
pub trait XNeg {
    /// The output type of the negation.
    type Output;
    /// Perform the extended negation operation.
    fn xneg(self) -> Self::Output;
}

impl<const N: usize> XNeg for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xneg(self) -> Self::Output {
        assert!(N < 128);
        let val = (self.val as i128).wrapping_neg();
        SignedDynBits { val, bits: N + 1 }
    }
}

impl XNeg for DynBits {
    type Output = SignedDynBits;
    fn xneg(self) -> Self::Output {
        assert!(self.bits < 128);
        let val = (self.val as i128).wrapping_neg();
        SignedDynBits {
            val,
            bits: self.bits + 1,
        }
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
