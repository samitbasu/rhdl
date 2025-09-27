use rhdl::prelude::*;

// This is a test harness that connects a random filler,
// a random drainer and an AXI channel into a single
// fixture.  It is easy to monitor the output - a single
// "full" bit that drops low if the channel ever yields
// an invalid value.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<N: BitWidth> {
    filler: crate::fifo::testing::filler::FIFOFiller<N>,
    sender: crate::axi4lite::channel::sender::U<Bits<N>>,
    receiver: crate::axi4lite::channel::receiver::U<Bits<N>>,
    drainer: crate::fifo::testing::drainer::FIFODrainer<N>,
}

impl<N: BitWidth> SynchronousIO for U<N> {
    type I = ();
    type O = bool;
    type Kernel = fixture_kernel<N>;
}

#[kernel]
pub fn fixture_kernel<N: BitWidth>(_cr: ClockReset, _i: (), q: Q<N>) -> (bool, D<N>) {
    let mut d = D::<N>::dont_care();
    // The filler needs access to the full signal of the sender
    d.filler.full = q.sender.full;
    // The sender input is connected to the filler output
    d.sender.to_send = q.filler.data;
    // The drainer is connected to the data output of the receiver
    d.drainer.data = q.receiver.data;
    // The advance signal of the sender comes from the drainer output
    d.receiver.next = q.drainer.next;
    // The receiver is connected to the sender output
    d.receiver.bus = q.sender.bus;
    // The sender is connected to the receiver output
    d.sender.bus = q.receiver.bus;
    (q.drainer.valid, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_channel_trace() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat(())
            .take(1000)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("channel");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["7b5d5037652ab465c3fa06d7e0c2adcc57973f17010adb90a38523e1bd2e0b20"];
        let digest = vcd.dump_to_file(&root.join("channel.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_channel_is_valid() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat(())
            .take(100_000)
            .with_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input).last().unwrap();
        assert!(last.value.2);
        Ok(())
    }

    #[test]
    fn test_channel_hdl() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat(())
            .take(100)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
