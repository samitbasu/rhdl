//! FIFO Write Core
//!
//! The write side of the FIFO.  In this design (which is meant to be maximally
//! simple, but also as robust as possible), the write side of the FIFO stores
//! an internal write address and an overflow flag.  The read address (owned by
//! the read side of the FIFO) is provided to the write logic as an input.
//!
//! Critical assumption:
//!  
//! We assume that the read address received from the
//! read side of the FIFO is conservative, meaning that
//! the real read location is at least as great
//! as the read address provided - i.e., that the reader may have already read
//! out the given memory location, but that the writer can safely write into the
//! FIFO provided it does not reach the given address
//!
//! Note that this design will waste a slot in the FIFO when the read and write
//! addresses are equal, as it cannot otherwise distinguish between a full and
//! empty FIFO.  So for N bits, this design can store 2^N-1 elements.
//!
//! Here is the schematic symbol.
//!
#![doc = badascii_formal!("
     +----+FIFOWriteCore+-------------+                         
 ?T  |                                |  bN      From           
+--->| data               read_address|<------+  FIFOReadCore   
  b1 |                                |  bool                     
<----+ ready                 did_write+------->  To FIFOReadCore
     |                                |?(bN,T)   To             
     |                            data+------->  BRAM           
     +--------------------------------+                         
")]
//!
//! The write core keeps track of the write pointer of the FIFO.

use crate::core::dff;
use crate::stream::{Ready, ready};
use badascii_doc::badascii_formal;
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// The FIFO write logic as a core
pub struct FIFOWriteCore<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    // The current write address
    write_address: dff::DFF<Bits<N>>,
    // We delay the write address by one clock before sending
    // it to the read side of the FIFO.  This is because it will
    // take one clock for the write to actually happen, and we
    // want to make sure the value is valid on the read side before
    // "counting" the write.
    did_write: dff::DFF<bool>,
    // Marker for the type T
    phantom: std::marker::PhantomData<T>,
}

impl<T, const N: usize> Default for FIFOWriteCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            write_address: dff::DFF::new(bits(0)),
            did_write: dff::DFF::new(false),
            phantom: std::marker::PhantomData,
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The inputs to the [FIFOWriteCore]
pub struct In<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The input data
    pub data: Option<T>,
    /// The current read address
    pub read_address: Bits<N>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The outputs from the [FIFOWriteCore]
pub struct Out<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    /// The generated ready signal
    pub ready: Ready<T>,
    /// A pulse that indicates we have written an item
    pub did_write: bool,
    /// The data to write to the BRAM
    pub data: Option<(Bits<N>, T)>,
}

impl<T, const N: usize> SynchronousIO for FIFOWriteCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, N>;
    type O = Out<T, N>;
    type Kernel = write_logic<T, N>;
}

#[kernel]
/// Kernel for the [FIFOWriteCore]
pub fn write_logic<T, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<T, N>,
) -> (Out<T, N>, D<T, N>)
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    let mut o = Out::<T, N>::dont_care();
    // Compute the full flag
    let full = (q.write_address + 1) == i.read_address;
    let mut will_write = false;
    o.data = if let Some(data) = i.data {
        if !full {
            will_write = true;
            Some((q.write_address, data))
        } else {
            None
        }
    } else {
        None
    };
    o.ready = ready::<T>(!full);
    o.did_write = q.did_write;
    let mut d = D::<T, N>::dont_care();
    d.write_address = if will_write {
        q.write_address + 1
    } else {
        q.write_address
    };
    d.did_write = will_write;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_write_logic() -> miette::Result<()> {
        let uut = FIFOWriteCore::<b8, 4>::default();
        let desc = uut.descriptor(ScopedName::top())?;
        let hdl = desc.hdl()?;
        let hdl = hdl.modules.pretty();
        expect_test::expect_file!["testing/write_logic.v"].assert_eq(&hdl);
        Ok(())
    }
}
