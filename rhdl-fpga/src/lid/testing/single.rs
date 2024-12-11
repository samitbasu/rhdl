use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<const N: usize> {
    filler: crate::fifo::testing::filler::U<N>,
    relay: crate::lid::option_carloni::U<Bits<N>>,
    drainer: crate::fifo::testing::drainer::U<N>,
}

impl<const N: usize> Default for U<N> {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::U::<N>::new(4, 0x8000),
            relay: crate::lid::option_carloni::U::<Bits<N>>::default(),
            drainer: crate::fifo::testing::drainer::U::<N>::new(4, 0x8000),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = ();
    type O = bool;
    type Kernel = single_kernel<N>;
}

#[kernel]
pub fn single_kernel<const N: usize>(_cr: ClockReset, _i: (), q: Q<N>) -> (bool, D<N>) {
    let mut d = D::<N>::dont_care();
    d.relay.ready = q.drainer.next;
    d.drainer.data = q.relay.data;
    d.relay.data = q.filler.data;
    d.filler.full = !q.relay.ready;
    let o = q.drainer.valid;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_single_trace() -> miette::Result<()> {
        let uut = U::<16>::default();
        let input = std::iter::repeat(())
            .take(5000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("lid");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0e5e76f3176adf6e5eea38c151f8749207de944e1dfd50481609463d8f1ce3f6"];
        let digest = vcd.dump_to_file(&root.join("single.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_single_is_valid() -> miette::Result<()> {
        let uut = U::<16>::default();
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
        let uut = U::<16>::default();
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
