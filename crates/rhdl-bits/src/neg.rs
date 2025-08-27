use super::signed_bits_impl::signed_wrapped;

use super::signed_dyn_bits::SignedDynBits;
use super::{BitWidth, signed_bits_impl::SignedBits};
use std::ops::Neg;

impl<N: BitWidth> Neg for SignedBits<N> {
    type Output = SignedBits<N>;
    fn neg(self) -> Self::Output {
        signed_wrapped(-self.val)
    }
}

impl Neg for SignedDynBits {
    type Output = SignedDynBits;
    fn neg(self) -> Self::Output {
        SignedDynBits {
            val: -self.val,
            bits: self.bits,
        }
        .wrapped()
    }
}

#[cfg(test)]
mod test {
    use crate::bitwidth::*;
    use crate::signed_bits_impl::SignedBits;

    #[test]
    fn test_neg_wrapping() {
        let x = i8::MIN;
        let y = x.wrapping_neg();
        assert_eq!(x, y);
    }

    #[test]
    fn test_neg_operator() {
        for i in i8::MIN..i8::MAX {
            let x = i;
            let y = x.wrapping_neg() as i16;
            let x_signed = SignedBits::<U8>::from(x as i128);
            let y_signed = -x_signed;
            assert_eq!(y_signed.val, y as i128);
        }
    }
}
