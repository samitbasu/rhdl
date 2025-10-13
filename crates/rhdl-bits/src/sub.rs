use std::ops::Sub;
use std::ops::SubAssign;

use crate::impl_assign_op;
use crate::impl_assigned_signed_op;
use crate::impl_binop;
use crate::impl_signed_binop;

use super::BitWidth;
use super::bits_impl::Bits;
use super::bits_impl::bits_masked;
use super::dyn_bits::DynBits;
use super::signed_bits_impl::SignedBits;
use super::signed_bits_impl::signed_wrapped;
use super::signed_dyn_bits::SignedDynBits;

impl_binop!(Sub, sub, u128::wrapping_sub);
impl_assign_op!(SubAssign, sub_assign, u128::wrapping_sub);
impl_signed_binop!(Sub, sub, i128::wrapping_sub);
impl_assigned_signed_op!(SubAssign, sub_assign, i128::wrapping_sub);

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_binop;

    #[test]
    fn test_sub() {
        for i in 0..=255 {
            for j in 0..=255 {
                test_binop!(-, u128::wrapping_sub, i, j);
            }
        }
    }

    #[test]
    fn test_sub_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits - bits;
        assert_eq!(result.val, 0);
        let bits: SignedBits<8> = 0b0101_1010.into();
        let result = bits - bits - bits;
        assert_eq!(result.val, -bits.val);
        let mut bits: Bits<126> = 0.into();
        bits = crate::test::set_bit(bits, 125, true);
        let result = bits - bits;
        assert_eq!(result.val, 0);
        let bits: Bits<54> = 0b1101_1010.into();
        let x = bits.val;
        let result = bits - 1;
        let bits_m_1: Bits<54> = 0b1101_1001.into();
        assert_eq!(result, bits_m_1);
        let result = 1 - bits;
        // The 2s complement equivalent of 1 - x is 1 + (x::mask() - x) + 1
        // which is 2 + (x::mask() - x)
        assert_eq!(
            result,
            SignedBits::<54>::from(1 - (x as i128)).as_unsigned()
        );
    }

    #[test]
    fn test_subassign_bits() {
        let mut bits: Bits<8> = 0b1101_1010.into();
        let bits_m_1: Bits<8> = 0b1101_1001.into();
        bits -= bits_m_1;
        assert_eq!(bits.val, 1_u128);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits -= 1;
        assert_eq!(bits.val, 0b1101_1001_u128);
    }

    #[test]
    fn test_subtraction_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let signed_i: SignedBits<8> = (i as i128).into();
                let signed_j: SignedBits<8> = (j as i128).into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i.wrapping_sub(j) as i128;
                assert_eq!(signed_k.val, built_in_k);
            }
        }
    }

    #[test]
    fn test_subtraction_i128() {
        let min = SignedBits::<128>::min_value();
        let max = SignedBits::<128>::max_value();
        for i in [min, -1, 0, 1, max] {
            for j in [min, -1, 0, 1, max] {
                let signed_i: SignedBits<128> = i.into();
                let signed_j: SignedBits<128> = j.into();
                let signed_k = signed_i - signed_j;
                let built_in_k = i.wrapping_sub(j);
                assert_eq!(signed_k.val, built_in_k);
            }
        }
    }

    #[test]
    fn test_subassign() {
        let mut x = SignedBits::<8>::from(1);
        x -= -2;
        assert_eq!(x.val, 3);
    }
}
