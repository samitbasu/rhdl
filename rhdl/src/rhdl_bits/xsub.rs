use std::ops::Add;

use crate::rhdl_typenum::prelude::*;

use super::{signed, BitWidth, Bits, SignedBits};

pub trait XSub {
    type Output;
    fn xsub(self, rhs: Self) -> Self::Output;
}

impl<N> XSub for Bits<N>
where
    N: BitWidth + Add<U1>,
    op!(N + U1): BitWidth,
{
    type Output = SignedBits<op!(N + U1)>;
    fn xsub(self, rhs: Bits<N>) -> Self::Output {
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

impl<N> XSub for SignedBits<N>
where
    N: Add<U1> + BitWidth,
    op!(N + U1): BitWidth,
{
    type Output = SignedBits<op!(N + U1)>;
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
    fn xsub(self, rhs: Self) -> Self::Output {
        signed(self.val.wrapping_sub(rhs.val))
    }
}

#[cfg(test)]
mod tests {

    use crate::rhdl_bits::bits;

    use super::*;

    #[test]
    fn test_xsub() {
        let a = bits::<U32>(0x1234_5678);
        let b = bits::<U32>(0x8765_4321);
        let c = a.xsub(b);
        assert_eq!(c, signed(0x1234_5678 - 0x8765_4321));
        let a = bits::<U8>(0);
        let b = bits::<U8>(255);
        let c = a.xsub(b);
        assert_eq!(c, signed(-255));
        let a = bits::<U8>(255);
        let b = bits::<U8>(0);
        let c = a.xsub(b);
        assert_eq!(c, signed(255));
        let a = signed::<U8>(127);
        let b = signed::<U8>(-128);
        let c = a.xsub(b);
        assert_eq!(c, signed(127 + 128));
        let c = b.xsub(a);
        assert_eq!(c, signed(-127 - 128));
    }

    #[test]
    fn test_xsub_size_trait() {
        let a = signed::<U4>(7);
        let b: SignedBits<U8> = signed::<U8>(3);
        let _c: SignedBits<U9> = a.xext::<U4>().xsub(b);
        let w = Maximum::<U4, U8>::BITS;
        assert_eq!(w, 8);
    }
}
