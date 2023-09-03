use std::ops::Not;

use crate::bits::Bits;

impl<const N: usize> Not for Bits<N> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0 & Self::mask().0)
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
        bits.set_bit(127, true);
        let result = !bits;
        assert_eq!(result.0, !0_u128 - (1 << 127));
        let bits: Bits<14> = 0b1101_1010.into();
        let result = !bits;
        assert_eq!(result.0, 0b0011_1111_0010_0101_u128);
    }
}
