use crate::core::ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple synchronous FIFO.  This FIFO is designed to be as simple as possible
/// and thus be robust.  It is a two-port FIFO, with separate read and write
/// ports.  The FIFO is parameterized by the number of bits in each element.
/// The depth of the FIFO is 2^N-1 elements.  You cannot fill the FIFO to 2^N elements.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital + Default, N: BitWidth> {
    write_logic: write_logic::U<N>,
    read_logic: read_logic::U<N>,
    ram: ram::option_sync::U<T, N>,
}

#[derive(PartialEq, Debug, Digital)]
pub struct I<T: Digital> {
    pub data: Option<T>,
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O<T: Digital> {
    pub data: Option<T>,
    pub full: bool,
    pub almost_empty: bool,
    pub almost_full: bool,
    pub overflow: bool,
    pub underflow: bool,
}

impl<T: Digital + Default, N: BitWidth> SynchronousIO for U<T, N> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = fifo_kernel<T, N>;
}

#[kernel]
pub fn fifo_kernel<T: Digital + Default, N: BitWidth>(
    _cr: ClockReset,
    i: I<T>,
    q: Q<T, N>,
) -> (O<T>, D<T, N>) {
    // This is essentially a wiring exercise.  The clock
    // and reset are propagated to the sub-elements automatically
    // so we just need to route the signals.
    let mut d = D::<T, N>::dont_care();
    let mut o = O::<T>::dont_care();
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

    fn write(data: b8) -> I<Bits<U8>> {
        I {
            data: Some(data),
            next: false,
        }
    }

    fn read() -> I<Bits<U8>> {
        I {
            data: None,
            next: true,
        }
    }

    fn test_seq() -> impl Iterator<Item = TimedSample<(ClockReset, I<Bits<U8>>)>> {
        let write_seq = (0..7).map(|i| write(bits(i + 1)));
        let read_seq = (0..7).map(|_| read());
        write_seq
            .chain(read_seq)
            .stream_after_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn check_that_output_is_valid() -> miette::Result<()> {
        let uut = U::<b8, U3>::default();
        let stream = test_seq();
        let output = uut
            .run(stream)?
            .synchronous_sample()
            .map(|x| x.value.2.data);
        let output = output.flatten().collect::<Vec<_>>();
        assert!(output.iter().all(|x| *x != 0));
        let ramp = output.iter().copied().skip_while(|x| *x == 1);
        assert!(ramp.eq(2..=7));
        Ok(())
    }

    #[test]
    fn basic_write_then_read_test() -> miette::Result<()> {
        let uut = U::<Bits<U8>, U3>::default();
        let stream = test_seq();
        let vcd = uut.run(stream)?.collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo")
            .join("synchronous");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["8ae871f0cc557a41a98640bc3e750a9138821e2741f3b4f7adebe413d6d8ce34"];
        let digest = vcd.dump_to_file(&root.join("fifo.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_hdl_generation_fifo() -> miette::Result<()> {
        let uut = U::<Bits<U8>, U3>::default();
        let stream = test_seq();
        let test_bench = uut.run(stream)?.collect::<SynchronousTestBench<_, _>>();
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
        type UC = U<Bits<U8>, U3>;
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
            .synchronous_sample()
            .filter_map(|x| if x.value.1.next { x.value.2.data } else { None })
            .collect::<Vec<_>>();
        assert_eq!(data, read_back);
        Ok(())
    }
}
