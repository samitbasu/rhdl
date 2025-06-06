//! Cores used to implement AXI4Lite controllers
//!
//! These cores are useful for building controllers
//! that support the AXI4Lite bus protocol.  They provide
//! RHDL stream interfaces that are ergonomic to use and
//! bridge to AXI4Lite signals.  THey can be used to build
//! AXI bus controllers (as opposed to endpoints).  
//!
#![doc = badascii!(r"
                  +-----------+               
                  |           |   To Bus      
+--+ write +----->|           |               
     requests     | Write2AXi |               
                  |           +----+ AXI +--->
<--+ result  +----+           |               
     codes        |           |               
                  +-----------+               
")]
//!
//! In this design, the controller generates a stream of
//! write requests, and for each request, receives a corresponding
//! result code from the endpoint.  This core simply handles
//! the buffering and the minimal protocol translation to
//! convert the signals into AXI compliant signals.
//!
//! There is an equivalent core for reading:
//!
#![doc = badascii!(r"
                  +-----------+               
                  |           |   To Bus      
+--+ address +--->|           |               
     requests     | Read2Axi  |               
                  |           +----+ AXI +--->
<--+ read +-------+           |               
     data         |           |               
                  +-----------+               
")]
//!
//! Which takes a stream of address requests for read operations
//! and returns a stream of read data/results as a seperate stream.
//! Again, this core does very little mostly buffering of the signals
//! at the AXI interface to be spec compliant, and recoding of the
//! AXI spec error signals into more ergonomic Rust enums.
//!
//! The AXI spec allows a controller to block between transfers.  For
//! that use case, the following core can be used.
//!
#![doc = badascii!(r"
                  +-----------+               
                  |           |   To Bus      
+-+ read/write +->|           |               
     requests     |BlockRW2Axi|               
                  |           +----+ AXI +--->
<--+ response +---+           |               
     data         |           |               
                  +-----------+               
")]
//! In this case, the request is wrapped in an enum that allows either
//! read or write requests to be dispatched (but not both simultaneously),
//! and the responses are collected in order.  To avoid races, the core will
//! block on each request, meaning that the read or write request must complete
//! before the next one is dispatched.  Needless to say, this is not a
//! high performance way to use the bus, but it's useful for testing and
//! simpler use cases.
//!
use badascii_doc::badascii;
pub mod blocking;
pub mod read;
pub mod write;
