use std::ops::{BitAnd, BitAndAssign};

use super::{bits_impl::Bits, signed_bits_impl::SignedBits, BitWidth};

impl<N: BitWidth> BitAnd<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn bitand(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) & rhs
    }
}

impl<N: BitWidth> BitAnd<u128> for Bits<N> {
    type Output = Self;
    fn bitand(self, rhs: u128) -> Self::Output {
        self & Bits::<N>::from(rhs)
    }
}

impl<N: BitWidth> BitAndAssign<u128> for Bits<N> {
    fn bitand_assign(&mut self, rhs: u128) {
        self.val &= rhs;
    }
}

impl<N: BitWidth> BitAnd<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn bitand(self, rhs: SignedBits<N>) -> Self::Output {
        SignedBits::<N>::from(self) & rhs
    }
}

impl<N: BitWidth> BitAnd<i128> for SignedBits<N> {
    type Output = Self;
    fn bitand(self, rhs: i128) -> Self::Output {
        self & SignedBits::<N>::from(rhs)
    }
}

impl<N: BitWidth> BitAndAssign<i128> for SignedBits<N> {
    fn bitand_assign(&mut self, rhs: i128) {
        self.val &= rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rhdl_bits::bitwidth::*;

    #[test]
    fn test_and_bits() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits & bits;
        assert_eq!(result.val, 0b1101_1010_u128);
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits & 0b1111_0000;
        assert_eq!(result.val, 0b1101_0000_u128);
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = 0b1111_0000 & bits;
        assert_eq!(result.val, 0b1101_0000_u128);
        let mut bits: Bits<U128> = 0.into();
        bits = crate::rhdl_bits::test::set_bit(bits, 127, true);
        let result = bits & bits;
        assert_eq!(result.val, 1_u128 << 127);
        let bits: Bits<U54> = 0b1101_1010.into();
        let result = bits & 1;
        assert_eq!(result.val, 0_u128);
        let result = 1 & bits;
        assert_eq!(result.val, 0_u128);
    }

    #[test]
    fn test_andassign_bits() {
        let mut bits: Bits<U8> = 0b1101_1010.into();
        bits &= bits;
        assert_eq!(bits.val, 0b1101_1010_u128);
        let mut bits: Bits<U8> = 0b1101_1010.into();
        bits &= 0b1111_0000;
        assert_eq!(bits.val, 0b1101_0000_u128);
        let mut bits: Bits<U8> = 0b1101_1010.into();
        bits &= 0b1111_0000;
        assert_eq!(bits.val, 0b1101_0000_u128);
        let mut bits: Bits<U128> = 0.into();
        bits = crate::rhdl_bits::test::set_bit(bits, 127, true);
        bits &= bits;
        assert_eq!(bits.val, 1_u128 << 127);
        let mut bits: Bits<U54> = 0b1101_1010.into();
        bits &= 1;
        assert_eq!(bits.val, 0_u128);
        let mut bits: Bits<U54> = 0b1101_1010.into();
        bits &= 1;
        assert_eq!(bits.val, 0_u128);
        let a: Bits<U12> = 0b1010_1010_1010.into();
        let b: Bits<U12> = 0b1111_0101_0111.into();
        let mut c = a;
        c &= b;
        assert_eq!(c.val, 0b1010_0000_0010);
    }

    #[test]
    fn test_anding_is_weird_for_signed() {
        let x = -3;
        let y = 4;
        let z = x & y;
        assert!(z > 0);
    }

    #[test]
    fn test_and_assign_signed() {
        let mut x = SignedBits::<U8>::from(-3);
        let y = SignedBits::<U8>::from(4);
        x &= y;
        assert_eq!(x.val, 4);
    }

    #[test]
    fn test_and_signed() {
        let x = SignedBits::<U8>::from(-3);
        let y = SignedBits::<U8>::from(4);
        let z = x & y;
        assert_eq!(z.val, 4);
    }

    #[test]
    fn test_and_signed_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let x: SignedBits<U8> = (i as i128).into();
                let y: SignedBits<U8> = (j as i128).into();
                let z = x & y;
                assert_eq!(z.val, (i & j) as i128);
            }
        }
    }

    #[test]
    fn test_and_assign_signed_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let mut x: SignedBits<U8> = (i as i128).into();
                let y: SignedBits<U8> = (j as i128).into();
                x &= y;
                assert_eq!(x.val, (i & j) as i128);
            }
        }
    }
}
