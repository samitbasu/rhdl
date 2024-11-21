use crate::core::synchronous_ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple synchronous FIFO.  This FIFO is designed to be as simple as possible
/// and thus be robust.  It is a two-port FIFO, with separate read and write
/// ports.  The FIFO is parameterized by the number of bits in each element.
/// The depth of the FIFO is 2^N-1 elements.  You cannot fill the FIFO to 2^N elements.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital, const N: usize> {
    write_logic: write_logic::U<N>,
    read_logic: read_logic::U<N>,
    ram: synchronous_ram::U<T, N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I<T: Digital> {
    data: Option<T>,
    next: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O<T: Digital> {
    data: Option<T>,
    full: bool,
    almost_empty: bool,
    almost_full: bool,
    overflow: bool,
    underflow: bool,
}

impl<T: Digital, const N: usize> SynchronousIO for U<T, N> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = fifo_kernel<T, N>;
}

#[kernel]
pub fn fifo_kernel<T: Digital, const N: usize>(
    _cr: ClockReset,
    i: I<T>,
    q: Q<T, N>,
) -> (O<T>, D<T, N>) {
    // This is essentially a wiring exercise.  The clock
    // and reset are propagated to the sub-elements automatically
    // so we just need to route the signals.
    let mut d = D::<T, N>::init();
    let mut o = O::<T>::init();
    let (write_data, write_enable) = match i.data {
        Some(data) => (data, true),
        None => (T::init(), false),
    };
    // Connect the read logic inputs
    d.read_logic.write_address = q.write_logic.write_address;
    d.read_logic.next = i.next;
    // Connect the write logic inputs
    d.write_logic.read_address = q.read_logic.ram_read_address;
    d.write_logic.write_enable = write_enable;
    // Connect the RAM inputs
    d.ram.write.addr = q.write_logic.ram_write_address;
    d.ram.write.value = write_data;
    d.ram.write.enable = write_enable;
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

    use rhdl::core::sim::ResetOrData;

    use super::*;

    fn write(data: b8) -> I<Bits<8>> {
        I {
            data: Some(data),
            next: false,
        }
    }

    fn read() -> I<Bits<8>> {
        I {
            data: None,
            next: true,
        }
    }

    fn test_seq() -> impl Iterator<Item = TimedSample<(ClockReset, I<Bits<8>>)>> {
        let write_seq = (0..7).map(|i| write(bits(i + 1)));
        let read_seq = (0..7).map(|_| read());
        write_seq
            .chain(read_seq)
            .stream_after_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn check_that_output_is_valid() {
        let uut = U::<b8, 3>::default();
        let stream = test_seq();
        let output = uut
            .run(stream)
            .sample_at_pos_edge(|x| x.value.0.clock)
            .map(|x| x.value.2.data);
        let output = output.flatten().collect::<Vec<_>>();
        assert!(output.iter().all(|x| *x != 0));
        let ramp = output.iter().copied().skip_while(|x| *x == 1);
        assert!(ramp.eq(2..=7));
    }

    #[test]
    fn basic_write_then_read_test() {
        let uut = U::<Bits<8>, 3>::default();
        let stream = test_seq();
        let vcd = uut.run(stream).collect::<Vcd>();
        vcd.dump_to_file(&PathBuf::from("fifo_sync.vcd")).unwrap();
    }

    #[test]
    fn test_hdl_generation_fifo() -> miette::Result<()> {
        let uut = U::<Bits<8>, 3>::default();
        let stream = test_seq();
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &TestBenchOptions::default())?;
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
        type UC = U<Bits<8>, 3>;
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
                    let mut next_input = I {
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
            .sample_at_pos_edge(|x| x.value.0.clock)
            .filter_map(|x| if x.value.1.next { x.value.2.data } else { None })
            .collect::<Vec<_>>();
        assert_eq!(data, read_back);
        Ok(())
    }
}
