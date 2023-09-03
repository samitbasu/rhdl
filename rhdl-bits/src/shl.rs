use std::ops::Shl;
use std::ops::ShlAssign;

use crate::bits::Bits;

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

impl<const N: usize> ShlAssign<Bits<N>> for Bits<N> {
    fn shl_assign(&mut self, rhs: Self) {
        *self = *self << rhs;
    }
}

impl<const N: usize> ShlAssign<u128> for Bits<N> {
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
}
