//! AXI4Lite Write Endpoint Interface
//!
//!# Purpose
//!
//! This core provides a bridge from an AXI write bus
//! (address, data/strobe and response) to a pair of
//! RHDL streams that can be connected to cores that
//! implement the write function.  This core takes on
//! the burden of merging the address and data requests
//! (which can arrive independently on different channels)
//! into a single stream of [WriteCommand] that can
//! be executed by the core.  Results of the write operation
//! are then fed into the `resp` sink.  
//!
//!# Schematic Symbol
//!
//! Here is the symbol for the core.  It provides a source
//! stream of [WriteCommand] structs, and sinks a stream of
//! [WriteResult] structs.
#![doc = badascii_formal!(r"
                       +-+WriteEndpoint+-+           
              awaddr   |   source        |?WriteCmd  
+            +-------->|       req.data  +-------->  
| Write       awvalid  |                 |           
| Address    +-------->|       req.ready |<-------+  
| Channel     awready  |                 |           
+            <---------+                 |           
              wdata    |                 |           
+            +-------->| axi             |           
| Write       wstrobe  |                 |           
| Data       +-------->|                 |           
| Channel     wvalid   |                 |           
|            +-------->|                 |           
|             wready   |                 |           
+            <---------+  - - - - - -    |           
              bresp    |  sink           |           
+            <---------+                 |?WriteReslt
| Write       bvalid   |       resp.data |<-------+  
| Response   <---------+                 |           
| Channel     bready   |      resp.ready +-------->  
+            +-------->|                 |           
                       +-----------------+           
")]
//!
//!# Internal Details
//!
//! Internally, the [WriteEndpoint] consists of the following
//! rough contents.  The AXI streams for the address and
//! data channels are converted into a pair of RHDL streams
//! using [Axi2Rhdl] cores.  A [crate::stream::zip::Zip] core is used
//! to `zip` the two streams into a single stream, which is then
//! used to feed the source stream of [WriteCommand].  On the
//! response side, the incoming stream of [WriteResult] structs
//! are [crate::stream::map::Map]-ped into a stream of
//! [ResponseKind] (2-bit response codes from the AXI specification).
//! These are then mapped out to the AXI response bus.
//!
#![doc = badascii!(r"
               +-+AXI2Strm+-+                   +-+To Core+-->     
     awaddr  1 |            | ?AxilAddr 7                          
    +--------->|      tdata +-------------+                        
     awvalid 2 |            |           8 |                        
    +--------->|      ready |<----------+ |  ++Zip+--+             
     awready 3 |            |           | |  |       |             
    <----------+            |           | +->|a_data | ?WriteCmd 11
               +------------+           |    |       +---------->  
               +-+AXI2Strm+-+           +----+a_rdy  |             
 4  wdata/wstrb|            | ?StrobeData 9  |       | ready  12   
    +--------->|       data +--------------->|b_data |<---------+  
 5    wvalid   |            |            10  |       |             
    +--------->|      ready |<---------------+b_rdy  |             
 6    wready   |            |                |       |             
    <----------+            |                +-------+             
               +------------+                                      
                                                                   
   +-+From Core+-->                                                
                                                                   
               +Map+---+     15       +-+Strm2Axi+---+   17        
13 ?WriteResult|       |?WriteResponse|        bresp +---------->  
    +--------->|       +------------->|              |   18        
14   resp_ready|       | ready  16    |        bvalid+---------->  
    <----------+       |<-------------+              |   19        
               +-- ----+              |        bready+<---------+  
                                      +--------------+             
")]
//!
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
            response_codes, AXI4Error, AxilAddr, ExFlag, ResponseKind, StrobedData, WriteCommand,
            WriteMISO, WriteMOSI, WriteResult,
        },
    },
    stream::{map::Map, zip::Zip},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
/// AXI Write Endpoint
///
/// This core translates an AXI write bus into
/// a source RHDL stream of [WriteCommand] structs,
/// and then sinks a stream of [WriteResult] structs
/// back to the AXI bus.
pub struct WriteEndpoint {
    addr_buf: Axi2Rhdl<AxilAddr>,
    data_buf: Axi2Rhdl<StrobedData>,
    zip: Zip<AxilAddr, StrobedData>,
    map: Map<WriteResult, ResponseKind>,
    resp_buf: Rhdl2Axi<ResponseKind>,
}

impl Default for WriteEndpoint {
    fn default() -> Self {
        Self {
            addr_buf: Axi2Rhdl::default(),
            data_buf: Axi2Rhdl::default(),
            zip: Zip::default(),
            map: Map::try_new::<map_result>().expect("ICE! Compilation of `map_result` failed"),
            resp_buf: Rhdl2Axi::default(),
        }
    }
}

#[kernel]
#[doc(hidden)]
pub fn map_result(_cr: ClockReset, resp: WriteResult) -> ResponseKind {
    match resp {
        Ok(flag) => match flag {
            ExFlag::Normal => response_codes::OKAY,
            ExFlag::Exclusive => response_codes::EXOKAY,
        },
        Err(err) => match err {
            AXI4Error::DECERR => response_codes::DECERR,
            AXI4Error::SLVERR => response_codes::SLVERR,
        },
    }
}

#[derive(PartialEq, Debug, Digital)]
/// Input for the [WriteEndpoint] core
pub struct In {
    /// AXI signals from the bus
    pub axi: WriteMOSI,
    /// request stream ready signal
    pub req_ready: bool,
    /// Response data stream
    pub resp_data: Option<WriteResult>,
}

#[derive(PartialEq, Debug, Digital)]
/// Output from the [WriteEndpoint] core
pub struct Out {
    /// AXI signals to the bus
    pub axi: WriteMISO,
    /// Request data stream
    pub req_data: Option<WriteCommand>,
    /// Response ready signal
    pub resp_ready: bool,
}

impl SynchronousIO for WriteEndpoint {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    let mut o = Out::dont_care();
    // Connection 1.
    d.addr_buf.tdata = i.axi.awaddr;
    // Connection 2.
    d.addr_buf.tvalid = i.axi.awvalid;
    // Connection 3
    o.axi.awready = q.addr_buf.tready;
    // Connection 4
    d.data_buf.tdata.data = i.axi.wdata;
    d.data_buf.tdata.strobe = i.axi.wstrb;
    // Connection 5
    d.data_buf.tvalid = i.axi.wvalid;
    // Connection 6
    o.axi.wready = q.data_buf.tready;
    // Connection 7
    d.zip.a_data = q.addr_buf.data;
    // Connection 8
    d.addr_buf.ready = q.zip.a_ready;
    // Connection 9
    d.zip.b_data = q.data_buf.data;
    // Connection 10
    d.data_buf.ready = q.zip.b_ready;
    // Connection 11
    o.req_data = None;
    if let Some((addr, strobed_data)) = q.zip.data {
        o.req_data = Some(WriteCommand { addr, strobed_data });
    }
    // Connection 12
    d.zip.ready = i.req_ready;
    // Connection 13
    d.map.data = i.resp_data;
    // Connection 14
    o.resp_ready = q.map.ready;
    // Connection 15
    d.resp_buf.data = q.map.data;
    // Connection 16
    d.map.ready = q.resp_buf.ready;
    // Connection 17
    o.axi.bresp = q.resp_buf.tdata;
    // Connection 18
    o.axi.bvalid = q.resp_buf.tvalid;
    // Connection 19
    d.resp_buf.tready = i.axi.bready;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut = WriteEndpoint::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
