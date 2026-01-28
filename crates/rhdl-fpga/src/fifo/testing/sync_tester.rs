//! Sync Tester
//!
//! This is a test harness that connects a random filler, a random drainer
//! and a synchronous fifo into a single fixture.  It is easy to monitor the
//! output - a single "valid" bit that drops low if the fifo ever yields an
//! invalid value.
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
/// This core provides a synchronous test fixture
pub struct SyncTester<const N: usize, const Z: usize>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<Z>: BitWidth,
{
    filler: crate::fifo::testing::filler::FIFOFiller<N>,
    fifo: crate::fifo::synchronous::SyncFIFO<Bits<N>, Z>,
    drainer: crate::fifo::testing::drainer::FIFODrainer<N>,
}

impl<const N: usize, const Z: usize> SynchronousIO for SyncTester<N, Z>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<Z>: BitWidth,
{
    type I = ();
    type O = bool;
    type Kernel = fixture_kernel<N, Z>;
}

#[kernel]
/// Kernel for the [SyncTester]
pub fn fixture_kernel<const N: usize, const Z: usize>(
    _cr: ClockReset,
    _i: (),
    q: Q<N, Z>,
) -> (bool, D<N, Z>)
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<Z>: BitWidth,
{
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

    use super::*;

    #[test]
    fn test_sync_fifo_trace() -> miette::Result<()> {
        let uut = SyncTester::<16, 6>::default();
        let input = std::iter::repeat_n((), 1000)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<VcdFile>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["d5f05eaf48ab71d47dd5fc91b9a5687d20c8aa5196ab7a6601c8cd8177a243ab"];
        let digest = vcd.dump_to_file(root.join("sync_fifo.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_sync_fifo_svg() -> miette::Result<()> {
        let uut = SyncTester::<16, 6>::default();
        let input = std::iter::repeat_n((), 1000)
            .with_reset(1)
            .clock_pos_edge(100)
            .skip_while(|x| x.time < 2000)
            .take_while(|x| x.time <= 3000);
        let svg = uut.run(input).collect::<SvgFile>();
        let options = SvgOptions::default();
        let expect = expect_file!["sync_fifo.svg.expect"];
        expect.assert_eq(&svg.to_string(&options).unwrap());
        Ok(())
    }

    #[test]
    fn test_sync_fifo_valid() -> miette::Result<()> {
        let uut = SyncTester::<16, 6>::default();
        let input = std::iter::repeat_n((), 100_000)
            .with_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input).last().unwrap();
        assert!(last.output);
        Ok(())
    }
}
