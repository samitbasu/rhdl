use rhdl::prelude::*;

pub mod decode;
pub mod encode;

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct Gray<const N: usize>(pub Bits<N>);
