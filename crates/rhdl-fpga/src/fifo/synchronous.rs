//! A simple synchronous FIFO.
//!
//! This FIFO is designed to be as simple as possible
//! and thus be robust.  It is a two-port FIFO, with separate read and write
//! ports.  The FIFO is parameterized by the number of bits in each element.
//! The depth of the FIFO is 2^N-1 elements.  You cannot fill the FIFO to 2^N elements.
//!
//! Here is the schematic symbol for the FIFO
#![doc = badascii_doc::badascii_formal!("
      +------+SyncFIFO+-----------+     
  ?T  |                           | ?T  
+---->| data                 data +---->
      |                           |     
<-----+ full                 next |<---+
      |                           |     
<-----+ almost_full  almost_empty +---->
      |                           |     
<-----+ overflow        underflow +---->
      |                           |     
      +---------------------------+     
")]
//!
//!# Example
//!
//! Testing a synchronous FIFO is a little tricky, since
//! the input and output interfaces have feedback, and thus
//! do not lend themselves to the regular `input -> uut -> output`
//! type of testing.  
//!
//! Instead, we can either build custom testing harnesses (see [SyncFIFOTester])
//! or use the feedback testing mechanism.  The feedback testing
//! mechanism allows you to provide a closure that computes the next
//! set of inputs given the current outputs.  For synchronous
//! circuits, like this one, the clock is handled for you, so your
//! function need not worry about clock edges.  Here is
//! an example.
//!
//!```
#![doc = include_str!("../../examples/sync_fifo.rs")]
//!```
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/sync_fifo.md")]

use crate::core::ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
/// A simple synchronous FIFO
///    `T` is the data type held by the FIFO.
/// Note that we need `T: Default`.
///  `N` the number bits in the address.  FIFO holds `2^{N-1}` elements
///  when full.
pub struct SyncFIFO<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    write_logic: write_logic::FIFOWriteCore<N>,
    read_logic: read_logic::FIFOReadCore<N>,
    ram: ram::option_sync::OptionSyncBRAM<T, N>,
}

impl<T: Digital, const N: usize> Default for SyncFIFO<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            write_logic: write_logic::FIFOWriteCore::default(),
            read_logic: read_logic::FIFOReadCore::default(),
            ram: ram::option_sync::OptionSyncBRAM::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs for the FIFO
pub struct In<T: Digital> {
    /// The data to be written to the FIFO
    pub data: Option<T>,
    /// The next signal for the read side
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from the FIFO
pub struct Out<T: Digital> {
    /// The output data
    pub data: Option<T>,
    /// The full signal
    pub full: bool,
    /// The almost empty signal
    pub almost_empty: bool,
    /// The almost full signal
    pub almost_full: bool,
    /// The overflow signal
    pub overflow: bool,
    /// The underflow signal
    pub underflow: bool,
}

impl<T: Digital, const N: usize> SynchronousIO for SyncFIFO<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T>;
    type O = Out<T>;
    type Kernel = fifo_kernel<T, N>;
}

