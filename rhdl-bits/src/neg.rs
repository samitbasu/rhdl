use crate::signed_bits::SignedBits;
use std::ops::Neg;

impl<const N: usize> Neg for SignedBits<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        !self + 1
    }
}

#[cfg(test)]
mod test {
    use crate::signed_bits::SignedBits;

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
            let x_signed = SignedBits::<8>::from(x as i128);
            let y_signed = -x_signed;
            assert_eq!(y_signed.0, y as i128);
        }
    }
}
