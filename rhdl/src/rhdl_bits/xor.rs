use std::ops::BitXor;
use std::ops::BitXorAssign;

use crate::impl_assign_op;
use crate::impl_binop;

use super::bits_impl::bits_masked;
use super::bits_impl::Bits;
use super::dyn_bits::DynBits;
use super::BitWidth;

impl_binop!(BitXor, bitxor, u128::bitxor);
impl_assign_op!(BitXorAssign, bitxor_assign, u128::bitxor);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{rhdl_typenum::prelude::*, test_binop};

    #[test]
    fn test_xor() {
        for i in 0..=255 {
            for j in 0..=255 {
                test_binop!(^, u128::bitxor, i, j);
            }
        }
    }

    #[test]
    fn test_xor_bits() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits ^ bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits ^ 0b1111_0000;
        assert_eq!(result.val, 0b0010_1010_u128);
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = 0b1111_0000 ^ bits;
        assert_eq!(result.val, 0b0010_1010_u128);
        let mut bits: Bits<U128> = 0.into();
        bits = crate::rhdl_bits::test::set_bit(bits, 127, true);
        let result = bits ^ bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<U54> = 0b1101_1010.into();
        let result = bits ^ 1;
        assert_eq!(result.val, 0b1101_1011_u128);
        let result = 1 ^ bits;
        assert_eq!(result.val, 0b1101_1011_u128);
        let a: Bits<U12> = 0b1010_1010_1010.into();
        let b: Bits<U12> = 0b0110_0100_0000.into();
        let c: Bits<U12> = 0b1100_1110_1010.into();
        assert_eq!(a ^ b, c);
    }
}
