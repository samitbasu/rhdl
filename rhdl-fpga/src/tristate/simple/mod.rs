use rhdl::prelude::*;
// Not sure how useful this will be as a production widget, but
// it demonstrates the use of a tri-state line to communicate
// between two modules.
mod receiver;
mod sender;
mod testing;

// This captures the state of the line (either leader is writing or reading)
#[derive(PartialEq, Debug, Default, Digital)]
pub enum LineState {
    Write,
    #[default]
    Read,
}
