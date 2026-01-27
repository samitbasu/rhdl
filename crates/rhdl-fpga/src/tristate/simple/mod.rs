use rhdl::prelude::*;
// Not sure how useful this will be as a production widget, but
// it demonstrates the use of a tri-state line to communicate
// between two modules.
mod receiver;
mod sender;
mod testing;

// This captures the state of the line (either leader is writing or reading)
#[derive(PartialEq, Debug, Default, Digital, Clone, Copy)]
pub enum LineState {
    Write,
    #[default]
    Read,
}

#[derive(Digital, Clone, Copy, Debug, PartialEq, Default)]
pub struct BitZ<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    pub value: Bits<N>,
    pub mask: Bits<N>,
}
