use crate::cdc::cross_counter;
use crate::core::ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple asynchronous FIFO.  This FIFO is designed to be as simple as possible
/// and thus be robust.  It is a two-port FIFO, with separate read and write
/// ports.  The FIFO is parameterized by the number of bits in each element.
/// The depth of the FIFO is 2^N-1 elements.  You cannot fill the FIFO to 2^N elements.
/// The FIFO is asynchronous, meaning that the read and write ports are not
/// synchronized to each other.  This means that the read and write ports
/// can be in different clock domains.
#[derive(Clone, Circuit, CircuitDQ, Default)]
pub struct U<T: Digital, W: Domain, R: Domain, const N: usize> {
    write_logic: Adapter<write_logic::U<N>, W>,
    read_logic: Adapter<read_logic::U<N>, R>,
    ram: ram::asynchronous::U<T, W, R, N>,
    read_count_for_write_logic: cross_counter::U<R, W, N>,
    write_count_for_read_logic: cross_counter::U<W, R, N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital, Timed)]
pub struct I<T: Digital, W: Domain, R: Domain> {
    /// The data to be written to the FIFO in the W domain
    pub data: Signal<Option<T>, W>,
    /// The next signal for the read logic in the R domain
    pub next: Signal<bool, R>,
    /// The clock and reset for the W domain
    pub cr_w: Signal<ClockReset, W>,
    /// The clock and reset for the R domain
    pub cr_r: Signal<ClockReset, R>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital, Timed)]
pub struct O<T: Digital, W: Domain, R: Domain> {
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

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for U<T, W, R, N> {
    type I = I<T, W, R>;
    type O = O<T, W, R>;
    type Kernel = async_fifo_kernel<T, W, R, N>;
}

#[kernel]
pub fn async_fifo_kernel<T: Digital, W: Domain, R: Domain, const N: usize>(
    i: I<T, W, R>,
    q: Q<T, W, R, N>,
) -> (O<T, W, R>, D<T, W, R, N>) {
    let (write_data, write_enable) = match i.data.val() {
        Some(data) => (data, true),
        None => (T::dont_care(), false),
    };
    let mut d = D::<T, W, R, N>::dont_care();
    // Clock the various components
    d.write_logic.clock_reset = i.cr_w;
    d.read_logic.clock_reset = i.cr_r;
    // Create a struct to drive the inputs of the RAM on the
    // write side.  These signals are all clocked in the write
    // domain.
    let mut ram_write = ram::asynchronous::WriteI::<T, N>::dont_care();
    ram_write.clock = i.cr_w.val().clock;
    ram_write.data = write_data;
    ram_write.enable = write_enable;
    ram_write.addr = q.write_logic.val().ram_write_address;
    d.ram.write = signal(ram_write);
    // Do the same thing for the read side of the RAM.
    let mut ram_read = ram::asynchronous::ReadI::<N>::dont_care();
    ram_read.clock = i.cr_r.val().clock;
    ram_read.addr = q.read_logic.val().ram_read_address;
    d.ram.read = signal(ram_read);
    // Provide the write logic with the enable and the
    // read address as determined by the split counter.
    d.write_logic.input = signal(write_logic::I::<{ N }> {
        read_address: q.read_count_for_write_logic.count.val(),
        write_enable,
    });
    // Provide the read logic with the next signal and the
    // write address as determined by the split counter.
    d.read_logic.input = signal(read_logic::I::<{ N }> {
        next: i.next.val(),
        write_address: q.write_count_for_read_logic.count.val(),
    });
    // Feed the read count to the read counter
    d.read_count_for_write_logic.data = signal(q.read_logic.val().will_advance);
    d.read_count_for_write_logic.data_cr = i.cr_r;
    d.read_count_for_write_logic.cr = i.cr_w;
    // Feed the write count to the write counter
    d.write_count_for_read_logic.data = signal(write_enable);
    d.write_count_for_read_logic.data_cr = i.cr_w;
    d.write_count_for_read_logic.cr = i.cr_r;
    // Populate the output signals
    let mut o = O::<T, W, R>::dont_care();
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
    use super::*;
    use std::path::PathBuf;

    fn test_stream() -> impl Iterator + Clone {
        let write = (0..16)
            .map(|x| Some(b8(x)))
            .chain(std::iter::repeat(None))
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let read = std::iter::repeat(false)
            .take(32)
            .chain(std::iter::repeat(true).take(16))
            .stream_after_reset(1)
            .clock_pos_edge(75);
        let input = write.merge(read, |w, r| I {
            data: signal::<_, Red>(w.1),
            next: signal::<_, Blue>(r.1),
            cr_w: signal::<_, Red>(w.0),
            cr_r: signal::<_, Blue>(r.0),
        });
        input
    }

    #[test]
    fn basic_write_test() -> miette::Result<()> {
        let write = (0..16)
            .map(|x| Some(bits(x)))
            .chain(std::iter::repeat(None))
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let read = std::iter::repeat(false)
            .take(32)
            .chain(std::iter::repeat(true).take(16))
            .stream_after_reset(1)
            .clock_pos_edge(75);
        let input = write.merge(read, |w, r| I {
            data: signal(w.1),
            next: signal(r.1),
            cr_w: signal(w.0),
            cr_r: signal(r.0),
        });
        //        let input = test_stream();
        let uut = U::<Bits<8>, Red, Blue, 5>::default();
        let vcd = uut.run(input.clone())?.collect::<Vcd>();
        vcd.dump_to_file(&PathBuf::from("async_fifo_write_test.vcd"))
            .unwrap();
        let test_bench = uut.run(input)?.collect::<TestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        let tm =
            test_bench.flow_graph(&uut, &TestBenchOptions::default().vcd("test_bench_fg.vcd"))?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_hdl_generation() {}
}
