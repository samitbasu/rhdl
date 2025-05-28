//! Cores used to implement AXI4Lite endpoints
//!
//! These cores are used to implement an endpoint for an
//! AXI4L bus.  They are helpful for building cores that
//! have AXI interfaces on them.  For example, to build
//! a core that supports read operations via AXI:
//!
#![doc = badascii!(r"
                                  To Endpoint  
                  +----------+                 
From Bus          |          +----> address +->
                  | Axi2Read |      requests   
+----+ AXI +----->|          |                 
                  |          |<---+ read +----+
                  +----------+      data       
")]
//!
//! Or to build a core that supports write operations
//! via AXI.
//!
#![doc = badascii!(r"
                                  To Endpoint  
                 +-----------+                 
From Bus         |           +----> write   +->
                 | Axi2Write |      requests   
+----+ AXI +---->|           |                 
                 |           |<---+ result +--+
                 +-----------+      codes      
")]
//!
//! There are complementary cores used for building
//! controllers available in a seperate module.

use badascii_doc::badascii;

pub mod read;
pub mod write;
