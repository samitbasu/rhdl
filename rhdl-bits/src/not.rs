use std::ops::Not;

use crate::{bits::Bits, signed_bits::SignedBits};

impl<const N: usize> Not for Bits<N> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0 & Self::mask().0)
    }
}

impl<const N: usize> Not for SignedBits<N> {
    type Output = Self;
    fn not(self) -> Self::Output {
        self.as_unsigned().not().as_signed()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_not_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = !bits;
        assert_eq!(result.0, 0b0010_0101_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = !bits;
        assert_eq!(result.0, !0_u128 - (1 << 127));
        let bits: Bits<14> = 0b1101_1010.into();
        let result = !bits;
        assert_eq!(result.0, 0b0011_1111_0010_0101_u128);
    }

    #[test]
    fn test_not_on_i8() {
        let x = -4_i8;
        let result = !x;
        assert_eq!(result, 3_i8);
    }

    #[test]
    fn test_not_on_signed_bits() {
        let x = SignedBits::<8>::from(-4);
        let result = !x;
        assert_eq!(result.0, 3_i128);
    }

    #[test]
    fn test_not_on_signed_does_not_overflow() {
        let x = SignedBits::<128>::from(-1);
        let result = !x;
        assert_eq!(result.0, 0_i128);
    }
}
