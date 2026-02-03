//! A Read Only Register bank on the AXI bus
//!
//! This simple core provides a readable set of register
//! values to an AXI bus.  The read values are also
//! exported via a wire bus for use by other (non-AXI) cores.
//!
//!# Schematic Symbol
//!
//! Here is the symbol for the core.  For
//! simplicity, we assume that 8 bits are enough
//! to select one of the registers in the bank).
//!
#![doc = badascii_formal!(r"
      ++AXI4ROM+---+      
      |            | [b32; N]     
 axi  |            +--------->
<---->|            |      
      +------------+      
")]
//!
//!# Internal Details
//!
//! Internally, the register contains a [ReadEndpoint]
//! for communication with the AXI bus.  The
//! address is checked against a provided value.
#![doc = badascii!(r"
               Read                          
  ++ROM+--+   ++Dec++ ?Axil ++ReadEp++       
  |       |   |     |  Addr |        |       
  |       |   |     |<------+        |       
  |       +-->|     | ready |        +------>
  |[b32;N]|   |     +------>|        |  axi  
  |       |   |     |?RdResp|        |<-----+
  +       |<--+     +------>|        |       
  |       |   |     | ready |        |       
  | o     |   |     |<------+        |       
  +-+-----+   +-----+       +--------+       
    |                                        
    v                                        
[b32;N]                                      
")]

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{
    axi4lite::{
        core::endpoint::read::ReadEndpoint,
        types::{AXI4Error, AxilAddr, AxilData, ReadMISO, ReadMOSI},
    },
    core::constant::Constant,
};

#[derive(Clone, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// AXI ROM
///
/// This core provides a single ROM that has an
/// AXI4Lite bus interface.  The base (byte) address
/// of the ROM is provided at construction time.
pub struct AxiRom<const N: usize> {
    read: ReadEndpoint,
    data: [Constant<AxilData>; N],
    address_low: Constant<AxilAddr>,
    address_high: Constant<AxilAddr>,
}

impl<const N: usize> AxiRom<N> {
    /// Create a register with the provided
    /// default value and the given register
    /// Where `address` is the address of the register
    /// and `reset_val` is the reset value of the
    /// register.
    pub fn new(address: AxilAddr, reset_val: [AxilData; N]) -> Self {
        assert!(N > 0 && N <= 256);
        Self {
            read: ReadEndpoint::default(),
            data: core::array::from_fn(|i| Constant::new(reset_val[i])),
            address_low: Constant::new(address),
            address_high: Constant::new(address + bits(N as u128 * 4)),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Input for the [AxiRegBank]
pub struct In {
    /// AXI signals from the bus for reading
    pub axi: ReadMOSI,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Output for the [AxiRegBank]
pub struct Out<const N: usize> {
    /// AXI signals to the bus for reading
    pub read_axi: ReadMISO,
    /// Read data from the client side
    pub data: [AxilData; N],
}

impl<const N: usize> SynchronousIO for AxiRom<N> {
    type I = In;
    type O = Out<N>;
    type Kernel = kernel<N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<const N: usize>(_cr: ClockReset, i: In, q: Q<N>) -> (Out<N>, D<N>) {
    let mut d = D::<N>::dont_care();
    let mut o = Out::<N>::dont_care();
    d.read.axi = i.axi;
    o.read_axi = q.read.axi;
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
    o.data = q.data;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_combinatorial_paths() -> miette::Result<()> {
        let uut: AxiRom<4> = AxiRom::new(
            bits(0x4_000_000),
            [
                bits(0xDEADBEEF),
                bits(0xBABEFEED),
                bits(0xAAAA5555),
                bits(0x1234ABCD),
            ],
        );
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
