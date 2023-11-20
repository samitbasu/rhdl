use std::ops::BitOr;
use std::ops::BitOrAssign;

use crate::bits::Bits;
use crate::signed_bits::SignedBits;

impl<const N: usize> BitOr<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn bitor(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) | rhs
    }
}

impl<const N: usize> BitOr<u128> for Bits<N> {
    type Output = Self;
    fn bitor(self, rhs: u128) -> Self::Output {
        self | Bits::<N>::from(rhs)
    }
}

impl<const N: usize> BitOrAssign<u128> for Bits<N> {
    fn bitor_assign(&mut self, rhs: u128) {
        *self = *self | rhs;
    }
}

impl<const N: usize> BitOr<SignedBits<N>> for i128 {
    type Output = SignedBits<N>;
    fn bitor(self, rhs: SignedBits<N>) -> Self::Output {
        SignedBits::<N>::from(self) | rhs
    }
}

impl<const N: usize> BitOr<i128> for SignedBits<N> {
    type Output = Self;
    fn bitor(self, rhs: i128) -> Self::Output {
        self | SignedBits::<N>::from(rhs)
    }
}

impl<const N: usize> BitOrAssign<i128> for SignedBits<N> {
    fn bitor_assign(&mut self, rhs: i128) {
        *self = *self | rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_or_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits | bits;
        assert_eq!(result.0, 0b1101_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits | 0b1111_0000;
        assert_eq!(result.0, 0b1111_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = 0b1111_0000 | bits;
        assert_eq!(result.0, 0b1111_1010_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = bits | bits;
        assert_eq!(result.0, 1_u128 << 127);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits | 1;
        assert_eq!(result.0, 0b1101_1011_u128);
        let result = 1 | bits;
        assert_eq!(result.0, 0b1101_1011_u128);
        let a: Bits<12> = 0b1010_1010_1010.into();
        let b: Bits<12> = 0b0101_0101_0101.into();
        let c = a | b;
        assert_eq!(c.0, 0b1111_1111_1111);
    }

    #[test]
    fn test_or_signed_bits() {
        let bits: SignedBits<8> = (-38).into();
        let result = bits | bits;
        assert_eq!(result.0, (-38).into());
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let bits: SignedBits<8> = (i as i128).into();
                let result = bits | (j as i128);
                assert_eq!(result.0, (i | j).into());
            }
        }
    }

    #[test]
    fn test_or_assign_signed_bits() {
        let mut bits: SignedBits<8> = (-38).into();
        bits |= bits;
        assert_eq!(bits.0, (-38).into());
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let mut bits: SignedBits<8> = (i as i128).into();
                bits |= j as i128;
                assert_eq!(bits.0, (i | j).into());
            }
        }
    }
}
