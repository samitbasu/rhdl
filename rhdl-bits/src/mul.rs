use rhdl_typenum::*;
use std::ops::{Add, Mul};

use crate::{bits, signed, Bits, SignedBits};

impl<N, M> Mul<Bits<M>> for Bits<N>
where
    N: BitWidth + Add<M>,
    M: BitWidth,
    Sum<N, M>: BitWidth,
{
    type Output = Bits<Sum<N, M>>;
    fn mul(self, rhs: Bits<M>) -> Self::Output {
        bits(self.val.wrapping_mul(rhs.val))
    }
}

impl<N, M> Mul<SignedBits<M>> for SignedBits<N>
where
    N: BitWidth + Add<M>,
    M: BitWidth,
    Sum<N, M>: BitWidth,
{
    type Output = SignedBits<Sum<N, M>>;
    fn mul(self, rhs: SignedBits<M>) -> Self::Output {
        signed(self.val.wrapping_mul(rhs.val))
    }
}

#[cfg(test)]
mod tests {
    use crate::alias::*;

    use super::*;

    #[test]
    fn test_mul() {
        let a = bits::<W32>(0x1234_5678);
        let b = bits::<W32>(0x8765_4321);
        let c = a * b;
        assert_eq!(c, bits::<W64>(0x1234_5678 * 0x8765_4321));
    }

    #[test]
    fn test_mul_at_max_sizes() {
        let a = b127::MAX;
        let b = b1::MAX;
        let c = a * b;
        assert_eq!(c, b127::MAX.resize());
        let a = b125::MAX;
        let b = b3::MAX;
        let c = a * b;
        assert_eq!(c, bits(a.val * b.val));
    }

    #[test]
    fn test_signed_mul_at_max_sizes() {
        let a = s126::MAX;
        let b = s2::MAX;
        let c = a * b;
        assert_eq!(c, s126::MAX.resize());
        let a = s125::MAX;
        let b = s3::MAX;
        let c = a * b;
        assert_eq!(c, signed(a.val * b.val));
    }
}
