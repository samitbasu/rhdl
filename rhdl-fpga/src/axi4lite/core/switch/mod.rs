//! Cores used to implement AXI4Lite Switches
//!
//! These cores are used to build AXI4L switches.
//! The switches contain both read switches and
//! write switches.  A switch is simply a multiplexor
//! for a set of masters and endpoints.  Roughly, something
//! like this:
//!
#![doc = badascii!(r"
                 +-+Switch+--+               
                 |           |               
M0   <--+AXI+--->|S0       M0|<----+AXI+---->
                 |           |               
                 |           |               
                 |         M1|<----+AXI+---->
                 |           |               
                 |           |               
                 |         M2|<----+AXI+---->
                 |           |               
                 +-----------+               
")]
//!
//! The switch operates by forming a temporary `channel` to connect
//! a request stream to an AXI endpoint, and will keep that channel
//! in place until the incoming request requires a channel change.
//! At that point, to guarantee ordering of requests, the switch will
//! wait for the pending transactions to complete before moving
//! the channel to a different endpoint.  Internally, the switch is
//! similar to the [BlockReadWriteController], in that it must track
//! the number of pending transactions to know when it is safe to
//! switch channels.

use badascii_doc::badascii;
pub mod read;
