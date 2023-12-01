use std::ops::Shl;
use std::ops::ShlAssign;

use crate::bits_impl::Bits;
use crate::signed_bits_impl::SignedBits;

impl<const N: usize> Shl<u128> for Bits<N> {
    type Output = Self;
    fn shl(self, rhs: u128) -> Self::Output {
        self << Bits::<8>::from(rhs)
    }
}

impl<const N: usize> Shl<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn shl(self, rhs: Bits<N>) -> Self::Output {
        assert!(N <= 8, "Shift amount must be less than 8 bits");
        Bits::<N>::from(self) << rhs
    }
}

impl<const M: usize, const N: usize> Shl<Bits<M>> for Bits<N> {
    type Output = Self;
    fn shl(self, rhs: Bits<M>) -> Self::Output {
        assert!(M <= 8, "Shift amount must be less than 8 bits");
        Self(u128::wrapping_shl(self.0, rhs.0 as u32) & Self::mask().0)
    }
}

impl<const M: usize, const N: usize> ShlAssign<Bits<M>> for Bits<N> {
    fn shl_assign(&mut self, rhs: Bits<M>) {
        *self = *self << rhs;
    }
}

impl<const N: usize> ShlAssign<u128> for Bits<N> {
    fn shl_assign(&mut self, rhs: u128) {
        *self = *self << rhs;
    }
}

impl<const N: usize> Shl<u128> for SignedBits<N> {
    type Output = Self;
    fn shl(self, rhs: u128) -> Self::Output {
        self << Bits::<8>::from(rhs)
    }
}

impl<const N: usize> Shl<Bits<N>> for i128 {
    type Output = SignedBits<N>;
    fn shl(self, rhs: Bits<N>) -> Self::Output {
        assert!(N <= 8, "Shift amount must be less than 8 bits");
        SignedBits::<N>::from(self) << rhs
    }
}

impl<const M: usize, const N: usize> Shl<Bits<M>> for SignedBits<N> {
    type Output = Self;
    fn shl(self, rhs: Bits<M>) -> Self::Output {
        assert!(M <= 8, "Shift amount must be less than 8 bits");
        (self.as_unsigned() << rhs).as_signed()
    }
}

impl<const M: usize, const N: usize> ShlAssign<Bits<M>> for SignedBits<N> {
    fn shl_assign(&mut self, rhs: Bits<M>) {
        *self = *self << rhs;
    }
}

impl<const N: usize> ShlAssign<u128> for SignedBits<N> {
    fn shl_assign(&mut self, rhs: u128) {
        *self = *self << rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shl_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits << 4;
        assert_eq!(result.0, 0b1010_0000_u128);
        let bits: Bits<16> = 0b0000_0000_1101_1010.into();
        let result = bits << 8;
        assert_eq!(result.0, 0b1101_1010_0000_0000_u128);
        let shift: Bits<8> = 8.into();
        let result = bits << shift;
        assert_eq!(result.0, 0b1101_1010_0000_0000_u128);
    }

    #[test]
    fn test_shl_signed_bits() {
        let bits: SignedBits<8> = (-38).into();
        let result = bits << 1;
        assert_eq!(result.0, -76_i128);
        for shift in 0..10 {
            let bits: SignedBits<8> = (-38).into();
            let result = bits << shift;
            assert_eq!(result.0, ((-38_i128 << shift) as i8).into());
            let shift_as_bits: Bits<8> = shift.into();
            let result = bits << shift_as_bits;
            assert_eq!(result.0, ((-38_i128 << shift) as i8).into());
        }
    }

    #[test]
    fn test_shl_assign_signed_bits() {
        let mut bits: SignedBits<8> = (-38).into();
        bits <<= 1;
        assert_eq!(bits.0, -76_i128);
        for shift in 0..10 {
            let mut bits: SignedBits<8> = (-38).into();
            bits <<= shift;
            assert_eq!(bits.0, ((-38_i128 << shift) as i8).into());
            let shift_as_bits: Bits<8> = shift.into();
            let mut bits: SignedBits<8> = (-38).into();
            bits <<= shift_as_bits;
            assert_eq!(bits.0, ((-38_i128 << shift) as i8).into());
        }
    }
}
