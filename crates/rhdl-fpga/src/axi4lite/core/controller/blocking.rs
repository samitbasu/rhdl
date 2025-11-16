//! AXI4Lite Blocking ReadWrite Controller
//!
//!# Purpose
//!
//! This core provides a simple blocking interface to issue a sequence
//! of read and write requests to an AXI bus using a single input and
//! output stream.  Each request is blocked until the response is received,
//! so that only one transaction is in flight at a time.  
//!
//!# Schematic Symbol
//!
//! The core sinks a stream of [BlockRequest]s and then sources a
//! stream of [BlockResponse].  The [BlockRequest] stream will
//! generally be stalled, as new requests are only accepted when the
//! previous request has completed.
//!
#![doc = badascii_formal!(r"
           ++BlockRWCntrl+--+       
           |  sink          |       
 ?BlockReq |                |       
 +-------->| req.data       |       
R<BlockReq>|                |       
<----------+ req.ready      |       
           |                |  AXI  
           |  - - - - - -   |<----->
           |                |       
           |                |       
           |  source        |       
?BlockResp |                |       
<----------+ resp.data      |       
R<BlockRsp>|                |       
+--------->| resp.ready     |       
           |                |       
           +----------------+       
")]
//!
//!# Internal Details
//!
//! Internally, the [BlockReadWriteController] splits the
//! request streams based on the variant of the [BlockRequest]
//! but will also blank/void out the request if it is
//! waiting for the previous transaction to complete.
//! The backpressure signal for the output (response) stream
//! comes from the external `resp_ready` signal.
//!
use badascii_doc::badascii;
#[doc = badascii!(r"
        ++StrmBuf++                   ++Map+-+                ++WriteCntrl+     
 ?BlkReq|         | ?BlockReq         |      |?WriteCmd       |           |     
 +----->|data data+------------------>|in   a+--------------->|req        |     
R<BlkRq>|         | ready             |      |?AxilAddr       |        axi|<--> 
 <------+rdy   rdy|<---------+ +----->|en   b+---+ +----------+ready      |write
        +---------+          | |      +------+   | |          |           |bus  
                             | |       +--------+|++ +--------+resp       |     
                             | |       |         +   |        |           |     
                             | |       |  +----------+    +-->|ready      |     
                   +---------+-++      |  |      +        |   +-----------+     
                   |    Control |<-----+  |      |        |                     
                   +-+----------+         |      |        |                     
                          ^   ^           +      |        |                     
                          |   +-----------------+|+----+  |   ++ReadCntrl++     
                   is_some|               +      |     |  +   |           |read 
                          |               |      +----+|+---->|req        |bus  
       +StmBuf++?Block +--+---+ ?WriteResp|            |  |   |        axi|<--> 
  ?BRsp|       | Resp  |     a|<----------+            +------+ready      |     
  <----+       |<------+      |            ?ReadResp      |   |           |     
       |       |       |out  b|<------------------------------+resp       |     
R<BRsp>|       |ready  |      |                           +   |           |     
 +---->|       +----+  ++Map+-+                           +-->|ready      |     
       +-------+    |                                     |   +-----------+     
                    +-------------------------------------+                     
")]
use badascii_doc::badascii_formal;

use rhdl::prelude::*;

use crate::{
    axi4lite::types::{
        AxilAddr, ReadMISO, ReadMOSI, ReadResult, StrobedData, WriteCommand, WriteMISO, WriteMOSI,
        WriteResult,
    },
    core::{dff::DFF, option::is_some},
    stream::{stream_buffer::StreamBuffer, Ready},
};

use super::{read::ReadController, write::WriteController};

#[derive(PartialEq, Clone, Copy, Digital)]
/// Make a blocking read or write request on an AXI bus
pub enum BlockRequest {
    /// Make a blocking write request with the given [WriteCommand]
    Write(WriteCommand),
    /// Make a blocking read request to the given [AxilAddr]
    Read(AxilAddr),
}

impl Default for BlockRequest {
    fn default() -> Self {
        BlockRequest::Write(WriteCommand {
            addr: bits(0),
            strobed_data: StrobedData {
                data: bits(0),
                strobe: bits(0),
            },
        })
    }
}

#[derive(PartialEq, Clone, Copy, Digital)]
/// The response to a [BlockRequest]
pub enum BlockResponse {
    /// The response to a write [BlockRequest] in the form of a [WriteResult]
    Write(WriteResult),
    /// The response to a read [BlockRequest] in the form of a [ReadResult]
    Read(ReadResult),
}

