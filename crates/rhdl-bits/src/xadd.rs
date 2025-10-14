use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits, signed_dyn_bits::SignedDynBits};
use crate::bitwidth::W;

/// Extended addition trait.  Represents a bit-preserving addition
/// of two values (either signed or unsigned) where the output
/// size is one bit larger than the larger of the two inputs.
/// This is useful for avoiding overflow in addition operations.
pub trait XAdd<Rhs = Self> {
    /// The output type of the addition.
    type Output;
    /// Perform the extended addition operation.
    fn xadd(self, rhs: Rhs) -> Self::Output;
}

impl<const N: usize, const M: usize> XAdd<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = DynBits;
    fn xadd(self, rhs: Bits<M>) -> Self::Output {
        assert!(N.max(M) < 128);
        self.dyn_bits().xadd(rhs)
    }
}

impl<const N: usize> XAdd<DynBits> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xadd(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.bits.max(N) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: N.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize> XAdd<Bits<N>> for DynBits
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xadd(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits.max(N) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(N) + 1,
        }
        .wrapped()
    }
}

impl XAdd<DynBits> for DynBits {
    type Output = DynBits;
    fn xadd(self, rhs: DynBits) -> Self::Output {
        assert!(self.bits.max(rhs.bits) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize, const M: usize> XAdd<SignedBits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedBits<M>) -> Self::Output {
        self.dyn_bits().xadd(rhs)
    }
}

impl<const N: usize> XAdd<SignedDynBits> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedDynBits) -> Self::Output {
        assert!(rhs.bits.max(N) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: N.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize> XAdd<SignedBits<N>> for SignedDynBits
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits.max(N) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(N) + 1,
        }
        .wrapped()
    }
}

impl XAdd<SignedDynBits> for SignedDynBits {
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedDynBits) -> Self::Output {
        assert!(self.bits.max(rhs.bits) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bits, signed};

    #[test]
    fn test_xadd() {
        let a = bits::<32>(0x1234_5678);
        let b = bits::<32>(0x8765_4321);
        let c = a.xadd(b).as_bits();
        assert_eq!(c, bits::<33>(0x1234_5678 + 0x8765_4321));
        let a = signed::<32>(-456);
        let b = signed::<32>(123);
        let c = a.xadd(b).as_signed_bits();
        assert_eq!(c, signed::<33>(-456 + 123));
        let a = bits::<8>(255);
        let b = bits::<8>(255);
        let c = a.xadd(b).as_bits();
        assert_eq!(c, bits::<9>(255 + 255));
    }
}
