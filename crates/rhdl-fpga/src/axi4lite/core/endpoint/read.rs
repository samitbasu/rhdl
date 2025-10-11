//! AXI4Lite Read Endpoint Interface
//!
//!# Purpose
//!
//! This core provides a way to interface RHDL stream cores to the
//! AXI4Lite bus interface.  This interface core simply
//! repackages the signals to and from the AXI bus into RHDL streams.
//! The addresses are provided on a source stream, and the result of
//! the read (including error conditions) is fed to a sink stream.
//!
//!# Schematic Symbol
//!
//! Here is the symbol for the core. It provides a source stream
//! for the addresses (which are `b32`), and sinks a stream of
//! values (which are [ReadResult]).
//!
#![doc = badascii_formal!(r"
         ++ReadEndpoint++
  araddr |      source  |            
+------->|              | ?b32       
 arvalid |    req.data  +----->      
+------->|              | R<b32>
 arready |    req.ready |<----+            
<--------+              |            
         |  - - - - - - |            
 rdata   |              |            
<--------+              |
 rresp   |      sink    |            
<--------+              | ?ReadResult
 rvalid  |   resp.data  |<----------+
<--------+              | R<ReadResult>
 rready  |   resp.ready +----------->
+------->+              |            
         +--------------+            
")]
//!
//!# Internal Details
//!
//! Internally, the [Axi2ReadStreams] consists of the following rough
//! contents.  A [crate::stream::map::Map] core is used to map the
//! [ReadResult] to a [ReadResponse], which is then adapted out to the
//! AXI bus.
//!
#![doc = badascii!(r"
           +-+AXI2Stream+-+                                
    araddr |              | ?b32                           
  +------->|          data+----->                          
   arvalid |              | req_ready    +--+To Core+--->      
  +------->|         ready|<----+                          
   arready |              |                                
  <--------+              |                                
           +--------------+                                
                                                           
+-+From Core+-->                                           
                                                           
              +Map+---+              +-+Strm2AXI+--+ rdata/rresp 
 ?ReadResult  |       |?ReadResponse |             +------>
 +----------->|       +------------->|             | rvalid
  resp_ready  |       | ready        |             +------>
 <------------+       |<------------+|             | rready
              +-------+              |             |<-----+
                                     +-------------+       
")]
//!
//! The [Axi2Rhdl] and [Rhdl2Axi] cores buffer their
//! inputs (and outputs) so as to be spec compliant (i.e., no
//! combinatorial logic on the bus is allowed in AXI).
//!
//!
//!# Example
//!
//! An example of using a [ReadController] and [ReadEndpoint]
//! together in a test harness is included here:
//!
#![doc = badascii!(r"

                      ++ReadController++                  ++ReadEndpoint++      +ReqSink+       
+ReqSource+           |  sink          | araddr    araddr |      source  |      |       |       
|         |   ?b32    |                +--------->------->|              | ?b32 |       |       
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
            response_codes, AXI4Error, AxilAddr, AxilData, ReadMISO, ReadMOSI, ReadResponse,
            ReadResult,
        },
    },
    stream::{map::Map, Ready},
};
use rhdl::prelude::*;

#[derive(Clone, Synchronous, SynchronousDQ)]
/// AXI Read Endpoint
///
/// This core converts the AXI bus signals into a RHDL stream
/// source of read addresses, and a RHDL stream sink of
/// read results.  
pub struct ReadEndpoint {
    inbuf: Axi2Rhdl<AxilData>,
    map: Map<ReadResult, ReadResponse>,
    outbuf: Rhdl2Axi<ReadResponse>,
}

impl Default for ReadEndpoint {
    fn default() -> Self {
        Self {
            inbuf: Axi2Rhdl::default(),
            map: Map::try_new::<map_result>().expect("ICE! Compilation of map_result failed!"),
            outbuf: Rhdl2Axi::default(),
        }
    }
}

#[kernel]
#[doc(hidden)]
pub fn map_result(_cr: ClockReset, res: ReadResult) -> ReadResponse {
    match res {
        ReadResult::Ok(data) => ReadResponse {
            resp: response_codes::OKAY,
            data,
        },
        ReadResult::Err(err) => match err {
            AXI4Error::SLVERR => ReadResponse {
                resp: response_codes::SLVERR,
                data: bits(0),
            },
            AXI4Error::DECERR => ReadResponse {
                resp: response_codes::DECERR,
                data: bits(0),
            },
        },
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Input for the [AxiReadStreams] core
pub struct In {
    /// AXI signals for core
    pub axi: ReadMOSI,
    /// Request stream ready signal from core
    pub req_ready: Ready<AxilAddr>,
    /// Response data stream from core
    pub resp_data: Option<ReadResult>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Output from the [AxiReadStreams] core
pub struct Out {
    /// AXI signals for core
    pub axi: ReadMISO,
    /// Request data to the core
    pub req_data: Option<AxilAddr>,
    /// Response ready signal to core
    pub resp_ready: Ready<ReadResult>,
}

impl SynchronousIO for ReadEndpoint {
    type I = In;
    type O = Out;
    type Kernel = kernel; //NoKernel3<ClockReset, In, Q, (Out, D)>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    // Input buffer inputs
    d.inbuf.tdata = i.axi.araddr;
    d.inbuf.tvalid = i.axi.arvalid;
    d.inbuf.ready = i.req_ready;
    // Map core inputs
    d.map.data = i.resp_data;
    d.map.ready = q.outbuf.ready;
    // Output buffer inputs
    d.outbuf.tready = i.axi.rready;
    d.outbuf.data = q.map.data;
    // Core outputs
    let mut o = Out::dont_care();
    o.axi.arready = q.inbuf.tready;
    o.axi.rdata = q.outbuf.tdata.data;
    o.axi.rresp = q.outbuf.tdata.resp;
    o.axi.rvalid = q.outbuf.tvalid;
    o.req_data = q.inbuf.data;
    o.resp_ready = q.map.ready;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut = ReadEndpoint::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
