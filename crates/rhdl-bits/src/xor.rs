//! # Boolean XOR operations via `^` and `^=`
//!
//! Use the `^` operator as usual:
//!
//! Here are a simple example of xoring 2 8-bit unsigned values:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let a: Bits<8> = 0b1101_1010.into();
//! let b: Bits<8> = 0b0000_1010.into();
//! let c = a ^ b; // 0b1101_0000
//! assert_eq!(c, b8(0b1101_0000));
//! ```
//!
//! We can convert them to [DynBits] and XOR them too:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! # let a: Bits<8> = 0b1101_1010.into();
//! # let b: Bits<8> = 0b0000_1010.into();
//! let a = a.dyn_bits();
//! let b = b.dyn_bits();
//! let c = a ^ b; // 0b1101_0000
//! assert_eq!(c.as_bits::<8>(), b8(0b1101_0000));
//! ```
//!
//! You can also use the `^=` operator to XOR assign in place:
//! ```
//! # use rhdl_bits::*;
//! # use rhdl_bits::alias::*;
//! let mut a: Bits<8> = 0b0001_1010.into();
//! let b: Bits<8> = 0b0000_1010.into();
//! a ^= b; // a is now 0b0001_0000
//! assert_eq!(a, b8(0b0001_0000));
//! ```
use std::ops::BitXor;
use std::ops::BitXorAssign;

use super::BitWidth;
use super::bits_impl::Bits;
use super::bits_impl::bits_masked;
use super::dyn_bits::DynBits;

impl_binop!(BitXor, bitxor, u128::bitxor);
impl_assign_op!(BitXorAssign, bitxor_assign, u128::bitxor);

#[cfg(test)]
mod test {
    use super::*;

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
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits ^ bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits ^ 0b1111_0000;
        assert_eq!(result.val, 0b0010_1010_u128);
        let bits: Bits<8> = 0b1101_1010.into();
        let result = 0b1111_0000 ^ bits;
        assert_eq!(result.val, 0b0010_1010_u128);
        let mut bits: Bits<128> = 0.into();
        bits = crate::test::set_bit(bits, 127, true);
        let result = bits ^ bits;
        assert_eq!(result.val, 0_u128);
        let bits: Bits<54> = 0b1101_1010.into();
        let result = bits ^ 1;
        assert_eq!(result.val, 0b1101_1011_u128);
        let result = 1 ^ bits;
        assert_eq!(result.val, 0b1101_1011_u128);
        let a: Bits<12> = 0b1010_1010_1010.into();
        let b: Bits<12> = 0b0110_0100_0000.into();
        let c: Bits<12> = 0b1100_1110_1010.into();
        assert_eq!(a ^ b, c);
    }
}
