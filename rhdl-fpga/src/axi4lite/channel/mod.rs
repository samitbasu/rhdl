/// An AXI channel is a simple handshake based mechanism to transfer data from one
/// point to another.  The data itself can be anything, so we model that as generic.
/// The data is transferred in a single direction using a pair of handshake signals.
/// The channel sender and the channel receiver both run state machines to manage the
/// handshake.  In order to be flexible, the channel sender and receiver do not include
/// internal buffers.  Buffering is handled by the user of the channel.
use rhdl::prelude::*;
pub mod receiver;
pub mod sender;
pub mod testing;

#[derive(PartialEq, Debug, Digital, Default)]
pub struct Ready {
    pub ready: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct DataValid<T: Digital> {
    pub data: T,
    pub valid: bool,
}

impl<T: Digital + Default> Default for DataValid<T> {
    fn default() -> Self {
        Self {
            data: T::default(),
            valid: false,
        }
    }
}
