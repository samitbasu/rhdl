#![warn(missing_docs)]
//! Pipe Cores
//!
//! Pipe cores are used to implement stream processing, in
//! which data elements flow and are transformed in the design.
//! They are composable, like iterators, and are meant to run
//! in high performance designs (and so carefully implement the
//! principles of latency insensitive designs).  The pipe cores
//! all have registered inputs and outputs, so as to avoid
//! combinatorial pathways between the input and output.  Furthermore
//! the pipe cores implement backpressure, providing both a
//! `ready` signal to upstream cores, and accepting a `ready`
//! signal from downstream cores.  Finally, the input and
//! output can be voided using [Option].  The protocol
//! implemented in the handshake is identical to the Ready/Valid
//! protocol from the AXI spec for channels.
//!

use badascii_doc::badascii;
use rhdl::prelude::Digital;
pub mod chunked;
pub mod filter;
pub mod filter_map;
pub mod flatten;
pub mod map;
pub mod testing;
pub mod zip;

#[derive(PartialEq, Digital)]
/// A generic Pipe type that holds a data and ready
/// signal.  Note that in a Pipe interface, these
/// signals are generally in opposite directions.
/// So a typical Pipe core will look like this:
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
/// <UUT as SynchronousIO>::In == PipeIO<T>
/// <UUT as SynchronousIO>::Out == PipeIO<S>
///```
/// This type exists so that cores can be reused by constraining
/// the input and output types.
pub struct PipeIO<T: Digital> {
    /// The data either flowing into or out of the Pipe core
    pub data: Option<T>,
    /// The ready signal either flowing into or out of the Pipe core
    pub ready: bool,
}
