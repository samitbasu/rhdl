use rhdl_typenum::prelude::*;

use super::{
    BitWidth, Bits, SignedBits, bits, dyn_bits::DynBits, signed, signed_dyn_bits::SignedDynBits,
};
use std::ops::Add;

pub trait XAdd<Rhs = Self> {
    type Output;
    fn xadd(self, rhs: Rhs) -> Self::Output;
}

impl<N, M> XAdd<Bits<M>> for Bits<N>
where
    M: BitWidth,
    N: Max<M> + BitWidth,
    Maximum<N, M>: Add<U1>,
    op!(max(N, M) + U1): BitWidth,
{
    type Output = Bits<op!(max(N, M) + U1)>;
    fn xadd(self, rhs: Bits<M>) -> Self::Output {
        // The wrapping_add here is precautionary.  As
        // the sum of the two values is guaranteed to fit
        // in the larger type, it should never actually
        // wrap.
        bits(self.val.wrapping_add(rhs.val))
    }
}

impl<N: BitWidth> XAdd<DynBits> for Bits<N> {
    type Output = DynBits;
    fn xadd(self, rhs: DynBits) -> Self::Output {
        assert!(rhs.bits.max(N::BITS) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: N::BITS.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<N: BitWidth> XAdd<Bits<N>> for DynBits {
    type Output = DynBits;
    fn xadd(self, rhs: Bits<N>) -> Self::Output {
        assert!(self.bits.max(N::BITS) < 128);
        DynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(N::BITS) + 1,
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

impl<N, M> XAdd<SignedBits<M>> for SignedBits<N>
where
    M: BitWidth,
    N: Max<M> + BitWidth,
    Maximum<N, M>: Add<U1>,
    op!(max(N, M) + U1): BitWidth,
{
    type Output = SignedBits<op!(max(N, M) + U1)>;
    fn xadd(self, rhs: SignedBits<M>) -> Self::Output {
        // The wrapping_add here is precautionary.  As
        // the sum of the two values is guaranteed to fit
        // in the larger type, it should never actually
        // wrap.
        signed(self.val.wrapping_add(rhs.val))
    }
}

impl<N: BitWidth> XAdd<SignedDynBits> for SignedBits<N> {
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedDynBits) -> Self::Output {
        assert!(rhs.bits.max(N::BITS) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: N::BITS.max(rhs.bits) + 1,
        }
        .wrapped()
    }
}

impl<N: BitWidth> XAdd<SignedBits<N>> for SignedDynBits {
    type Output = SignedDynBits;
    fn xadd(self, rhs: SignedBits<N>) -> Self::Output {
        assert!(self.bits.max(N::BITS) < 128);
        SignedDynBits {
            val: self.val.wrapping_add(rhs.val),
            bits: self.bits.max(N::BITS) + 1,
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

    #[test]
    fn test_xadd() {
        let a = bits::<U32>(0x1234_5678);
        let b = bits::<U32>(0x8765_4321);
        let c = a.xadd(b);
        assert_eq!(c, bits::<U33>(0x1234_5678 + 0x8765_4321));
        let a = signed::<U32>(-456);
        let b = signed::<U32>(123);
        let c = a.xadd(b);
        assert_eq!(c, signed::<U33>(-456 + 123));
        let a = bits::<U8>(255);
        let b = bits::<U8>(255);
        let c = a.xadd(b);
        assert_eq!(c, bits::<U9>(255 + 255));
    }
}
