use crate::signed_bits_impl::SignedBits;
use rhdl_typenum::BitWidth;
use std::ops::Neg;

impl<N: BitWidth> Neg for SignedBits<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            val: self.val.wrapping_neg(),
            marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::signed_bits_impl::SignedBits;
    use rhdl_typenum::W8;

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
            let y = x.wrapping_neg();
            let x_signed = SignedBits::<W8>::from(x as i128);
            let y_signed = -x_signed;
            assert_eq!(y_signed.val, y as i128);
        }
    }
}
