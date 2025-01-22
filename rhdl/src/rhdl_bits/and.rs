use std::ops::{BitAnd, BitAndAssign};

use crate::{impl_assign_op, impl_binop};

use super::bits_impl::bits_masked;
use super::{bits_impl::Bits, dyn_bits::DynBits, BitWidth};

impl_binop!(BitAnd, bitand, u128::bitand);
impl_assign_op!(BitAndAssign, bitand_assign, u128::bitand);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{rhdl_bits::bitwidth::*, test_binop};

    #[test]
    fn test_and() {
        for i in 0..=255 {
            for j in 0..=255 {
                test_binop!(&, u128::bitand, i, j);
            }
        }
    }

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
}
