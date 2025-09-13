use std::ops::Add;

use rhdl_typenum::prelude::*;

use super::{
    BitWidth, Bits, SignedBits, dyn_bits::DynBits, signed, signed_dyn_bits::SignedDynBits,
};

pub trait XSub<Rhs = Self> {
    type Output;
    fn xsub(self, rhs: Rhs) -> Self::Output;
}

impl<N, M> XSub<Bits<M>> for Bits<N>
where
    M: BitWidth,
    N: Max<M> + BitWidth,
    Maximum<N, M>: Add<U1>,
    op!(max(N, M) + U1): BitWidth,
{
    type Output = SignedBits<op!(max(N, M) + U1)>;
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

impl<N: BitWidth> XSub<Bits<N>> for DynBits {
    type Output = SignedDynBits;
    fn xsub(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits.max(N::BITS) < 128);
        let a = self.val as i128;
        let b = rhs.raw() as i128;
        SignedDynBits {
            val: a.wrapping_sub(b),
            bits: self.bits.max(N::BITS) + 1,
        }
        .wrapped()
    }
}

impl<N: BitWidth> XSub<DynBits> for Bits<N> {
    type Output = SignedDynBits;
    fn xsub(self, rhs: DynBits) -> Self::Output {
        assert!(N::BITS.max(rhs.bits) < 128);
        let a = self.raw() as i128;
        let b = rhs.val as i128;
        SignedDynBits {
            val: a.wrapping_sub(b),
            bits: N::BITS.max(rhs.bits) + 1,
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

impl<N, M> XSub<SignedBits<M>> for SignedBits<N>
where
    N: Max<M> + BitWidth,
    M: BitWidth,
    Maximum<N, M>: Add<U1>,
    op!(max(N, M) + U1): BitWidth,
{
    type Output = SignedBits<op!(max(N, M) + U1)>;
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

impl<N: BitWidth> XSub<SignedDynBits> for SignedBits<N> {
    type Output = SignedDynBits;
    fn xsub(self, rhs: SignedDynBits) -> Self::Output {
        assert!(N::BITS.max(rhs.bits) < 128);
        SignedDynBits {
            val: self.val.wrapping_sub(rhs.val),
            bits: N::BITS.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<N: BitWidth> XSub<SignedBits<N>> for SignedDynBits {
    type Output = SignedDynBits;
    fn xsub(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits.max(N::BITS) < 128);
        SignedDynBits {
            val: self.val.wrapping_sub(rhs.val),
            bits: self.bits.max(N::BITS) + 1,
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

    use crate::bits;

    use super::*;

    /*    fn xsub_trait<N>()
        where
            N: BitWidth,
            op!(N + U1): BitWidth,
            op!(max(N, N) + U1): BitWidth,
        {
            let a = bits::<N>(42);
            let b = bits::<N>(36);
            let c = a.xsub(b);
            assert_eq!(c.raw(), signed::<Sum<N, U1>>(42 - 36).raw());
        }

        #[test]
        fn test_xsub_trait() {
            xsub_trait::<U8>();
        }
    */
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
        let _c: SignedBits<U9> = a.xsub(b);
        let w = Maximum::<U4, U8>::BITS;
        assert_eq!(w, 8);
    }
}
