//! AXI4Lite Write Controller Interface
//!
//!# Purpose
//!
//! This core provides a way to build an AXI write controller using a
//! pair of RHDL streams.  One is a stream for addresses/data to write
//! to the AXI bus.  The other is a stream of responses from
//! the bus back to the controller.  Each request must generate
//! a response.  This core can be paired up with the [WriteEndpoint]
//! core to transport the RHDL stream across an AXI bus interface
//! in a spec compliant manner.
//!
//!# Schematic Symbol
//!
//! Here is the symbol for the core.  The core will sink a stream
//! of [WriteCommand] and source a stream of
//! [WriteResponse].
//!
#![doc = badascii_formal!(r"
           ++WriteController+                      
           |  sink          | awaddr               
  ?WriteCmd|                +--------->  +         
 +-------->| req.data       | awvalid    | Write   
R<WriteCmd>|                +--------->  | Address 
<----------+ req.ready      | awready    | Channel 
           |                |<--------+  +         
           |                | wdata                
           |                +--------->  +         
           |                | wstrobe    | Write   
           |                +--------->  | Data    
           |                | wvalid     | Channel 
           |                +--------->  |         
           |                | wready     |         
           |  - - - - - -   |<--------+  +         
           |  source        | bresp                
?WriteReslt|                |<--------+  +         
<----------+ resp.data      | bvalid     | Write   
R<WrtRslt> |                |<--------+  | Response
+--------->| resp.ready     | bready     | Channel 
           |                +--------->  +         
           +----------------+                      
")]
//!
//! The core looks complicated, but it's mostly just interconnect to the AXI bus, which
//! has a lot of control and handshake signals.  Using the core is pretty simple.  Provide
//! a stream of [WriteCommand] structs, and receive a stream of [WriteResult] codes.
//!
//!# Internal Details
//!
//! Internally, the [WriteController] consists of the following rough
//! contents.  A [crate::stream::map::Map] core is used to convert between
//! the AXI [ResponseKind] to a [WriteResponse].  A [crate::stream::tee::Tee] core
//! is used to feed the address and data channels from the single input stream.
#![doc = badascii!(r"
                                      +-+Strm2AXI+---+ awaddr  7     
+-+From Core+-->              2       |              +---------->   
                       +------------->|              | awvalid 8     
                       |      3       |  addr_buf    +---------->   
                       | +------------+              | awready 9     
            ++Tee+--+  | |            |              |<---------+   
 1 ?WriteCmd|   addr+--+ |            +--------------+              
   +------->|in     | A  |                                          
 6  ready   |    rdy|<---+            +-+Strm2AXI+---+ wdata/wstrobe 10/11
   <--------+       |         4       |              +---------->   
            |   data+---------------->|              | wvalid   12    
            |       | B       5       |  data_buf    +---------->   
            |    rdy|<----------------+              | wready   13    
            +-------+                 |              |<---------+   
                                      +--------------+              
                                                                    
                                                    +--+To Core+--->
                                                                    
   bresp   14   +-+AXI2Strm+---+     17        +Map+---+    19        
  +------------>|              | ?ReadResponse |       |?ReadResult 
   bvalid  15   |              +-------------->|       +----------->
  +------------>|  outbuf      | ready  18     |       | resp_ready 
   bready  16   |              |<--------------+       |<----------+
  <-------------+              |               |       |    20        
                +--------------+               +-------+            
")]
//!
//! The [Axi2Rhdl] and [Rhdl2Axi] cores buffer their
//! inputs (outputs) so as to be spec compliant (i.e., no
//! combinatorial logic on the bus is allowed in AXI).
//!
//!# Example
//!
//! An example of using a [WriteController] and [WriteEndpoint]
//! together in a test harness is included here:
//!
#![doc = badascii!(r"
+CmdSrc+-+         ++WriteController+   5/6    +-+WriteEndpoint+-+    7    +CmdSink++
|        |     1   |  sink          | awaddr   |   source        |?WriteCmd|        |
|        |?WriteCmd|                +--------->|       req.data  +-------->|        |
|        +-------->| req.data       | awvalid  |                 |    8    |        |
|        |     2   |                +--------->|       req.ready |<-------+|        |
|        |<--------+ req.ready      | awready  |                 |         |        |
|        |         |                |<---------+                 |         |        |
+--------+         |                | wdata    |                 |         +--------+
                   |                +--------->| axi             |                   
                   |                | wstrobe  |                 |                   
                   |                +--------->|                 |                   
                   |                | wvalid   |                 |                   
                   |                +--------->|                 |                   
                   |                | wready   |                 |                   
                   |  - - - - - -   |<---------+  - - - - - -    |                   
