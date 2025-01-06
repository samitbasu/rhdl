use crate::{signed, signed_bits_impl::SignedBits};
use rhdl_typenum::*;
use std::ops::{Add, Neg};

impl<N> Neg for SignedBits<N>
where
    N: BitWidth + Add<W1>,
    Sum<N, W1>: BitWidth,
{
    type Output = SignedBits<Sum<N, W1>>;
    fn neg(self) -> Self::Output {
        signed(-self.val)
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
            let y = -(x as i16);
            let x_signed = SignedBits::<W8>::from(x as i128);
            let y_signed = -x_signed;
            assert_eq!(y_signed.val, y as i128);
        }
    }
}
