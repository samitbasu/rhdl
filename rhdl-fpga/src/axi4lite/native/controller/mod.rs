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
use badascii_doc::badascii;
pub mod read;
pub mod write;
