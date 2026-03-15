//! A simple synchronous FIFO.
//!
//! This FIFO is designed to be as simple as possible
//! and thus be robust.  It is a two-port FIFO, with separate read and write
//! ports.  The FIFO is parameterized by the number of bits in each element.
//! The depth of the FIFO is 2^N elements.  
//!
//! Here is the schematic symbol for the FIFO.  It uses a ready/valid interface
//! on both the input and output sides.
#![doc = badascii_doc::badascii_formal!("
      +------+SyncFIFO+-----------+     
  ?T  |                           | ?T  
+---->| data                 data +---->
      |                           |     
<-----+ ready               ready |<---+
      |                           |     
      +---------------------------+     
")]
//! You can think of the ready signal as `!full`.  Similarly, the `ready` signal
//! is basically equivalent to `next`, as the FIFO will advance if `data` is `Some` and
//! `ready` is `true`.
//!
//!# Example
//!
//!
use crate::core::ram;
use crate::stream::StreamIO;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;
use crate::core::counter::Counter;

#[derive(Clone, Default, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
pub struct SyncFIFO<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    write_logic: write_logic::FIFOWriteCore<T, N>,
    read_logic: read_logic::FIFOReadCore<T, N>,
    ram: ram::option_sync::OptionSyncBRAM<T, N>,
    write_counter: Counter<N>,
    read_counter: Counter<N>,
}

impl<T: Digital, const N: usize> SynchronousIO for SyncFIFO<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = StreamIO<T, T>;
    type O = StreamIO<T, T>;
    type Kernel = fifo_kernel<T, N>;
}

#[kernel]
pub fn fifo_kernel<T: Digital, const N: usize>(
    _cr: ClockReset,
    i: StreamIO<T, T>,
    q: Q<T, N>,
) -> (StreamIO<T, T>, D<T, N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<T, N>::dont_care();
    d.read_counter = q.read_logic.did_read;
    d.write_counter = q.write_logic.did_write;
    d.write_logic.data = i.data;
    d.write_logic.read_address = q.read_counter;
    d.read_logic.write_address = q.write_counter;
    d.read_logic.ready = i.ready;
    d.read_logic.data = q.ram;
    d.ram.write = q.write_logic.data;
    d.ram.read_addr = q.read_logic.ram_read_address;
    let o = StreamIO::<T, T> {
        data: q.read_logic.data_out,
        ready: q.write_logic.ready,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdl_generation_fifo() -> miette::Result<()> {
        let uut = SyncFIFO::<Bits<8>, 3>::default();
        let desc = uut.descriptor(ScopedName::top())?;
        let hdl = desc.hdl()?;
        let verilog = hdl.modules.pretty();
        expect_test::expect_file!["testing/sync_fifo.v"].assert_eq(&verilog);
        Ok(())
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = SyncFIFO::<Bits<8>, 3>::default();
        let descriptor = uut.descriptor(ScopedName::top())?;
        descriptor.check_for_combinatorial_paths()?;
        Ok(())
    }
}
