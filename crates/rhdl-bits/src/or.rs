//! # Boolean OR operations via `|` and `|=`
//!
//! Use the `|` operator as usual:
//!
//! Here are a simple example of oring 2 8-bit unsigned values:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 0b1101_1010.into();
//! let b: Bits<8> = 0b0000_1010.into();
//! let c = a | b; // 0b1101_1010
//! assert_eq!(c, b8(0b1101_1010));
//! ```
//!
//! We can convert them to [DynBits] and OR them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b1101_1010.into();
//! # let b: Bits<8> = 0b0000_1010.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a | b; // 0b1101_1010
//! assert_eq!(c.as_bits::<8>(), b8(0b1101_1010));
//! ```
//!
//! You can also use the `|=` operator to OR and assign in place:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let mut a: Bits<8> = 0b0001_1010.into();
//! let b: Bits<8> = 0b0000_1010.into();
//! a |= b; // a is now 0b0001_1010
//! assert_eq!(a, b8(0b0001_1010));
//! ```
use std::ops::BitOr;
use std::ops::BitOrAssign;

use super::BitWidth;
use super::bits_impl::Bits;
use super::bits_impl::bits_masked;
use super::dyn_bits::DynBits;

impl_binop!(BitOr, bitor, u128::bitor);
impl_assign_op!(BitOrAssign, bitor_assign, u128::bitor);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_or() {
        for i in 0..=255 {
            for j in 0..=255 {
                test_binop!(|, u128::bitor, i, j);
            }
        }
    }

    #[test]
    fn test_or_bits() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits | bits;
        assert_eq!(result.raw(), 0b1101_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits | 0b1111_0000;
        assert_eq!(result.raw(), 0b1111_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = 0b1111_0000 | bits;
        assert_eq!(result.raw(), 0b1111_1010_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = bits | bits;
        assert_eq!(result.raw(), 1_u128 << 127);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits | 1;
        assert_eq!(result.raw(), 0b1101_1011_u128);
        let result = 1 | bits;
        assert_eq!(result.raw(), 0b1101_1011_u128);
        let a: Bits<12> = 0b1010_1010_1010.into();
        let b: Bits<12> = 0b0101_0101_0101.into();
        let c = a | b;
        assert_eq!(c.raw(), 0b1111_1111_1111);
    }
}
