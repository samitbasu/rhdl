use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<N: BitWidth> {
    filler: crate::fifo::testing::filler::U<N>,
    sender: crate::lid::fifo_to_rv::U<Bits<N>>,
    relay: crate::lid::option_carloni::U<Bits<N>>,
    receiver: crate::lid::rv_to_fifo::U<Bits<N>>,
    drainer: crate::fifo::testing::drainer::U<N>,
}

impl<N: BitWidth> Default for U<N> {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::U::<N>::new(4, 0x8000),
            sender: crate::lid::fifo_to_rv::U::<Bits<N>>::default(),
            relay: crate::lid::option_carloni::U::<Bits<N>>::default(),
            receiver: crate::lid::rv_to_fifo::U::<Bits<N>>::default(),
            drainer: crate::fifo::testing::drainer::U::<N>::new(4, 0x8000),
        }
    }
}

impl<N: BitWidth> SynchronousIO for U<N> {
    type I = ();
    type O = bool;
    type Kernel = single_kernel<N>;
}

#[kernel]
pub fn single_kernel<N: BitWidth>(_cr: ClockReset, _i: (), q: Q<N>) -> (bool, D<N>) {
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

    use super::*;

    #[test]
    fn test_single_trace() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat(())
            .take(5000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("lid");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["a14d4fedccb215481c98ab426be8f9227d2b74c738c0a93fe2454247f544e091"];
        let digest = vcd.dump_to_file(&root.join("single.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_single_is_valid() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat(())
            .take(100_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input)?.last().unwrap();
        assert!(last.value.2);
        Ok(())
    }

    #[test]
    fn test_single_hdl() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat(())
            .take(500)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
