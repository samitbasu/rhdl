//! A Single register on the AXI Bus
//!
//! A simple dual ported register with an AXI4Lite bus interface.
//! The address of the register is provided when the core is
//! instantiated.
//!
//!# Schematic Symbol
//!
//! Here is the symbol for the core.
//!
#![doc = badascii_formal!(r"
      +--+AXI4Reg+-+       
      |            | b32     
      |      read  +------>
+---->|            |       
 axi  |            | ?b32    
<----+|     write  |<-----+
      |            |       
      +------------+      
")]
//!
//!# Internal Details
//!
//! Internally, the register contains both a [ReadEndpoint] and
//! [WriteEndpoint] for communication with the AXI bus.  The
//! address is checked against a provided value.  Write contention
//! (i.e., writes to the register from both the client side
//! and the AXI side on the same clock cycle) are resolved in
//! factor of the client.
#![doc = badascii!(r"
                          Write                 Read                          
       ++WriteEp++       ++Dec++   ++Reg+--+   ++Dec++ ?Axil ++ReadEp++       
       |         |?WrCmd |     |   |       |   |     |  Addr |        |       
       |         +------>|     |   |       |   |     |<------+        |       
+----->|         | ready |     +-->|       +-->|     | ready |        +------>
  axi  |         |<------+     |   |  b32  |   |     +------>|        |  axi  
<------+         |?WrResp|     |   |       |   |     |?RdResp|        |<-----+
       |         |<------+     |<--+       |<--+     +------>|        |       
       |         | ready |     |   |       |   |     | ready |        |       
       |         +------>|     |   | o   i |   |     |<------+        |       
       +---------+       +-----+   +-+-----+   +-----+       +--------+       
                                     |   ^                                    
                                     v   |                                    
                                         +                                    
")]
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{
    axi4lite::{
        core::endpoint::{read::ReadEndpoint, write::WriteEndpoint},
        types::{
            strobe_to_mask, AXI4Error, AxilAddr, AxilData, ReadMISO, ReadMOSI, WriteMISO, WriteMOSI,
        },
    },
    core::{constant::Constant, dff::DFF},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
/// AXI Register
///
/// This core provides a single dual-ported register
/// that has an AXI4Lite bus interface.  The address
/// of the register is provided at construction time.
pub struct AxiRegister {
    read: ReadEndpoint,
    write: WriteEndpoint,
    data: DFF<AxilData>,
    address: Constant<AxilAddr>,
}

impl AxiRegister {
    /// Create a register with the provided
    /// default value and the given register
    /// Where `address` is the address of the register
    /// and `reset_val` is the reset value of the
    /// register.
    pub fn new(address: AxilAddr, reset_val: AxilData) -> Self {
        Self {
            read: ReadEndpoint::default(),
            write: WriteEndpoint::default(),
            data: DFF::new(reset_val),
            address: Constant::new(address),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
/// Input for the [AxiRegister]
pub struct In {
    /// AXI signals from the bus for reading
    pub read_axi: ReadMOSI,
    /// AXI signals from the bus for writing
    pub write_axi: WriteMOSI,
    /// Write data from the client side
    pub data: Option<AxilData>,
}

#[derive(PartialEq, Debug, Digital)]
/// Output for the [AxiRegister]
pub struct Out {
    /// AXI signals to the bus for reading
    pub read_axi: ReadMISO,
    /// AXI signals to the bus for writing
    pub write_axi: WriteMISO,
    /// Read data from the client side
    pub data: AxilData,
}

impl SynchronousIO for AxiRegister {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    let mut o = Out::dont_care();
    d.write.axi = i.write_axi;
    o.write_axi = q.write.axi;
    d.read.axi = i.read_axi;
    o.read_axi = q.read.axi;
    d.write.req_ready.raw = q.write.resp_ready.raw;
    d.write.resp_data = None;
    d.data = q.data;
    if let Some(cmd) = q.write.req_data {
        if q.write.resp_ready.raw {
            if cmd.addr == q.address {
                let mask = strobe_to_mask(cmd.strobed_data.strobe);
                d.data = (q.data & (!mask)) | (cmd.strobed_data.data & mask);
                d.write.resp_data = Some(Ok(()));
            } else {
                d.write.resp_data = Some(Err(AXI4Error::DECERR));
            }
        }
    }
    d.read.req_ready.raw = q.read.resp_ready.raw;
    d.read.resp_data = None;
    if let Some(req) = q.read.req_data {
        if q.read.resp_ready.raw {
            if req == q.address {
                d.read.resp_data = Some(Ok(q.data));
            } else {
                d.read.resp_data = Some(Err(AXI4Error::DECERR));
            }
        }
    }
    if let Some(write) = i.data {
        d.data = write;
    }
    o.data = q.data;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut = AxiRegister::new(bits(0), bits(0));
        AxiRegister::new(bits(0), bits(0));
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn hdl_is_ok() -> miette::Result<()> {
        let uut = AxiRegister::new(bits(0), bits(0));
        let _ = uut.hdl("top")?;
        Ok(())
    }
}
