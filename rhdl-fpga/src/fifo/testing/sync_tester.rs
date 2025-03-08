use rhdl::prelude::*;

// This is a test harness that connects a random filler, a random drainer
// and a synchronous fifo into a single fixture.  It is easy to monitor the
// output - a single "valid" bit that drops low if the fifo ever yields an
// invalid value.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<N: BitWidth, Z: BitWidth> {
    filler: crate::fifo::testing::filler::U<N>,
    fifo: crate::fifo::synchronous::U<Bits<N>, Z>,
    drainer: crate::fifo::testing::drainer::U<N>,
}

impl<N: BitWidth, Z: BitWidth> SynchronousIO for U<N, Z> {
    type I = ();
    type O = bool;
    type Kernel = fixture_kernel<N, Z>;
}

#[kernel]
pub fn fixture_kernel<N: BitWidth, Z: BitWidth>(
    _cr: ClockReset,
    _i: (),
    q: Q<N, Z>,
) -> (bool, D<N, Z>) {
    let mut d = D::<N, Z>::dont_care();
    // The filler needs access to the full signal of the FIFO
    d.filler.full = q.fifo.full;
    // The fifo input is connected to the filler output
    d.fifo.data = q.filler.data;
    // The drainer is connected to the data output of the FIFO
    d.drainer.data = q.fifo.data;
    // The advance signal of the FIFO comes from the drainer output
    d.fifo.next = q.drainer.next;
    (q.drainer.valid, d)
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, expect_file};
    use rhdl::core::trace::svg::SvgOptions;

    use super::*;

    #[test]
    fn test_sync_fifo_trace() -> miette::Result<()> {
        let uut = U::<U16, U6>::default();
        let input = std::iter::repeat(())
            .take(1000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["8f9450e679089c0611b60776c37c3832b60a572d23a34abcebcb15ded632e05f"];
        let digest = vcd.dump_to_file(&root.join("sync_fifo.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_sync_fifo_svg() -> miette::Result<()> {
        let uut = U::<U16, U6>::default();
        let input = std::iter::repeat(())
            .take(1000)
            .stream_after_reset(1)
            .clock_pos_edge(100)
            .skip_while(|x| x.time < 2000)
            .take_while(|x| x.time <= 3000);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let options = SvgOptions::default();
        let svg = vcd.dump_svg(&options);
        let expect = expect_file!["sync_fifo.svg.expect"];
        expect.assert_eq(&svg.to_string());
        Ok(())
    }

    #[test]
    fn test_sync_fifo_valid() -> miette::Result<()> {
        let uut = U::<U16, U6>::default();
        let input = std::iter::repeat(())
            .take(100_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input)?.last().unwrap();
        assert!(last.value.2);
        Ok(())
    }
}
