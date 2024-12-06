use rhdl::prelude::*;

pub mod decode;
pub mod encode;

#[derive(Debug, Digital)]
pub struct Gray<const N: usize>(pub Bits<N>);
