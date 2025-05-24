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
           |                +--------->  | Address 
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
           |                |<--------+  | Response
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
                                      +-+Strm2AXI+---+ awaddr       
+-+From Core+-->                      |              +---------->   
                       +------------->|              | awvalid      
                       |              |  addr_buf    +---------->   
                       | +------------+              | awready      
            ++Tee+--+  | |            |              |<---------+   
   ?WriteCmd|   addr+--+ |            +--------------+              
   +------->|in     | A  |                                          
    ready   |    rdy|<---+            +-+Strm2AXI+---+ wdata/wstrobe
   <--------+       |                 |              +---------->   
            |   data+---------------->|              | wvalid       
            |       | B               |  data_buf    +---------->   
            |    rdy|<----------------+              | wready       
            +-------+                 |              |<---------+   
                                      +--------------+              
                                                                    
                                                    +--+To Core+--->
                                                                    
   bresp        +-+AXI2Strm+---+               +Map+---+            
  +------------>|              | ?ReadResponse |       |?ReadResult 
   bvalid       |              +-------------->|       +----------->
  +------------>|  outbuf      | ready         |       | resp_ready 
   bready       |              |<--------------+       |<----------+
  <-------------+              |               |       |            
                +--------------+               +-------+            
")]
//!
//! The [Axi2Rhdl] and [Rhdl2Axi] cores buffer their
//! inputs (outputs) so as to be spec compliant (i.e., no
//! combinatorial logic on the bus is allowed in AXI).
//!

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
    stream::{map::Map, tee::Tee},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
/// AXI Write Controller
///
/// This core sinks a RHDL stream of [WriteCommand]
/// structs, and converts them into AXI write transactions
/// and then converts the resulting responses into a stream
/// of [WriteResult].
pub struct WriteController {
    inbuv: Rhdl2Axi<WriteCommand>,
    tee: Tee<AxilAddr, StrobedData>,
    map: Map<ResponseKind, WriteResult>,
    outbuf: Axi2Rhdl<ResponseKind>,
}

#[kernel]
#[doc(hidden)]
pub fn map_result(_cr: ClockReset, resp: ResponseKind) -> WriteResult {
    match resp {
        response_codes::OKAY => Ok(ExFlag::Normal),
        response_codes::EXOKAY => Ok(ExFlag::Exclusive),
        response_codes::SLVERR => Err(AXI4Error::SLVERR),
        _ => Err(AXI4Error::DECERR),
    }
}

#[derive(PartialEq, Debug, Digital)]
/// Input for the [WriteController] core
pub struct In {
    /// AXI signals from the bus
    pub axi: WriteMISO,
    /// Request data stream
    pub req_data: Option<WriteCommand>,
    /// Response ready signal
    pub resp_ready: bool,
}

#[derive(PartialEq, Debug, Digital)]
/// Output fro the [WriteController] core
pub struct Out {
    /// AXI signals to the bus
    pub axi: WriteMOSI,
    /// Request stream ready signal
    pub req_ready: bool,
    /// Response data stream
    pub resp_data: Option<WriteResult>,
}

impl SynchronousIO for WriteController {
    type I = In;
    type O = Out;
    type Kernel = NoKernel3<ClockReset, In, Q, (Out, D)>;
}
