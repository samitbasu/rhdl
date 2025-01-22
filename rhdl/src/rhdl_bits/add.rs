use std::ops::Add;
use std::ops::AddAssign;

use crate::impl_assign_op;
use crate::impl_assigned_signed_op;
use crate::impl_binop;
use crate::impl_signed_binop;

use super::bits_impl::bits_masked;
use super::bits_impl::Bits;
use super::dyn_bits::DynBits;
use super::signed_bits_impl::signed_wrapped;
use super::signed_bits_impl::SignedBits;
use super::signed_dyn_bits::SignedDynBits;
use super::BitWidth;
// By default, all add operations are wrapping.

impl_binop!(Add, add, u128::wrapping_add);
impl_assign_op!(AddAssign, add_assign, u128::wrapping_add);
impl_signed_binop!(Add, add, i128::wrapping_add);
impl_assigned_signed_op!(AddAssign, add_assign, i128::wrapping_add);

#[cfg(test)]
mod test {
    use super::*;
    use crate::rhdl_bits::bitwidth::*;
    use crate::test_binop;

    #[test]
    fn test_add() {
        for i in 0..=255 {
            for j in 0..=255 {
                test_binop!(+, u128::wrapping_add, i, j);
            }
        }
    }

    #[test]
    fn test_add_bits() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let b_val = bits.val;
        let result = bits + bits;
        assert_eq!(result.val, (b_val as u8).wrapping_mul(2) as u128);
        let result = bits + bits + bits;
        assert_eq!(result.val, (b_val as u8).wrapping_mul(3) as u128);
        let mut bits: Bits<U124> = 0.into();
        bits = crate::rhdl_bits::test::set_bit(bits, 123, true);
        bits = (bits + bits).resize();
        let result = bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<U54> = 0b1101_1010.into();
        let result = bits + 1;
        assert_eq!(result.val, 219_u128);
        let result = 1 + bits;
        assert_eq!(result.val, 219_u128);
        let mut bits: Bits<U8> = 0b1101_1010.into();
        bits = (bits + bits).resize();
        assert_eq!(bits.val, 180);
        let mut bits: Bits<U8> = 0b1101_1010.into();
        bits = (bits + bits).resize();
        bits = (bits + bits).resize();
        assert_eq!(bits.val, ((218 * 4) as u128) & 0xff);
        let mut bits: Bits<U126> = 0.into();
        bits = crate::rhdl_bits::test::set_bit(bits, 125, true);
        bits = (bits + bits).resize();
        assert_eq!(bits.val, 0_u128);
        let mut bits: Bits<U54> = 0b1101_1010.into();
        bits = (bits + 1).resize();
        assert_eq!(bits.val, 219_u128);
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let i_as_signed = SignedBits::<U8>::from(i as i128);
                let j_as_signed = SignedBits::<U8>::from(j as i128);
                let k_as_signed: SignedBits<U8> = (i_as_signed + j_as_signed).resize();
                let k = i8::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i64() {
        for i in [i64::MIN, -1, 0, 1, i64::MAX] {
            for j in [i64::MIN, -1, 0, 1, i64::MAX] {
                let i_as_signed = SignedBits::<U64>::from(i as i128);
                let j_as_signed = SignedBits::<U64>::from(j as i128);
                let k_as_signed = (i_as_signed + j_as_signed).resize::<U64>();
                let k = i64::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_range() {
        eprintln!(
            "signed range {}..{}",
            SignedBits::<U9>::min_value(),
            SignedBits::<U9>::max_value()
        );
        assert_eq!(SignedBits::<U9>::min_value(), -256);
        assert_eq!(SignedBits::<U9>::max_value(), 255);
    }

    #[test]
    fn test_add_assign_signed() {
        let mut x = SignedBits::<U8>::from(1);
        x = (x + SignedBits::<U8>::from(-2)).resize();
        assert_eq!(x.val, -1);
        let z = x + 7;
        x = z.resize();
        assert_eq!(x.val, 6);
    }
}
