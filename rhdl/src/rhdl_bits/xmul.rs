use crate::{bits, signed, BitWidth, Bits, SignedBits};
use rhdl_typenum::prelude::*;
use std::ops::Add;

pub trait XMul<Rhs = Self> {
    type Output;
    fn xmul(self, rhs: Rhs) -> Self::Output;
}

impl<N, M> XMul<Bits<M>> for Bits<N>
where
    N: Add<M> + BitWidth,
    M: BitWidth,
    op!(N + M): BitWidth,
{
    type Output = Bits<op!(N + M)>;
    fn xmul(self, rhs: Bits<M>) -> Self::Output {
        bits(self.val.wrapping_mul(rhs.val))
    }
}

impl<N, M> XMul<SignedBits<M>> for SignedBits<N>
where
    N: Add<M> + BitWidth,
    M: BitWidth,
    op!(N + M): BitWidth,
{
    type Output = SignedBits<op!(N + M)>;
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
        let a = bits::<U32>(0x1234_5678);
        let b = bits::<U32>(0x8765_4321);
        let c = a.xmul(b);
        assert_eq!(c, bits::<U64>(0x1234_5678 * 0x8765_4321));
        let a = signed::<U32>(-456);
        let b = signed::<U32>(123);
        let c = a.xmul(b);
        assert_eq!(c, signed::<U64>(-456 * 123));
        let a = bits::<U8>(255);
        let b = bits::<U8>(255);
        let c = a.xmul(b);
        assert_eq!(c, bits::<U16>(255 * 255));
        let a = signed::<U8>(-128);
        let b = signed::<U8>(-128);
        let c = a.xmul(b);
        assert_eq!(c, signed::<U16>(-128 * -128));
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
