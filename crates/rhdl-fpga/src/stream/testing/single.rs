use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    filler: crate::fifo::testing::filler::FIFOFiller<N>,
    sender: crate::stream::fifo_to_stream::FIFOToStream<Bits<N>>,
    relay: crate::stream::stream_buffer::StreamBuffer<Bits<N>>,
    receiver: crate::stream::stream_to_fifo::StreamToFIFO<Bits<N>>,
    drainer: crate::fifo::testing::drainer::FIFODrainer<N>,
}

impl<const N: usize> Default for U<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::FIFOFiller::<N>::new(4, 0.5),
            sender: crate::stream::fifo_to_stream::FIFOToStream::<Bits<N>>::default(),
            relay: crate::stream::stream_buffer::StreamBuffer::<Bits<N>>::default(),
            receiver: crate::stream::stream_to_fifo::StreamToFIFO::<Bits<N>>::default(),
            drainer: crate::fifo::testing::drainer::FIFODrainer::<N>::new(4, 0.5),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = ();
    type O = bool;
    type Kernel = single_kernel<N>;
}

#[kernel]
pub fn single_kernel<const N: usize>(_cr: ClockReset, _i: (), q: Q<N>) -> (bool, D<N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<N>::dont_care();
    // Connect the drainer to the FIFO side of the receiver
    d.receiver.next = q.drainer.next;
    d.drainer.data = q.receiver.data;
    // Connect the RV side of the receiver to the relay
    d.relay.ready = q.receiver.ready;
    d.receiver.data = q.relay.data;
    // Connect the RV side of the relay to the sender
    d.relay.data = q.sender.data;
    d.sender.ready = q.relay.ready;
    // Connect the FIFO side of the sender to the filler
    d.sender.data = q.filler.data;
    d.filler.full = q.sender.full;
    let o = q.drainer.valid;
    (o, d)
}

#[cfg(test)]
mod tests {

    use expect_test::expect;
    use rhdl::core::circuit::drc;

    use super::*;

    #[test]
    fn test_single_trace() -> miette::Result<()> {
        let uut = U::<6>::default();
        let input = std::iter::repeat_n((), 5000)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("lid");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!("a6b5243e5d1700bc832dd6baee74473edecf906ff0a8169a7ba5a13704d1870c");
        let digest = vcd.dump_to_file(root.join("single.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_single_is_valid() -> miette::Result<()> {
        let uut = U::<6>::default();
        let input = std::iter::repeat_n((), 100_000)
            .with_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input).last().unwrap();
        assert!(last.output);
        Ok(())
    }

    #[test]
    fn test_single_hdl() -> miette::Result<()> {
        let uut = U::<6>::default();
        let input = std::iter::repeat_n((), 500)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = crate::stream::stream_buffer::StreamBuffer::<Bits<16>>::default();
        drc::no_combinatorial_paths(&uut)?;
        let uut = crate::stream::fifo_to_stream::FIFOToStream::<Bits<8>>::default();
        drc::no_combinatorial_paths(&uut)?;
        let uut = crate::stream::stream_to_fifo::StreamToFIFO::<Bits<8>>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
