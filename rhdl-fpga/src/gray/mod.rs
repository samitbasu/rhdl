use rhdl::prelude::*;

pub mod decode;
pub mod encode;

#[derive(Debug, Digital)]
pub struct Gray<N: BitWidth>(pub Bits<N>);