impl Default for BlockResponse {
    fn default() -> Self {
        BlockResponse::Write(Err(crate::axi4lite::types::AXI4Error::DECERR))
    }
}

#[derive(PartialEq, Digital, Clone, Copy, Default)]
#[doc(hidden)]
pub enum State {
    #[default]
    Idle,
    Writing,
    Reading,
}

#[derive(Clone, Synchronous, SynchronousDQ, Default)]
/// Blocking AXI Read/Write Controller
///
/// Executes a sequence of read/write commands provided
/// at the input one at a time in a blocking fashion, and
/// sources a stream with the output results.
pub struct BlockReadWriteController {
    inbuf: StreamBuffer<BlockRequest>,
    write_controller: WriteController,
    read_controller: ReadController,
    outbuf: StreamBuffer<BlockResponse>,
    state: DFF<State>,
}

#[derive(PartialEq, Clone, Copy, Digital)]
/// Input for the [BlockReadWriteController]
pub struct In {
    /// The [BlockRequest] input for the request stream
    pub request: Option<BlockRequest>,
    /// Backpressure/ready signal for the response stream
    pub resp_ready: Ready<BlockResponse>,
    /// The input side of the write AXI bus
    pub write_axi: WriteMISO,
    /// The input side of the read AXI bus
    pub read_axi: ReadMISO,
}

#[derive(PartialEq, Clone, Copy, Digital)]
/// Output for the [BlockReadWriteController]
pub struct Out {
    /// The [BlockResponse] stream output
    pub response: Option<BlockResponse>,
    /// Backpressure/ready signal for the request stream
    pub req_ready: Ready<BlockRequest>,
    /// The output side of the write AXI bus
    pub write_axi: WriteMOSI,
    /// The output side of the read AXI bus
    pub read_axi: ReadMOSI,
}

impl SynchronousIO for BlockReadWriteController {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    let mut o = Out::dont_care();
    d.state = q.state;
    d.inbuf.data = i.request;
    // First check to see if we can capture a response
    let mut will_unload = false;
    let can_accept = q.outbuf.ready.raw;
    d.read_controller.resp_ready.raw = can_accept;
    d.write_controller.resp_ready.raw = can_accept;
    d.outbuf.data = None;
    // There is space to hold a result... check
    match q.state {
        State::Idle => {}
        State::Reading => {
            if let Some(resp) = q.read_controller.resp_data {
                d.outbuf.data = Some(BlockResponse::Read(resp));
                if can_accept {
                    d.state = State::Idle;
                    will_unload = true;
                }
            }
        }
        State::Writing => {
            if let Some(resp) = q.write_controller.resp_data {
                d.outbuf.data = Some(BlockResponse::Write(resp));
                if can_accept {
                    d.state = State::Idle;
                    will_unload = true;
                }
            }
        }
    }
    // Decide if we will issue a new command on this cycle
    let will_start = q.write_controller.req_ready.raw
        & q.read_controller.req_ready.raw
        & ((q.state == State::Idle) | will_unload)
        & is_some::<BlockRequest>(q.inbuf.data);
    // Feed the write and read controllers
    d.write_controller.axi = i.write_axi;
    d.write_controller.req_data = None;
    d.read_controller.axi = i.read_axi;
    d.read_controller.req_data = None;
    d.inbuf.ready.raw = will_start;
    if will_start {
        if let Some(req) = q.inbuf.data {
            match req {
                BlockRequest::Read(read) => {
                    d.state = State::Reading;
                    d.read_controller.req_data = Some(read);
                }
                BlockRequest::Write(write) => {
                    d.state = State::Writing;
                    d.write_controller.req_data = Some(write);
                }
            };
        }
    }
    d.outbuf.ready = i.resp_ready;
    o.read_axi = q.read_controller.axi;
    o.req_ready = q.inbuf.ready;
    o.response = q.outbuf.data;
    o.write_axi = q.write_controller.axi;
    (o, d)
}

#[cfg(test)]
mod tests {
    use rhdl::{core::circuit::scoped_name::ScopedName, prelude::vlog::Pretty};

    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut = BlockReadWriteController::default();
        let descriptor = uut.descriptor(ScopedName::top())?;
        let hdl = descriptor.hdl()?;
        let module = hdl.modules.pretty();
        expect_test::expect_file!["blocking_controller.vlog"].assert_eq(&module);
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
