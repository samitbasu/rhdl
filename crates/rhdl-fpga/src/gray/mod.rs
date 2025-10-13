//! Gray code encoder and decoder
use rhdl::prelude::*;
pub mod decode;
pub mod encode;

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Newtype wrapper to indicate that the underlying
/// bit vector is Gray coded
pub struct Gray<const N: usize>(pub Bits<N>)
where
    rhdl::bits::W<N>: BitWidth;
