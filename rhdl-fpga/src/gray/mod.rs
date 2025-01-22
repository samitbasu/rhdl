use rhdl::prelude::*;

pub mod decode;
pub mod encode;

#[derive(PartialEq, Debug, Digital)]
pub struct Gray<N: BitWidth>(pub Bits<N>);
