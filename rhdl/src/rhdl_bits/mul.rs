use std::ops::Mul;

use crate::{impl_binop, impl_signed_binop};

use super::dyn_bits::DynBits;
use super::signed_dyn_bits::SignedDynBits;
use super::{bits_impl::bits_masked, signed_bits_impl::signed_wrapped, BitWidth, Bits, SignedBits};

impl_binop!(Mul, mul, u128::wrapping_mul);
impl_signed_binop!(Mul, mul, i128::wrapping_mul);

#[cfg(test)]
mod tests {
    use crate::rhdl_bits::bits;
    use crate::rhdl_bits::bits_impl::bits_masked;
    use crate::rhdl_bits::bitwidth::*;
    use crate::test_binop;

    #[test]
    fn test_muls() {
        for i in 0..=255 {
            for j in 0..=255 {
                test_binop!(*, i, j);
            }
        }
    }

    #[test]
    fn test_mul() {
        let a = bits::<U32>(0x1234_5678);
        let b = bits::<U32>(0x8765_4321);
        let c = a * b;
        assert_eq!(
            c,
            bits::<U32>(0x1234_5678_u32.wrapping_mul(0x8765_4321) as u128)
        );
    }
}
