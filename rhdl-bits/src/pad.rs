use rhdl_macro::pad_impl;
use seq_macro::seq;

use crate::bits_impl::Bits;
use crate::signed_bits_impl::SignedBits;

pub trait Pad {
    type Output;

    fn pad(self) -> Self::Output;
}

seq!(N in 1..=127 {
    pad_impl!(N);
});
