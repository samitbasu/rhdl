#![warn(missing_docs)]
//! Stream Cores
//!
//! Stream cores are used to implement stream processing, in
//! which data elements flow and are transformed in the design.
//! They are composable, like iterators, and are meant to run
//! in high performance designs (and so carefully implement the
//! principles of latency insensitive designs).  The stream cores
//! all have registered inputs and outputs, so as to avoid
//! combinatorial pathways between the input and output.  Furthermore
//! the stream cores implement backpressure, providing both a
//! `ready` signal to upstream cores, and accepting a `ready`
//! signal from downstream cores.  Finally, the input and
//! output can be voided using [Option].  The protocol
//! implemented in the handshake is identical to the Ready/Valid
//! protocol from the AXI spec for channels.
//!

use badascii_doc::badascii;
use rhdl::prelude::Digital;
pub mod chunked;
pub mod fifo_to_stream;
pub mod filter;
pub mod filter_map;
pub mod flatten;
pub mod map;
pub mod pipe_wrapper;
pub mod stream_to_fifo;
pub mod tee;
pub mod testing;
pub mod zip;

#[derive(PartialEq, Digital)]
/// A generic Stream IO type that holds a data and ready
/// signal.  Note that in a Stream interface, these
/// signals are generally in opposite directions.
/// So a typical Stream core will look like this:
///
#[doc = badascii!("
     +-+UUT+------+     
 ?T  |            | ?S  
+--->|data    data+---->
     |            |     
     |            |     
<---+|ready  ready|<---+
     +------------+     
")]
///
///  In this case, we would have that
///```ignore
/// <UUT as SynchronousIO>::In == StreamIO<T>
/// <UUT as SynchronousIO>::Out == StreamIO<S>
///```
/// This type exists so that cores can be reused by constraining
/// the input and output types.
pub struct StreamIO<T: Digital> {
    /// The data either flowing into or out of the Pipe core
    pub data: Option<T>,
    /// The ready signal either flowing into or out of the Pipe core
    pub ready: bool,
}
