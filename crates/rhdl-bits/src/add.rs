//! # Addition operations via `+` and `+=`
//!
//! By default, all add operations are wrapping.  Use the `+` operator as usual:
//!
//! Here are a simple example of adding 2 8-bit unsigned values:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 20.into();
//! let b: Bits<8> = 10.into();
//! let c = a + b; // 30
//! assert_eq!(c, b8(30));
//! ```
//!
//! We can convert them to [DynBits] and add them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 20.into();
//! # let b: Bits<8> = 10.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a + b; // 30
//! assert_eq!(c.as_bits::<8>(), b8(30));
//! ```
//!
//! When working with signed values, remember to put parentheses around negative literals:
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a : SignedBits<8> = 120.into();
//! let b : SignedBits<8> = (-10).into();
//! let c = a + b; // 110
//! assert_eq!(c, s8(110));
//! ```
//!
//! Finally, here is a signed dynamic example:
//!
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a : SignedBits<8> = 120.into();
//! # let b : SignedBits<8> = (-10).into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a + b; // 110
//! assert_eq!(c.as_signed_bits::<8>(), s8(110));
//! ```
//!
//! Note that you cannot mix unsigned and signed types, any more than you can add `i8` and `u8` in normal Rust.
//!
//!
//! You can also use the `+=` operator to add and assign in place:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let mut a: Bits<8> = 20.into();
//! let b: Bits<8> = 10.into();
//! a += b; // a is now 30
//! assert_eq!(a, b8(30));
//! ```

use std::ops::Add;
use std::ops::AddAssign;

use super::BitWidth;
use super::bits_impl::Bits;
use super::bits_impl::bits_masked;
use super::dyn_bits::DynBits;
use super::signed_bits_impl::SignedBits;
use super::signed_bits_impl::signed_wrapped;
use super::signed_dyn_bits::SignedDynBits;
// By default, all add operations are wrapping.

impl_binop!(Add, add, u128::wrapping_add);
impl_assign_op!(AddAssign, add_assign, u128::wrapping_add);
impl_signed_binop!(Add, add, i128::wrapping_add);
impl_assigned_signed_op!(AddAssign, add_assign, i128::wrapping_add);

#[cfg(test)]
mod test {
    use super::*;

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
        let bits: Bits<8> = 0b1101_1010.into();
        let b_val = bits.val;
        let result = bits + bits;
        assert_eq!(result.val, (b_val as u8).wrapping_mul(2) as u128);
        let result = bits + bits + bits;
        assert_eq!(result.val, (b_val as u8).wrapping_mul(3) as u128);
        let mut bits: Bits<124> = 0.into();
        bits = crate::test::set_bit(bits, 123, true);
        bits = (bits + bits).resize();
        let result = bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits + 1;
        assert_eq!(result.val, 219_u128);
        let result = 1 + bits;
        assert_eq!(result.val, 219_u128);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits = (bits + bits).resize();
        assert_eq!(bits.val, 180);
        let mut bits: Bits<8> = 0b1101_1010.into();
        bits = (bits + bits).resize();
        bits = (bits + bits).resize();
        assert_eq!(bits.val, ((218 * 4) as u128) & 0xff);
        let mut bits: Bits<126> = 0.into();
        bits = crate::test::set_bit(bits, 125, true);
        bits = (bits + bits).resize();
        assert_eq!(bits.val, 0_u128);
        let mut bits: Bits<54> = 0b1101_1010.into();
        bits = (bits + 1).resize();
        assert_eq!(bits.val, 219_u128);
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i8() {
        for i in i8::MIN..i8::MAX {
            for j in i8::MIN..i8::MAX {
                let i_as_signed = SignedBits::<8>::from(i as i128);
                let j_as_signed = SignedBits::<8>::from(j as i128);
                let k_as_signed: SignedBits<8> = (i_as_signed + j_as_signed).resize();
                let k = i8::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_addition_matches_built_in_behavior_for_i64() {
        for i in [i64::MIN, -1, 0, 1, i64::MAX] {
            for j in [i64::MIN, -1, 0, 1, i64::MAX] {
                let i_as_signed = SignedBits::<64>::from(i as i128);
                let j_as_signed = SignedBits::<64>::from(j as i128);
                let k_as_signed = (i_as_signed + j_as_signed).resize::<64>();
                let k = i64::wrapping_add(i, j);
                assert_eq!(k_as_signed.val, k as i128);
            }
        }
    }

    #[test]
    fn test_signed_range() {
        assert_eq!(SignedBits::<9>::min_value(), -256);
        assert_eq!(SignedBits::<9>::max_value(), 255);
    }

    #[test]
    fn test_add_assign_signed() {
        let mut x = SignedBits::<8>::from(1);
        x = (x + SignedBits::<8>::from(-2)).resize();
        assert_eq!(x.val, -1);
        let z = x + 7;
        x = z.resize();
        assert_eq!(x.val, 6);
    }
}
