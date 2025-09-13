use super::{
    BitWidth, Bits, SignedBits, bits, dyn_bits::DynBits, signed, signed_dyn_bits::SignedDynBits,
};
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

impl<N> XMul<Bits<N>> for DynBits
where
    N: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits + N::BITS <= 128);
        DynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + N::BITS,
        }
        .wrapped()
    }
}

impl<N> XMul<DynBits> for Bits<N>
where
    N: BitWidth,
{
    type Output = DynBits;
    fn xmul(self, rhs: DynBits) -> Self::Output {
        assert!(N::BITS + rhs.bits <= 128);
        DynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: N::BITS + rhs.bits,
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

impl<N> XMul<SignedBits<N>> for SignedDynBits
where
    N: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits + N::BITS <= 128);
        SignedDynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: self.bits + N::BITS,
        }
        .wrapped()
    }
}

impl<N> XMul<SignedDynBits> for SignedBits<N>
where
    N: BitWidth,
{
    type Output = SignedDynBits;
    fn xmul(self, rhs: SignedDynBits) -> Self::Output {
        assert!(N::BITS + rhs.bits <= 128);
        SignedDynBits {
            val: self.val.wrapping_mul(rhs.val),
            bits: N::BITS + rhs.bits,
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
