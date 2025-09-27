//! A simple asynchronous FIFO.
//!
//! This FIFO is designed to be as simple as possible
//! and thus be robust.  It is a two-port FIFO, with separate read and write
//! ports.  The FIFO is parameterized by the number of bits in each element.
//! The depth of the FIFO is 2^N-1 elements.  You cannot fill the FIFO to 2^N elements.
//! The FIFO is asynchronous, meaning that the read and write ports are not
//! synchronized to each other.  This means that the read and write ports
//! can be in different clock domains.
//!
//! Here is the schematic symbol for the FIFO
#![doc = badascii_doc::badascii_formal!("
      +-----+AsyncFIFO+------------------------+     
  ?T  |                   +                    | ?T  
+---->| data       W      |     R         data +---->
      |         domain   <+>  domain           |     
<-----+ full              |               next |<---+
      |                   +                    |     
<-----+ almost_full               almost_empty +---->
      |                                        |     
<-----+ overflow                     underflow +---->
      |                                        |     
+---->| cr_w                              cr_r |<---+
      |                                        |     
      +----------------------------------------+     
")]
//!
//!# Example
//!
//! It's difficult to write a simple test case for an
//! [AsyncFIFO] for two reasons.  The first is that
//! it includes two clock domains, and so you need
//! to merge 2 input streams to feed a simulation.  The
//! bigger problem is that the interface has feedback on
//! both sides.  That is to say, that the write interface on the input
//! must respond to the `full` signal in the output, and
//! the read interface on the output must respond to the
//! emptiness of the output (i.e., if `data` is `None`, then
//! the FIFO is empty, and `next = true` will cause an underflow.)
//!
//! Thus to test the FIFO, you can either use an [AsyncFIFOTester], which
//! includes a feed and drain, or you can use the [run_async_red_blue] helper
//! function, which allows you to run a double-feedback simulation with
//! two clocks.  It requires care, as RHDL does not enforce the
//! constraint that the feedback loops only modify inputs that are
//! connected to their clock domain.  So be very careful!
//!
//!```
#![doc = include_str!("../../examples/async_fifo.rs")]
//!```
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/async_fifo.md")]

use crate::cdc::cross_counter;
use crate::core::ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple asynchronous FIFO.  
///  `T` is the data type held by the FIFO.  Must satisfy
/// `T : Default`.
///  `W` the clock domain for the write side of the FIFO.
///  `R` the clock domain for the read side of the FIFO
///  `N` the number bits in the address.  FIFO holds `2^{N-1}` elements
///  when full.
#[derive(Clone, Circuit, CircuitDQ, Default)]
pub struct AsyncFIFO<T: Digital, W: Domain, R: Domain, const N: usize>
where
    Const<N>: BitWidth,
{
    write_logic: Adapter<write_logic::FIFOWriteCore<Const<N>>, W>,
    read_logic: Adapter<read_logic::FIFOReadCore<Const<N>>, R>,
    ram: ram::option_async::OptionAsyncBRAM<T, W, R, Const<N>>,
    read_count_for_write_logic: cross_counter::CrossCounter<R, W, N>,
    write_count_for_read_logic: cross_counter::CrossCounter<W, R, N>,
}

#[derive(PartialEq, Debug, Digital, Timed)]
/// Inputs for the FIFO
pub struct In<T: Digital, W: Domain, R: Domain> {
    /// The data to be written to the FIFO in the W domain
    pub data: Signal<Option<T>, W>,
    /// The next signal for the read logic in the R domain
    pub next: Signal<bool, R>,
    /// The clock and reset for the W domain
    pub cr_w: Signal<ClockReset, W>,
    /// The clock and reset for the R domain
    pub cr_r: Signal<ClockReset, R>,
}

