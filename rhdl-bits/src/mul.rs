use std::ops::Mul;

use rhdl_macro::mul_impl;
use seq_macro::seq;

use crate::bits_impl::Bits;
use crate::signed_bits_impl::SignedBits;

seq!(N in 1..=48 {
    mul_impl!(N);
});

/*impl Mul<SignedBits<8>> for SignedBits<8> {
    type Output = SignedBits<16>;
    fn mul(self, rhs: SignedBits<8>) -> Self::Output {
        SignedBits::from(self.0 * rhs.0)
    }
}

impl Mul<Bits<8>> for Bits<8> {
    type Output = Bits<16>;
    fn mul(self, rhs: Bits<8>) -> Self::Output {
        Bits::from(self.0 * rhs.0)
    }
}
*/
