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

#[derive(Clone, Copy, Debug, PartialEq, Digital, Default)]
pub struct ChannelRToS {
    pub ready: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct ChannelSToR<T: Digital> {
    pub data: T,
    pub valid: bool,
}

impl<T: Digital> Default for ChannelSToR<T> {
    fn default() -> Self {
        Self {
            data: T::init(),
            valid: false,
        }
    }
}