#[derive(PartialEq, Debug, Digital, Timed)]
/// Outputs from the FIFO
pub struct Out<T: Digital, W: Domain, R: Domain> {
    /// The data read from the FIFO in the R domain
    pub data: Signal<Option<T>, R>,
    /// The almost empty flag in the R domain
    pub almost_empty: Signal<bool, R>,
    /// The underflow flag in the R domain
    pub underflow: Signal<bool, R>,
    /// The full flag in the W domain
    pub full: Signal<bool, W>,
    /// The almost full flag in the W domain
    pub almost_full: Signal<bool, W>,
    /// The overflow flag in the W domain
    pub overflow: Signal<bool, W>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for AsyncFIFO<T, W, R, N>
where
    Const<N>: BitWidth,
{
    type I = In<T, W, R>;
    type O = Out<T, W, R>;
    type Kernel = async_fifo_kernel<T, W, R, N>;
}

#[kernel]
/// Async FIFO kernel
pub fn async_fifo_kernel<T: Digital, W: Domain, R: Domain, const N: usize>(
    i: In<T, W, R>,
    q: Q<T, W, R, N>,
) -> (Out<T, W, R>, D<T, W, R, N>)
where
    Const<N>: BitWidth,
{
    let mut d = D::<T, W, R, N>::dont_care();
    // Clock the various components
    d.write_logic.clock_reset = i.cr_w;
    d.read_logic.clock_reset = i.cr_r;
    // Create a struct to drive the inputs of the RAM on the
    // write side.  These signals are all clocked in the write
    // domain.
    let mut ram_write = ram::option_async::WriteI::<T, Const<N>>::dont_care();
    let ram_write_addr = q.write_logic.val().ram_write_address;
    ram_write.clock = i.cr_w.val().clock;
    let mut write_enable = false;
    ram_write.data = if let Some(data) = i.data.val() {
        write_enable = true;
        Some((ram_write_addr, data))
    } else {
        None
    };
    d.ram.write = signal(ram_write);
    // Do the same thing for the read side of the RAM.
    let mut ram_read = ram::asynchronous::ReadI::<Const<N>>::dont_care();
    ram_read.clock = i.cr_r.val().clock;
    ram_read.addr = q.read_logic.val().ram_read_address;
    d.ram.read = signal(ram_read);
    // Provide the write logic with the enable and the
    // read address as determined by the split counter.
    d.write_logic.input = signal(write_logic::In::<Const<N>> {
        read_address: q.read_count_for_write_logic.count.val(),
        write_enable,
    });
    // Provide the read logic with the next signal and the
    // write address as determined by the split counter.
    d.read_logic.input = signal(read_logic::In::<Const<N>> {
        next: i.next.val(),
        write_address: q.write_count_for_read_logic.count.val(),
    });
    // Feed the read count to the read counter
    d.read_count_for_write_logic.incr = signal(q.read_logic.val().will_advance);
    d.read_count_for_write_logic.incr_cr = i.cr_r;
    d.read_count_for_write_logic.cr = i.cr_w;
    // Feed the write count to the write counter
    d.write_count_for_read_logic.incr = signal(write_enable);
    d.write_count_for_read_logic.incr_cr = i.cr_w;
    d.write_count_for_read_logic.cr = i.cr_r;
    // Populate the output signals
    let mut o = Out::<T, W, R>::dont_care();
    o.data = signal(if q.read_logic.val().empty {
        None
    } else {
        Some(q.ram.val())
    });
    o.full = signal(q.write_logic.val().full);
    o.almost_empty = signal(q.read_logic.val().almost_empty);
    o.almost_full = signal(q.write_logic.val().almost_full);
    o.overflow = signal(q.write_logic.val().overflow);
    o.underflow = signal(q.read_logic.val().underflow);
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn basic_write_test() -> miette::Result<()> {
        let write = (0..16)
            .map(|x| Some(bits(x)))
            .chain(std::iter::repeat(None))
            .with_reset(1)
            .clock_pos_edge(100);
        let read = std::iter::repeat_n(false, 32)
            .chain(std::iter::repeat_n(true, 16))
            .with_reset(1)
            .clock_pos_edge(75);
        let input = write.merge(read, |w, r| In {
            data: signal(w.1),
            next: signal(r.1),
            cr_w: signal(w.0),
            cr_r: signal(r.0),
        });
        //        let input = test_stream();
        let uut = AsyncFIFO::<Bits<U8>, Red, Blue, 5>::default();
        let vcd = uut.run(input.clone()).collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo")
            .join("asynchronous");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["ae4eb2d25a3424831f5217372db1c76702815b266d0770ef4251d5bb41d25ab5"];
        let digest = vcd
            .dump_to_file(root.join("async_fifo_write_test.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        let test_bench = uut.run(input).collect::<TestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
