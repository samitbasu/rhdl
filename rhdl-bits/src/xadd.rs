use std::ops::Add;

use rhdl_typenum::*;

use crate::{bits, signed, Bits, SignedBits};

pub trait XAdd<Rhs = Self> {
    type Output;
    fn xadd(self, rhs: Rhs) -> Self::Output;
}

impl<N, M> XAdd<Bits<M>> for Bits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = Bits<Sum<Maximum<N, M>, W1>>;
    fn xadd(self, rhs: Bits<M>) -> Self::Output {
        // The wrapping_add here is precautionary.  As
        // the sum of the two values is guaranteed to fit
        // in the larger type, it should never actually
        // wrap.
        bits(self.val.wrapping_add(rhs.val))
    }
}

impl<N, M> XAdd<SignedBits<M>> for SignedBits<N>
where
    N: BitWidth + Max<M>,
    M: BitWidth,
    Maximum<N, M>: Add<W1>,
    Sum<Maximum<N, M>, W1>: BitWidth,
{
    type Output = SignedBits<Sum<Maximum<N, M>, W1>>;
    fn xadd(self, rhs: SignedBits<M>) -> Self::Output {
        // The wrapping_add here is precautionary.  As
        // the sum of the two values is guaranteed to fit
        // in the larger type, it should never actually
        // wrap.
        signed(self.val.wrapping_add(rhs.val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xadd() {
        let a = bits::<W32>(0x1234_5678);
        let b = bits::<W32>(0x8765_4321);
        let c = a.xadd(b);
        assert_eq!(c, bits::<W33>(0x1234_5678 + 0x8765_4321));
        let a = signed::<W32>(-456);
        let b = signed::<W32>(123);
        let c = a.xadd(b);
        assert_eq!(c, signed::<W33>(-456 + 123));
        let a = bits::<W8>(255);
        let b = bits::<W8>(255);
        let c = a.xadd(b);
        assert_eq!(c, bits::<W9>(255 + 255));
    }
}
