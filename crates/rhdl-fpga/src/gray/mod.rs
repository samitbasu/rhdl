//! Gray code encoder and decoder
use rhdl::prelude::*;
pub mod decode;
pub mod encode;

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Newtype wrapper to indicate that the underlying
/// bit vector is Gray coded
pub struct Gray<N: BitWidth>(pub Bits<N>);
