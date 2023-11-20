use std::ops::{BitAnd, BitAndAssign};

use crate::{bits::Bits, signed_bits::SignedBits};

impl<const N: usize> BitAnd<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn bitand(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) & rhs
    }
}

impl<const N: usize> BitAnd<u128> for Bits<N> {
    type Output = Self;
    fn bitand(self, rhs: u128) -> Self::Output {
        self & Bits::<N>::from(rhs)
    }
}

impl<const N: usize> BitAndAssign<u128> for Bits<N> {
    fn bitand_assign(&mut self, rhs: u128) {
        *self = *self & rhs;
    }
}

impl<const N: usize> BitAnd<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn bitand(self, rhs: SignedBits<N>) -> Self::Output {
        SignedBits::<N>::from(self) & rhs
    }
}

impl<const N: usize> BitAnd<i128> for SignedBits<N> {
    type Output = Self;
    fn bitand(self, rhs: i128) -> Self::Output {
        self & SignedBits::<N>::from(rhs)
    }
}

impl<const N: usize> BitAndAssign<i128> for SignedBits<N> {
    fn bitand_assign(&mut self, rhs: i128) {
        *self = *self & rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_and_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits & bits;
        assert_eq!(result.0, 0b1101_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits & 0b1111_0000;
        assert_eq!(result.0, 0b1101_0000_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = 0b1111_0000 & bits;
        assert_eq!(result.0, 0b1101_0000_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = bits & bits;
        assert_eq!(result.0, 1_u128 << 127);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits & 1;
        assert_eq!(result.0, 0_u128);
        let result = 1 & bits;
        assert_eq!(result.0, 0_u128);
    }

    #[test]
    fn test_andassign_bits() {
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits &= bits;
        assert_eq!(bits.0, 0b1101_1010_u128);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits &= 0b1111_0000;
        assert_eq!(bits.0, 0b1101_0000_u128);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits &= 0b1111_0000;
        assert_eq!(bits.0, 0b1101_0000_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        bits &= bits;
        assert_eq!(bits.0, 1_u128 << 127);
        let mut bits: Bits<54> = 0b1101_1010.into();
        bits &= 1;
        assert_eq!(bits.0, 0_u128);
        let mut bits: Bits<54> = 0b1101_1010.into();
        bits &= 1;
        assert_eq!(bits.0, 0_u128);
        let a: Bits<12> = 0b1010_1010_1010.into();
        let b: Bits<12> = 0b1111_0101_0111.into();
        let mut c = a;
        c &= b;
        assert_eq!(c.0, 0b1010_0000_0010);
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
        let mut x = SignedBits::<8>::from(-3);
        let y = SignedBits::<8>::from(4);
        x &= y;
        assert_eq!(x.0, 4);
    }

    #[test]
    fn test_and_signed() {
        let x = SignedBits::<8>::from(-3);
        let y = SignedBits::<8>::from(4);
        let z = x & y;
        assert_eq!(z.0, 4);
    }

    #[test]
    fn test_and_signed_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let x: SignedBits<8> = (i as i128).into();
                let y: SignedBits<8> = (j as i128).into();
                let z = x & y;
                assert_eq!(z.0, (i & j).into());
            }
        }
    }

    #[test]
    fn test_and_assign_signed_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let mut x: SignedBits<8> = (i as i128).into();
                let y: SignedBits<8> = (j as i128).into();
                x &= y;
                assert_eq!(x.0, (i & j).into());
            }
        }
    }
}