#[kernel]
/// The compute kernel for the [SyncFIFO]
pub fn fifo_kernel<T: Digital, const N: usize>(
    _cr: ClockReset,
    i: In<T>,
    q: Q<T, N>,
) -> (Out<T>, D<T, N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    // This is essentially a wiring exercise.  The clock
    // and reset are propagated to the sub-elements automatically
    // so we just need to route the signals.
    let mut d = D::<T, N>::dont_care();
    let mut o = Out::<T>::dont_care();
    // Connect the read logic inputs
    d.read_logic.write_address = q.write_logic.write_address;
    d.read_logic.next = i.next;
    // Connect the write logic inputs
    d.write_logic.read_address = q.read_logic.ram_read_address;
    // Connect the RAM inputs
    d.ram.write = if let Some(data) = i.data {
        d.write_logic.write_enable = true;
        Some((q.write_logic.ram_write_address, data))
    } else {
        d.write_logic.write_enable = false;
        None
    };
    d.ram.read_addr = q.read_logic.ram_read_address;
    // Populate the outputs
    o.data = if q.read_logic.empty {
        None
    } else {
        Some(q.ram)
    };
    o.full = q.write_logic.full;
    o.almost_empty = q.read_logic.almost_empty;
    o.almost_full = q.write_logic.almost_full;
    o.overflow = q.write_logic.overflow;
    o.underflow = q.read_logic.underflow;
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use expect_test::expect;
    use rhdl::core::sim::ResetOrData;

    use super::*;

    fn write(data: b8) -> In<Bits<8>> {
        In {
            data: Some(data),
            next: false,
        }
    }

    fn read() -> In<Bits<8>> {
        In {
            data: None,
            next: true,
        }
    }

    fn test_seq() -> impl Iterator<Item = TimedSample<(ClockReset, In<Bits<8>>)>> {
        let write_seq = (0..7).map(|i| write(bits(i + 1)));
        let read_seq = (0..7).map(|_| read());
        write_seq.chain(read_seq).with_reset(1).clock_pos_edge(100)
    }

    #[test]
    fn check_that_output_is_valid() -> miette::Result<()> {
        let uut = SyncFIFO::<b8, 3>::default();
        let stream = test_seq();
        let output = uut.run(stream).synchronous_sample().map(|x| x.output.data);
        let output = output.flatten().collect::<Vec<_>>();
        assert!(output.iter().all(|x| *x != 0));
        let ramp = output.iter().copied().skip_while(|x| *x == 1);
        assert!(ramp.eq(2..=7));
        Ok(())
    }

    #[test]
    fn basic_write_then_read_test() -> miette::Result<()> {
        let uut = SyncFIFO::<Bits<8>, 3>::default();
        let stream = test_seq();
        let vcd = uut.run(stream).collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo")
            .join("synchronous");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0b412e99ea2905619689d090939884adb9499e54a519f3b92daa8fdb1c23f191"];
        let digest = vcd.dump_to_file(root.join("fifo.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_hdl_generation_fifo() -> miette::Result<()> {
        let uut = SyncFIFO::<Bits<8>, 3>::default();
        let stream = test_seq();
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_fifo_streaming() -> miette::Result<()> {
        // First, allocate a large vector of random data to feed through the FIFO
        let data = (0..1000000)
            .map(|_| bits(rand::random::<u8>() as u128))
            .collect::<Vec<_>>();
        // The writer will write data to the FIFO if it is not full, if there is data to write, and if a random
        // value is true.  The random value determines how often the writer writes data to the FIFO.
        let mut writer_iter = data.iter().copied().fuse();
        // The reader will read data from the FIFO if it is not empty, and if a random value is true.  The random value
        // determines how often the reader reads data from the FIFO.
        type UC = SyncFIFO<Bits<8>, 3>;
        let uut = UC::default();
        let mut writer_finished = false;
        let mut need_reset = true;
        let read_back = uut
            .run_fn(
                |output| {
                    if need_reset {
                        need_reset = false;
                        return Some(ResetOrData::Reset);
                    }
                    let mut next_input = In {
                        data: None,
                        next: false,
                    };
                    if !output.full && rand::random::<u8>() > 50 {
                        next_input.data = writer_iter.next();
                        writer_finished = next_input.data.is_none();
                    }
                    if output.data.is_some() && rand::random::<u8>() > 50 {
                        next_input.next = true;
                    }
                    if writer_finished && output.data.is_none() {
                        return None;
                    }
                    Some(ResetOrData::Data(next_input))
                },
                100,
            )
            //.vcd_file(&PathBuf::from("fifo_streaming.vcd"))
            .synchronous_sample()
            .filter_map(|x| if x.input.1.next { x.output.data } else { None })
            .collect::<Vec<_>>();
        assert_eq!(data, read_back);
        Ok(())
    }
}
