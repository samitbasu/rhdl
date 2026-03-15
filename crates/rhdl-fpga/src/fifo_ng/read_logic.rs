//! FIFO Read Core
//!
//! The read side of a FIFO.  In this design (which is meant to be maximally
//! simple, but also as robus as possible), the read side of the FIFO stores
//! an internal read address and a staging buffer for the output data.  The
//! write address (owned by the write side of the FIFO) is provided to the
//! read logic as an input).
//!
//! Critical assumption:
//! We assume that the write address received from the write side of the FIFO
//! is conservative, meaning that the real write location is at least as great
//! as the write address provided - i.e., that the writer has definitely put
//! data into the given address when we get it.  It may have written additional
//! data that we don't know about, but we can be sure that the data at the
//! write address is valid.
//!
//! Note that this design will waste a slot in the FIFO when the read and write
//! addresses are equal, as it cannot otherwise distinguish between a full and
//! empty FIFO.
//!
//! Here is the schematic symbol.
//!
#![doc = badascii_doc::badascii_formal!("
                  +----+FIFOReadCore+-------------------+      
               bN |                                     |      
       To    +--->| write_address                       | ?T   
FIFOWriteCore  b1 |                                data +----> 
             <----+ did_read                            | b1    
       To      bN |                               ready |<----+
       BRAM  <----+ ram_read_address                    |      
               T  |                                     |      
       From  +--->| data                                |      
       BRAM       +-------------------------------------+      
")]
//! The [FIFOReadCore] is more of an internal component, that you are
//! unlikely to use directly.  Instead, use a [SyncFIFO] or [AsyncFIFO]
//! instead.
use crate::{
    core::{dff, option::is_some},
    stream::Ready,
};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// The FIFO read logic as a core
pub struct FIFOReadCore<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    ram_read_address: dff::DFF<Bits<N>>,
    did_read: dff::DFF<bool>,
    output_buffer: dff::DFF<Option<T>>,
}

impl<T, const N: usize> Default for FIFOReadCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            ram_read_address: dff::DFF::new(bits(0)),
            did_read: dff::DFF::new(false),
            output_buffer: dff::DFF::new(None),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The inputs to the [FIFOReadCore]
pub struct In<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    /// The current write address
    pub write_address: Bits<N>,
    /// The data back from the RAM
    pub data: T,
    /// The ready signal for downstream logic.
    pub ready: Ready<T>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The outputs from the [FIFOReadCore]
pub struct Out<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    /// The read address to the RAM
    pub ram_read_address: Bits<N>,
    /// A pulse that indicates we have consumed an item
    pub did_read: bool,
    /// The data output from the internal buffer.
    pub data_out: Option<T>,
}

impl<T, const N: usize> SynchronousIO for FIFOReadCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, N>;
    type O = Out<T, N>;
    type Kernel = read_logic<T, N>;
}

#[kernel]
pub fn read_logic<T, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<T, N>,
) -> (Out<T, N>, D<T, N>)
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<T, N>::dont_care();
    // The behavior of the read logic depends on 3 booleans.
    //  - empty is true if the read and write pointers are
    //    equal, indicating the FIFO is empty.
    //  - downstream_ready is true if the downstream logic is ready to accept data.
    //  - output_is_some is true if the output buffer contains valid data.
    let empty = i.write_address == q.ram_read_address;
    let downstream_ready = i.ready.raw;
    let output_is_some = is_some::<T>(q.output_buffer);
    let will_consume = downstream_ready && output_is_some;
    // Latch prevention
    d.output_buffer = q.output_buffer;
    let will_advance = !empty && (downstream_ready || !output_is_some);
    let next_address = if will_advance {
        q.ram_read_address + 1
    } else {
        q.ram_read_address
    };
    if will_advance {
        d.output_buffer = Some(i.data);
    } else if will_consume {
        d.output_buffer = None;
    }
    d.ram_read_address = next_address;
    d.did_read = will_advance;
    let o = Out::<T, N> {
        ram_read_address: next_address,
        did_read: q.did_read,
        data_out: q.output_buffer,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_read_logic() -> miette::Result<()> {
        let uut = FIFOReadCore::<b8, 4>::default();
        let desc = uut.descriptor(ScopedName::top())?;
        let hdl = desc.hdl()?;
        let hdl = hdl.modules.pretty();
        expect_test::expect_file!["testing/read_logic.v"].assert_eq(&hdl);
        Ok(())
    }
}
