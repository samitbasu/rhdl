use std::ops::Not;

use super::{BitWidth, bits_impl::Bits, signed_bits_impl::SignedBits};

impl<N: BitWidth> Not for Bits<N> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self {
            val: !self.val & Self::mask().val,
            marker: std::marker::PhantomData,
        }
    }
}

impl<N: BitWidth> Not for SignedBits<N> {
    type Output = Self;
    fn not(self) -> Self::Output {
        self.as_unsigned().not().as_signed()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::bitwidth::*;

    #[test]
    fn test_not_bits() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = !bits;
        assert_eq!(result.val, 0b0010_0101_u128);
        let mut bits: Bits<U128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = !bits;
        assert_eq!(result.val, !0_u128 - (1 << 127));
        let bits: Bits<U14> = 0b1101_1010.into();
        let result = !bits;
        assert_eq!(result.val, 0b0011_1111_0010_0101_u128);
    }

    #[test]
    fn test_not_on_i8() {
        let x = -4_i8;
        let result = !x;
        assert_eq!(result, 3_i8);
    }

    #[test]
    fn test_not_on_signed_bits() {
        let x = SignedBits::<U8>::from(-4);
        let result = !x;
        assert_eq!(result.val, 3_i128);
    }

    #[test]
    fn test_not_on_signed_does_not_overflow() {
        let x = SignedBits::<U128>::from(-1);
        let result = !x;
        assert_eq!(result.val, 0_i128);
    }
}
