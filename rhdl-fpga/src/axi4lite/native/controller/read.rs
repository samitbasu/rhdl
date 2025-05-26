//! AXI4Lite Read Controller Interface
//!
//!# Purpose
//!
//! This core provides a way to build an AXI read controller using a
//! pair of RHDL streams.  One is a stream for addresses to read
//! from the AXI bus.  The other is a stream of responses from
//! the bus back to the controller.  Each request must generate
//! a response.  This core can be paired up with the [ReadEndpoint]
//! core to transport the RHDL stream across an AXI bus interface
//! in a spec compliant manner.
//!
//!# Schematic Symbol
//!
//! Here is the symbol for the core.  The core will sink a stream
//! of addresses (which are [AxilAddr]) and source a stream of
//! [ReadResult].
//!
#![doc = badascii_formal!(r"
           ++ReadController++          
           |  sink          | araddr   
 ?AxilAddr |                +--------->
 +-------->| req.data       | arvalid  
           |                +--------->
<----------+ req.ready      | arready  
           |                |<--------+
           |  - - - - - -   |          
           |                | rdata    
           |                |<--------+
           |  source        | rresp    
?ReadResult|                |<--------+
<----------+ resp.data      | rvalid   
           |                |<--------+
+--------->| resp.ready     | rready   
           |                +--------->
           +----------------+          
")]
//!
//!# Internal Details
//!
//! Internally, the [ReadController] consists of the following rough
//! contents.  A [crate::stream::map::Map] core is used to convert between
//! the AXI [ReadResponse] to a [ReadResult].
#![doc = badascii!(r"
+-+From Core+-->                                                   
                     +-+Strm2AXI+---+ araddr                       
            AxilAddr |              +---------->                   
            +------->|              | arvalid                      
             ready   |    inbuf     +---------->                   
            <--------+              | arready                      
                     |              |<---------+                   
                     +--------------+                              
                                                   +--+To Core+--->
                                                                   
  rdata/rresp  +-+AXI2Strm+---+               +Map+---+            
 +------------>|              | ?ReadResponse |       |?ReadResult 
  rvalid       |              +-------------->|       +----------->
 +------------>|  outbuf      | ready         |       | resp_ready 
  rready       |              |<--------------+       |<----------+
 <-------------+              |               |       |            
               +--------------+               +-------+            
")]
//!
//! The [Axi2Rhdl] and [Rhdl2Axi] cores buffer their
//! inputs (outputs) so as to be spec compliant (i.e., no
//! combinatorial logic on the bus is allowed in AXI).
//!
//!# Example
//!
//! An example of using a [ReadController] and [ReadEndpoint]
//! together in a test harness is included here:
//!
#![doc = badascii!(r"

                      ++ReadController++                  ++ReadEndpoint++      +ReqSink+       
+ReqSource+           |  sink          | araddr    araddr |      source  |?Axil |       |       
|         | ?AxilAddr |                +--------->------->|              | Addr |       |       
|         +---------->| req.data       | arvalid  arvalid |    req.data  +----->|       |       
|         |           |                +--------->------->|              |      |       |       
|         |<----------+ req.ready      | arready  arready |    req.ready |<----+|       |       
|         |           |                |<--------+--------+              |      +-------+       
+---------+           |  - - - - - -   |                  |  - - - - - - |                      
                      |                | rdata    rdata   |              |                      
                      |                |<--------+--------+              |                      
                      |  source        | rresp    rresp   |      sink    |            +ReplySrc+
+ReplySink+?ReadResult|                |<--------+--------+              | ?ReadResult|        |
|         |<----------+ resp.data      | rvalid   rvalid  |   resp.data  |<-----------+        |
|         |           |                |<--------+--------+              |            |        |
|         +---------->| resp.ready     | rready   rready  |   resp.ready +----------->|        |
+---------+           |                +--------->------->+              |            +--------+
                      +----------------+                  +--------------+                      
")]
//!
//! Non-synthesizable functions are used to generate the request addresses and the
//! replies for demonstration purposes.
//!
//!```
#![doc = include_str!("../../../../examples/axi_read.rs")]
//!```
//!
//! with a trace file
//!
#![doc = include_str!("../../../../doc/axi_read.md")]

use badascii_doc::{badascii, badascii_formal};

use crate::{
    axi4lite::{
        stream::{axi_to_rhdl::Axi2Rhdl, rhdl_to_axi::Rhdl2Axi},
        types::{
            response_codes, AXI4Error, AxilAddr, ReadMISO, ReadMOSI, ReadOperation, ReadResponse,
            ReadResult,
        },
    },
    stream::map::Map,
};
use rhdl::prelude::*;

#[derive(Clone, Synchronous, SynchronousDQ)]
/// AXI Read Controller
///
/// This core sinks a RHDL stream of
/// addresses into AXI bus read transactions, and
/// converts the resulting stream of read responses into
/// a source stream of [ReadResult].
pub struct ReadController {
    inbuf: Rhdl2Axi<AxilAddr>,
    map: Map<ReadResponse, ReadResult>,
    outbuf: Axi2Rhdl<ReadResponse>,
}

impl Default for ReadController {
    fn default() -> Self {
        Self {
            inbuf: Rhdl2Axi::default(),
            map: Map::try_new::<map_result>().expect("ICE! Compilation of `map_result` failed!"),
            outbuf: Axi2Rhdl::default(),
        }
    }
}

#[kernel]
#[doc(hidden)]
pub fn map_result(_cr: ClockReset, resp: ReadResponse) -> ReadResult {
    match resp.resp {
        response_codes::OKAY => ReadResult::Ok(ReadOperation::Normal(resp.data)),
        response_codes::EXOKAY => ReadResult::Ok(ReadOperation::Exclusive(resp.data)),
        response_codes::DECERR => ReadResult::Err(AXI4Error::DECERR),
        response_codes::SLVERR => ReadResult::Err(AXI4Error::SLVERR),
        _ => ReadResult::Err(AXI4Error::DECERR),
    }
}

#[derive(PartialEq, Debug, Digital)]
/// Input for the [ReadController] core
pub struct In {
    /// AXI signals from bus
    pub axi: ReadMISO,
    /// Request data stream
    pub req_data: Option<AxilAddr>,
    /// Response ready signal
    pub resp_ready: bool,
}

#[derive(PartialEq, Debug, Digital)]
/// Output for the [ReadController] core
pub struct Out {
    /// AXI signals to the bus
    pub axi: ReadMOSI,
    /// Request stream ready signal
    pub req_ready: bool,
    /// Response data stream
    pub resp_data: Option<ReadResult>,
}

impl SynchronousIO for ReadController {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    // Wire up the input buffer inputs
    d.inbuf.data = i.req_data;
    d.inbuf.tready = i.axi.arready;
    // Wire up the output buffer inputs
    d.outbuf.tdata = ReadResponse {
        resp: i.axi.rresp,
        data: i.axi.rdata,
    };
    d.outbuf.tvalid = i.axi.rvalid;
    d.outbuf.ready = q.map.ready;
    // Wire up the map inputs
    d.map.data = q.outbuf.data;
    d.map.ready = i.resp_ready;
    // Wire up the axi outputs
    let mut o = Out::dont_care();
    o.req_ready = q.inbuf.ready;
    o.resp_data = q.map.data;
    o.axi.araddr = q.inbuf.tdata;
    o.axi.arvalid = q.inbuf.tvalid;
    o.axi.rready = q.outbuf.tready;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut = ReadController::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_compile() -> miette::Result<()> {
        compile_design::<map_result>(CompilationMode::Synchronous)?;
        Ok(())
    }
}
