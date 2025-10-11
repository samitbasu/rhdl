//! A Register bank on the AXI Bus
//!
//! This is a simple dual ported bank fo registers
//! with an AXI4Lite bus interface.  The bank decodes
//! the addresses and supports concurrent read and
//! write operations from both the AXI4Lite bus interface
//! and from the core itself.
//!
//!# Schematic Symbol
//!
//! Here is the symbol for the core.  For
//! simplicity, we assume that 8 bits are enough
//! to select one of the registers in the bank).
//!
#![doc = badascii_formal!(r"
      ++AXI4RegBnk++      
      |            | [b32; N]     
      |      read  +--------->
+---->|            |      
 axi  |            | ?(b8,b32)
<----+|     write  |<--------+
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
  axi  |         |<------+     |   |[b32;N]|   |     +------>|        |  axi  
<------+         |?WrResp|     |   |       |   |     |?RdResp|        |<-----+
       |         |<------+     |<--+       |<--+     +------>|        |       
       |         | ready |     |   |       |   |     | ready |        |       
       |         +------>|     |   | o   i |   |     |<------+        |       
       +---------+       +-----+   +-+-----+   +-----+       +--------+       
                                     |   ^                                    
                                     v   |                                    
                                 [b32;N] +?(ndx, data)                                   
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
/// This core provides a bank of dual-ported registers
/// that has an AXI4Lite bus interface.  The base address
/// of the registers are provided at construction time.
pub struct AxiRegBank<const N: usize> {
    read: ReadEndpoint,
    write: WriteEndpoint,
    data: [DFF<AxilData>; N],
    address_low: Constant<AxilAddr>,
    address_high: Constant<AxilAddr>,
}

impl<const N: usize> AxiRegBank<N> {
    /// Create a register with the provided
    /// default value and the given register
    /// Where `address` is the address of the register
    /// and `reset_val` is the reset value of the
    /// register.
    pub fn new(address: AxilAddr, reset_val: [AxilData; N]) -> Self {
        assert!(N > 0 && N <= 256);
        Self {
            read: ReadEndpoint::default(),
            write: WriteEndpoint::default(),
            data: core::array::from_fn(|i| DFF::new(reset_val[i])),
            address_low: Constant::new(address),
            address_high: Constant::new(address + bits(N as u128 * 4)),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Input for the [AxiRegBank]
pub struct In {
    /// AXI signals from the bus for reading
    pub read_axi: ReadMOSI,
    /// AXI signals from the bus for writing
    pub write_axi: WriteMOSI,
    /// Write data from the client side
    pub data: Option<(b8, AxilData)>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Output for the [AxiRegBank]
pub struct Out<const N: usize> {
    /// AXI signals to the bus for reading
    pub read_axi: ReadMISO,
    /// AXI signals to the bus for writing
    pub write_axi: WriteMISO,
    /// Read data from the client side
    pub data: [AxilData; N],
}

impl<const N: usize> SynchronousIO for AxiRegBank<N> {
    type I = In;
    type O = Out<N>;
    type Kernel = kernel<N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<const N: usize>(_cr: ClockReset, i: In, q: Q<N>) -> (Out<N>, D<N>) {
    let mut d = D::<N>::dont_care();
    let mut o = Out::<N>::dont_care();
    d.write.axi = i.write_axi;
    o.write_axi = q.write.axi;
    d.read.axi = i.read_axi;
    o.read_axi = q.read.axi;
    d.write.req_ready.raw = q.write.resp_ready.raw;
    d.write.resp_data = None;
    d.data = q.data;
    if let Some(cmd) = q.write.req_data {
        if q.write.resp_ready.raw {
            if cmd.addr < q.address_low || cmd.addr > q.address_high {
                d.write.resp_data = Some(Err(AXI4Error::DECERR));
            } else {
                // Each register takes 4 bytes, so we need to right shfit by 2
                let reg_ndx = (cmd.addr - q.address_low) >> 2;
                let mask = strobe_to_mask(cmd.strobed_data.strobe);
                d.data[reg_ndx] = (q.data[reg_ndx] & (!mask)) | (cmd.strobed_data.data & mask);
                d.write.resp_data = Some(Ok(()));
            }
        }
    }
    d.read.req_ready.raw = q.read.resp_ready.raw;
    d.read.resp_data = None;
    if let Some(req) = q.read.req_data {
        if q.read.resp_ready.raw {
            if req < q.address_low || req > q.address_high {
                d.read.resp_data = Some(Err(AXI4Error::DECERR));
            } else {
                // Each register takes 4 bytes, so we need to right shift by 2
                let reg_ndx = (req - q.address_low) >> 2;
                d.read.resp_data = Some(Ok(q.data[reg_ndx]));
            }
        }
    }
    if let Some((ndx, write)) = i.data {
        d.data[ndx] = write;
    }
    o.data = q.data;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut: AxiRegBank<4> = AxiRegBank::new(bits(0x4_000_000), Default::default());
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_compile_times() -> miette::Result<()> {
        let tic = std::time::Instant::now();
        let uut: AxiRegBank<4> = AxiRegBank::new(bits(0x4_000_000), Default::default());
        let _hdl = uut.descriptor("top")?;
        let toc = tic.elapsed();
        println!("HDL generation took {toc:?}");
        Ok(())
    }
}
