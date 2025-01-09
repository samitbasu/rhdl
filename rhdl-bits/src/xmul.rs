use std::ops::Add;

use rhdl_typenum::*;

use crate::{bits, signed, Bits, SignedBits};

pub trait XMul<Rhs = Self> {
    type Output;
    fn xmul(self, rhs: Rhs) -> Self::Output;
}

impl<N, M> XMul<Bits<M>> for Bits<N>
where
    N: BitWidth + Add<M>,
    M: BitWidth,
    Sum<N, M>: BitWidth,
{
    type Output = Bits<Sum<N, M>>;
    fn xmul(self, rhs: Bits<M>) -> Self::Output {
        bits(self.val.wrapping_mul(rhs.val))
    }
}

impl<N, M> XMul<SignedBits<M>> for SignedBits<N>
where
    N: BitWidth + Add<M>,
    M: BitWidth,
    Sum<N, M>: BitWidth,
{
    type Output = SignedBits<Sum<N, M>>;
    fn xmul(self, rhs: SignedBits<M>) -> Self::Output {
        signed(self.val.wrapping_mul(rhs.val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::*;

    #[test]
    fn test_xmul() {
        let a = bits::<W32>(0x1234_5678);
        let b = bits::<W32>(0x8765_4321);
        let c = a.xmul(b);
        assert_eq!(c, bits::<W64>(0x1234_5678 * 0x8765_4321));
        let a = signed::<W32>(-456);
        let b = signed::<W32>(123);
        let c = a.xmul(b);
        assert_eq!(c, signed::<W64>(-456 * 123));
        let a = bits::<W8>(255);
        let b = bits::<W8>(255);
        let c = a.xmul(b);
        assert_eq!(c, bits::<W16>(255 * 255));
        let a = signed::<W8>(-128);
        let b = signed::<W8>(-128);
        let c = a.xmul(b);
        assert_eq!(c, signed::<W16>(-128 * -128));
    }

    #[test]
    fn test_xmul_at_max_sizes() {
        let a = b127::MAX;
        let b = b1::MAX;
        let c = a.xmul(b);
        assert_eq!(c, b127::MAX.resize());
        let a = b125::MAX;
        let b = b3::MAX;
        let c = a.xmul(b);
        assert_eq!(c, bits(a.val * b.val));
    }

    #[test]
    fn test_signed_mul_at_max_sizes() {
        let a = s126::MAX;
        let b = s2::MAX;
        let c = a.xmul(b);
        assert_eq!(c, s126::MAX.resize());
        let a = s125::MAX;
        let b = s3::MAX;
        let c = a.xmul(b);
        assert_eq!(c, signed(a.val * b.val));
    }
}
