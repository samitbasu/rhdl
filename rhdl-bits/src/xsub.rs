use std::ops::{Add, Sub};

use rhdl_typenum::*;

use crate::{bits, signed, Bits, SignedBits};

pub trait XSub<Rhs = Self> {
    type Output;
    fn xsub(self, rhs: Rhs) -> Self::Output;
}

impl<N, M> XSub<Bits<M>> for Bits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = SignedBits<Sum<Maximum<N, M>, W1>>;
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
        let a = self.raw() as i128;
        let b = rhs.raw() as i128;
        signed(a.wrapping_sub(b))
    }
}

impl<N, M> XSub<SignedBits<M>> for SignedBits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = SignedBits<Sum<Maximum<N, M>, W1>>;
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
        signed(self.val.wrapping_sub(rhs.val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xsub() {
        let a = bits::<W32>(0x1234_5678);
        let b = bits::<W32>(0x8765_4321);
        let c = a.xsub(b);
        assert_eq!(c, signed(0x1234_5678 - 0x8765_4321));
        let a = bits::<W8>(0);
        let b = bits::<W8>(255);
        let c = a.xsub(b);
        assert_eq!(c, signed(-255));
        let a = bits::<W8>(255);
        let b = bits::<W8>(0);
        let c = a.xsub(b);
        assert_eq!(c, signed(255));
        let a = signed::<W8>(127);
        let b = signed::<W8>(-128);
        let c = a.xsub(b);
        assert_eq!(c, signed(127 + 128));
        let c = b.xsub(a);
        assert_eq!(c, signed(-127 - 128));
    }
}
