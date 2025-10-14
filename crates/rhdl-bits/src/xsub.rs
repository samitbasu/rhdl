use super::{BitWidth, Bits, SignedBits, dyn_bits::DynBits, signed_dyn_bits::SignedDynBits};
use crate::bitwidth::W;

/// Extended subtraction trait.  Represents a bit-preserving subtraction
/// of two values (either signed or unsigned) where the output
/// size is one bit larger than the larger of the two inputs.
/// This is useful for avoiding overflow in subtraction operations.
pub trait XSub<Rhs = Self> {
    /// The output type of the subtraction.
    type Output;
    /// Perform the extended subtraction operation.
    fn xsub(self, rhs: Rhs) -> Self::Output;
}

impl<const N: usize, const M: usize> XSub<Bits<M>> for Bits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = SignedDynBits;
    fn xsub(self, rhs: Bits<M>) -> Self::Output {
        // The rationale here is as follows.
        // If
        //     A in [0, 2^n-1]
        //     B in [0, 2^m-1]
        // To minimize A-B, we choose A to be zero, and
        // to maximize A-B, we choose B to be zero.  In
        // that case, the range of the difference is
        //    A-B  in [-(2^m-1), 2^n-1]
        // The range for a k-bit signed 2's complement
        // integer is [-(2^(k-1)), 2^(k-1)-1].  So, if
        // we want to represent the full range of the
        // difference, we need
        //    k-1 >= m,
        //    k-1 >= n
        // Or equivalently,
        //    k >= max(m,n) + 1
        self.dyn_bits().xsub(rhs)
    }
}

impl<const N: usize> XSub<Bits<N>> for DynBits
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xsub(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits.max(N) < 128);
        let a = self.val as i128;
        let b = rhs.raw() as i128;
        SignedDynBits {
            val: a.wrapping_sub(b),
            bits: self.bits.max(N) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize> XSub<DynBits> for Bits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xsub(self, rhs: DynBits) -> Self::Output {
        assert!(N.max(rhs.bits) < 128);
        let a = self.raw() as i128;
        let b = rhs.val as i128;
        SignedDynBits {
            val: a.wrapping_sub(b),
            bits: N.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl XSub<DynBits> for DynBits {
    type Output = SignedDynBits;
    fn xsub(self, rhs: DynBits) -> Self::Output {
        assert!(self.bits.max(rhs.bits) < 128);
        SignedDynBits {
            val: (self.val as i128).wrapping_sub(rhs.val as i128),
            bits: self.bits.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize, const M: usize> XSub<SignedBits<M>> for SignedBits<N>
where
    W<N>: BitWidth,
    W<M>: BitWidth,
{
    type Output = SignedDynBits;
    // The rationale here is as follows.
    //    A in [-2^(n-1), 2^(n-1)-1]
    //    B in [-2^(m-1), 2^(m-1)-1]
    // To minimize A-B, we choose A to be -2^(n-1),
    // and let B be 2^(m-1)-1.  Thus, the smallest
    // value of A-B is
    //    A-B >= -2^(n-1) - 2^(m-1) + 1
    //        >= -(2^(n-1) + 2^(m-1)) + 1
    // To maximize A-B, we choose A to be 2^(n-1)-1,
    // and let B be -2^(m-1).  Thus, the largest
    // value of A-B is
    //    A-B <= 2^(n-1) - 1 - (-2^(m-1))
    //        <= 2^(n-1) + 2^(m-1) - 1
    // We can bound the range by choosing p = max(m,n),
    // in which case the range of the difference is
    //    A-B in [-(2^(p-1) + 2^(p-1)) + 1, 2^(p-1) + 2^(p-1) - 1]
    //        in [-2^p + 1, 2^p - 1]
    // The range for a k-bit signed 2's complement
    // integer is [-(2^(k-1)), 2^(k-1)-1].
    // So, if we want to represent the full range of
    // the difference, we need
    //    k-1 >= p, or k >= max(m,n) + 1
    fn xsub(self, rhs: SignedBits<M>) -> Self::Output {
        self.dyn_bits().xsub(rhs)
    }
}

impl<const N: usize> XSub<SignedDynBits> for SignedBits<N>
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xsub(self, rhs: SignedDynBits) -> Self::Output {
        assert!(N.max(rhs.bits) < 128);
        SignedDynBits {
            val: self.val.wrapping_sub(rhs.val),
            bits: N.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<const N: usize> XSub<SignedBits<N>> for SignedDynBits
where
    W<N>: BitWidth,
{
    type Output = SignedDynBits;
    fn xsub(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits.max(N) < 128);
        SignedDynBits {
            val: self.val.wrapping_sub(rhs.val),
            bits: self.bits.max(N) + 1,
        }
        .wrapped()
    }
}

impl XSub<SignedDynBits> for SignedDynBits {
    type Output = SignedDynBits;
    fn xsub(self, rhs: SignedDynBits) -> Self::Output {
        assert!(self.bits.max(rhs.bits) < 128);
        SignedDynBits {
            val: self.val.wrapping_sub(rhs.val),
            bits: self.bits.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod tests {

    use crate::{bits, signed};

    use super::*;

    #[test]
    fn test_xsub() {
        let a = bits::<32>(0x1234_5678);
        let b = bits::<32>(0x8765_4321);
        let c = a.xsub(b);
        assert_eq!(c.as_signed_bits::<33>(), signed(0x1234_5678 - 0x8765_4321));
        let a = bits::<8>(0);
        let b = bits::<8>(255);
        let c = a.xsub(b);
        assert_eq!(c.as_signed_bits::<9>(), signed(-255));
        let a = bits::<8>(255);
        let b = bits::<8>(0);
        let c = a.xsub(b);
        assert_eq!(c.as_signed_bits::<9>(), signed(255));
        let a = signed::<8>(127);
        let b = signed::<8>(-128);
        let c = a.xsub(b);
        assert_eq!(c.as_signed_bits::<9>(), signed(127 + 128));
        let c = b.xsub(a);
        assert_eq!(c.as_signed_bits::<9>(), signed(-127 - 128));
    }
}
