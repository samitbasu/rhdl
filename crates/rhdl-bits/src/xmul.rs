use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits, signed_dyn_bits::SignedDynBits};
use crate::bitwidth::W;

/// Extended multiplication trait.  Represents a bit-preserving multiplication
/// of two values (either signed or unsigned) where the output
/// size is the sum of the sizes of the two inputs.
/// This is useful for avoiding overflow in multiplication operations.
pub trait XMul<Rhs = Self> {
    /// The output type of the multiplication.
    type Output;
    /// Perform the extended multiplication operation.
    fn xmul(self, rhs: Rhs) -> Self::Output;
}

impl<const N: usize, const M: usize> XMul<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: Bits<M>) -> Self::Output {
        self.dyn_bits().xmul(rhs)
    }
}

impl<const N: usize> XMul<Bits<N>> for DynBits
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits + N <= 128);
        DynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + N,
        }
        .wrapped()
    }
}

impl<const N: usize> XMul<DynBits> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: DynBits) -> Self::Output {
        assert!(N + rhs.bits <= 128);
        DynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: N + rhs.bits,
        }
        .wrapped()
    }
}

impl XMul<DynBits> for DynBits {
    type Output = DynBits;
    fn xmul(self, rhs: DynBits) -> Self::Output {
        assert!(self.bits + rhs.bits <= 128);
        DynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + rhs.bits,
        }
        .wrapped()
    }
}

impl<const N: usize, const M: usize> XMul<SignedBits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedBits<M>) -> Self::Output {
        self.dyn_bits().xmul(rhs)
    }
}

impl<const N: usize> XMul<SignedBits<N>> for SignedDynBits
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits + N <= 128);
        SignedDynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + N,
        }
        .wrapped()
    }
}

impl<const N: usize> XMul<SignedDynBits> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedDynBits) -> Self::Output {
        assert!(N + rhs.bits <= 128);
        SignedDynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: N + rhs.bits,
        }
        .wrapped()
    }
}

impl XMul<SignedDynBits> for SignedDynBits {
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedDynBits) -> Self::Output {
        assert!(self.bits + rhs.bits <= 128);
        SignedDynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + rhs.bits,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::alias::*;
    use crate::{bits, signed};

    #[test]
    fn test_xmul() {
        let a = bits::<32>(0x1234_5678);
        let b = bits::<32>(0x8765_4321);
        let c = a.xmul(b);
        assert_eq!(c.as_bits(), bits::<64>(0x1234_5678 * 0x8765_4321));
        let a = signed::<32>(-456);
        let b = signed::<32>(123);
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits(), signed::<64>(-456 * 123));
        let a = bits::<8>(255);
        let b = bits::<8>(255);
        let c = a.xmul(b);
        assert_eq!(c.as_bits(), bits::<16>(255 * 255));
        let a = signed::<8>(-128);
        let b = signed::<8>(-128);
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits(), signed::<16>(-128 * -128));
    }

    #[test]
    fn test_xmul_at_max_sizes() {
        let a = b127::MAX;
        let b = b1::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_bits::<128>(), b127::MAX.resize());
        let a = b125::MAX;
        let b = b3::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_bits::<128>(), bits(a.val * b.val));
    }

    #[test]
    fn test_signed_mul_at_max_sizes() {
        let a = s126::MAX;
        let b = s2::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits::<128>(), s126::MAX.resize());
        let a = s125::MAX;
        let b = s3::MAX;
        let c = a.xmul(b);
        assert_eq!(c.as_signed_bits::<128>(), signed(a.val * b.val));
    }
}
