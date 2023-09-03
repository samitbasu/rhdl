use std::ops::Add;
use std::ops::AddAssign;

use crate::bits::Bits;

impl<const N: usize> Add<u128> for Bits<N> {
    type Output = Self;
    fn add(self, rhs: u128) -> Self::Output {
        self + Bits::<N>::from(rhs)
    }
}

impl<const N: usize> Add<Bits<N>> for u128 {
    type Output = Bits<N>;
    fn add(self, rhs: Bits<N>) -> Self::Output {
        Bits::<N>::from(self) + rhs
    }
}

impl<const N: usize> Add<Bits<N>> for Bits<N> {
    type Output = Self;
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(u128::wrapping_add(self.0, rhs.0) & Self::mask().0)
    }
}

impl<const N: usize> AddAssign<Bits<N>> for Bits<N> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const N: usize> AddAssign<u128> for Bits<N> {
    fn add_assign(&mut self, rhs: u128) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits + bits;
        assert_eq!(result.0, 180_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits + bits + bits;
        assert_eq!(result.0, 142_u128);
        let mut bits: Bits<128> = 0.into();
        bits.set_bit(127, true);
        let result = bits + bits;
        assert_eq!(result.0, 0_u128);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits + 1;
        assert_eq!(result.0, 219_u128);
        let result = 1 + bits;
        assert_eq!(result.0, 219_u128);
    }

    #[test]
    fn test_add_assign_bits() {
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits += bits;
        assert_eq!(bits.0, 180_u128);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits += bits;
        bits += bits;
        assert_eq!(bits.0, ((218 * 4) as u128) & 0xff);
        let mut bits: Bits<128> = 0.into();
        bits.set_bit(127, true);
        bits += bits;
        assert_eq!(bits.0, 0_u128);
        let mut bits: Bits<54> = 0b1101_1010.into();
        bits += 1;
        assert_eq!(bits.0, 219_u128);
    }
}
