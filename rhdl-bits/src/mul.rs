use std::ops::Mul;

use rhdl_macro::mul_impl;
use seq_macro::seq;

use crate::bits_impl::Bits;
use crate::signed_bits_impl::SignedBits;

seq!(N in 1..=48 {
    mul_impl!(N);
});
