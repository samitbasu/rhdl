use crate::{bitwidth::*, signed_bits_impl::signed_wrapped};

use crate::{signed, signed_bits_impl::SignedBits, BitWidth};
use std::ops::{Add, Neg};

impl<N: BitWidth> Neg for SignedBits<N> {
    type Output = SignedBits<N>;
    fn neg(self) -> Self::Output {
        signed_wrapped(-self.val)
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