+------+       3   |  source        | bresp    |  sink           |     9     +------+
|      |?WriteReslt|                |<---------+                 |?WriteReslt|      |
|Write |<----------+ resp.data      | bvalid   |       resp.data |<----------+Write |
|Result|       4   |                |<---------+                 |     10    |Result|
|Sink  +---------->| resp.ready     | bready   |      resp.ready +---------->|Source|
|      |           |                +--------->|                 |           |      |
+------+           +----------------+          +-----------------+           +------+
")]
//!
//! Non-synthesizable functions are used to generate a stream of random
//! [WriteCommand]s and [WriteResult]s.
//!
//!```
#![doc = include_str!("../../../../examples/axi_write.rs")]
//!```
//!
//! with a trace file
//!
#![doc = include_str!("../../../../doc/axi_write.md")]

use badascii_doc::{badascii, badascii_formal};

use rhdl::prelude::*;

use crate::{
    axi4lite::{
        stream::{axi_to_rhdl::Axi2Rhdl, rhdl_to_axi::Rhdl2Axi},
        types::{
            response_codes, AXI4Error, AxilAddr, ResponseKind, StrobedData, WriteCommand,
            WriteMISO, WriteMOSI, WriteResult,
        },
    },
    stream::{map::Map, ready, tee::Tee, Ready},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// AXI Write Controller
///
/// This core sinks a RHDL stream of [WriteCommand]
/// structs, and converts them into AXI write transactions
/// and then converts the resulting responses into a stream
/// of [WriteResult].
pub struct WriteController {
    tee: Tee<AxilAddr, StrobedData>,
    addr_buf: Rhdl2Axi<AxilAddr>,
    data_buf: Rhdl2Axi<StrobedData>,
    map: Map<ResponseKind, WriteResult>,
    outbuf: Axi2Rhdl<ResponseKind>,
}

impl Default for WriteController {
    fn default() -> Self {
        Self {
            tee: Tee::default(),
            addr_buf: Rhdl2Axi::default(),
            data_buf: Rhdl2Axi::default(),
            map: Map::try_new::<map_result>().expect("ICE! Compilation of `map_result` failed!"),
            outbuf: Axi2Rhdl::default(),
        }
    }
}

#[kernel]
#[doc(hidden)]
pub fn map_result(_cr: ClockReset, resp: ResponseKind) -> WriteResult {
    match resp {
        response_codes::OKAY => Ok(()),
        response_codes::EXOKAY => Ok(()),
        response_codes::SLVERR => Err(AXI4Error::SLVERR),
        _ => Err(AXI4Error::DECERR),
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Input for the [WriteController] core
pub struct In {
    /// AXI signals from the bus
    pub axi: WriteMISO,
    /// Request data stream
    pub req_data: Option<WriteCommand>,
    /// Response ready signal
    pub resp_ready: Ready<WriteResult>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Output from the [WriteController] core
pub struct Out {
    /// AXI signals to the bus
    pub axi: WriteMOSI,
    /// Request stream ready signal
    pub req_ready: Ready<WriteCommand>,
    /// Response data stream
    pub resp_data: Option<WriteResult>,
}

impl SynchronousIO for WriteController {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    // Connection 1
    d.tee.data = None;
    if let Some(cmd) = i.req_data {
        d.tee.data = Some((cmd.addr, cmd.strobed_data));
    }
    // Connection 2
    d.addr_buf.data = q.tee.s_data;
    // Connection 3
    d.tee.s_ready = q.addr_buf.ready;
    // Connection 4
    d.data_buf.data = q.tee.t_data;
    // Connection 5
    d.tee.t_ready = q.data_buf.ready;
    let mut o = Out::dont_care();
    // Connection 6
    o.req_ready = ready::<WriteCommand>(q.tee.ready.raw);
    // Connection 7
    o.axi.awaddr = q.addr_buf.tdata;
    // Connection 8
    o.axi.awvalid = q.addr_buf.tvalid;
    // Connection 9
    d.addr_buf.tready = i.axi.awready;
    // Connection 10
    o.axi.wdata = q.data_buf.tdata.data;
    // Connection 11
    o.axi.wstrb = q.data_buf.tdata.strobe;
    // Connection 12
    o.axi.wvalid = q.data_buf.tvalid;
    // Connection 13
    d.data_buf.tready = i.axi.wready;
    // Connection 14
    d.outbuf.tdata = i.axi.bresp;
    // Connection 15
    d.outbuf.tvalid = i.axi.bvalid;
    // Connection 16
    o.axi.bready = q.outbuf.tready;
    // Connection 17
    d.map.data = q.outbuf.data;
    // Connection 18
    d.outbuf.ready = q.map.ready;
    // Connection 19
    o.resp_data = q.map.data;
    // Connection 20
    d.map.ready = i.resp_ready;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut = WriteController::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_compile() -> miette::Result<()> {
        compile_design::<map_result>(CompilationMode::Synchronous)?;
        Ok(())
    }
}
