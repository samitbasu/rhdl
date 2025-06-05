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
//! The core uses a FIFO interface for the requests and the results.  If you need
//! a stream interface, you can couple it to a set of [FIFO2Stream] cores.  In general
//! though, given the blocking nature, streams present no real benefit here.
//!
#![doc = badascii_formal!(r"
           ++BlockRWCntrl+--+       
           |  sink          |       
 ?BlockReq |                |       
 +-------->| req.data       |       
           |                |       
<----------+ req.full       |       
           |                |  AXI  
           |  - - - - - -   |<----->
           |                |       
           |                |       
           |  source        |       
?BlockReslt|                |       
<----------+ resp.data      |       
           |                |       
+--------->| resp.next      |       
           |                |       
           +----------------+       
")]
//!
//!# Internal Details
//!
//! Internally, the [BlockReadWriteController] splits the
//! request streams based
//!
use badascii_doc::badascii;
#[doc = badascii!(r"
               ++FIFO+----++          ++Map+-+                ++WriteCntrl+     
    ?BlockReq  |           |?BlockReq |      |?WriteCmd       |           |     
   +---------->|data   data+--------->|in   a+--------------->|req        |     
               |           |          |      |?AxilAddr       |        axi|<--> 
   <-----------+full   next|<+ +----->|en   b+---+ +----------+ready      |write
               +-----------+ | |      +------+   | |          |           |bus  
                             | |       +--------+|++ +--------+resp       |     
                             | |       |         +   |        |           |     
                             | |       |  +----------+   +--->|ready      |     
                   +---------+-++      |  |      +       |    +-----------+     
                   |    Control +------+  |      |       |                      
                   |            +         |      |       |                      
                   +-+--------+++         +      |       |                      
                     |         +----------------+|+----+ |    ++ReadCntrl++     
                     |                    +      |     | +    |           |read 
                     |                    |      +----+|+---->|req        |bus  
         ++FIFO+---+ | +------+ ?WriteResp|            | |    |        axi|<--> 
?BlckResp|         | +>|en   a|<----------+            +------+ready      |     
 <-------+data data|<--+      |            ?ReadResp     |    |           |     
    next |         |   |out  b|<------------------------------+resp       |     
  +----->|     full+-+ |      |                          +    |           |     
         +---------+ | ++Map+-+                          +--->|ready      |     
                     |                                   |    +-----------+     
                     +-------> ! +-----------------------+                      
")]
use badascii_doc::badascii_formal;

use rhdl::prelude::*;

use crate::{
    axi4lite::types::{
        AxilAddr, ReadMISO, ReadMOSI, ReadResult, StrobedData, WriteCommand, WriteMISO, WriteMOSI,
        WriteResult,
    },
    core::{dff::DFF, option::is_some},
    fifo::synchronous::SyncFIFO,
    stream::{fifo_to_stream::FIFOToStream, stream_to_fifo::StreamToFIFO},
};

use super::{read::ReadController, write::WriteController};

#[derive(PartialEq, Digital)]
pub enum BlockRequest {
    Write(WriteCommand),
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

#[derive(PartialEq, Digital)]
pub enum BlockResponse {
    Write(WriteResult),
    Read(ReadResult),
}

impl Default for BlockResponse {
    fn default() -> Self {
        BlockResponse::Write(Err(crate::axi4lite::types::AXI4Error::DECERR))
    }
}

#[derive(PartialEq, Digital, Default)]
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
/// returns the results in an output queue.  Uses FIFO interfaces
/// instead of stream interfaces, but you can adapt it to use
/// streams easily.  To avoid excess generics, this controller
/// has a hardwired input FIFO size of 8.
pub struct BlockReadWriteController {
    inbuf: SyncFIFO<BlockRequest, U3>,
    write_controller: WriteController,
    read_controller: ReadController,
    outbuf: SyncFIFO<BlockResponse, U3>,
    state: DFF<State>,
}

#[derive(PartialEq, Digital)]
/// Input for the [BlockReadWriteController]
pub struct In {
    pub request: Option<BlockRequest>,
    pub resp_next: bool,
    pub write_axi: WriteMISO,
    pub read_axi: ReadMISO,
}

#[derive(PartialEq, Digital)]
/// Output for the [BlockReadWriteController]
pub struct Out {
    pub response: Option<BlockResponse>,
    pub req_full: bool,
    pub write_axi: WriteMOSI,
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
    // Decide if we will issue a new command on this cycle
    let will_start = q.write_controller.req_ready
        & q.read_controller.req_ready
        & (q.state == State::Idle)
        & is_some::<BlockRequest>(q.inbuf.data);
    // Feed the write and read controllers
    d.write_controller.axi = i.write_axi;
    d.write_controller.req_data = None;
    d.read_controller.axi = i.read_axi;
    d.read_controller.req_data = None;
    d.inbuf.next = will_start;
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
    d.outbuf.next = i.resp_next;
    let can_accept = !q.outbuf.full;
    d.read_controller.resp_ready = can_accept;
    d.write_controller.resp_ready = can_accept;
    d.outbuf.data = None;
    // There is space to hold a result... check
    match q.state {
        State::Idle => {}
        State::Reading => {
            if let Some(resp) = q.read_controller.resp_data {
                d.outbuf.data = Some(BlockResponse::Read(resp));
                d.state = if can_accept { State::Idle } else { q.state };
            }
        }
        State::Writing => {
            if let Some(resp) = q.write_controller.resp_data {
                d.outbuf.data = Some(BlockResponse::Write(resp));
                d.state = if can_accept { State::Idle } else { q.state };
            }
        }
    }
    o.read_axi = q.read_controller.axi;
    o.req_full = q.inbuf.full;
    o.response = q.outbuf.data;
    o.write_axi = q.write_controller.axi;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut = BlockReadWriteController::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
