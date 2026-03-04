use crate::fifo::{
    ng_test::{drainer, filler},
    sync::SyncFIFO,
};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
pub struct SyncTester<const N: usize, const Z: usize>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<Z>: BitWidth,
{
    filler: filler::FIFOFiller<N>,
    fifo: SyncFIFO<Bits<N>, Z>,
    drainer: drainer::FIFODrainer<N>,
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
    d.filler.ready = q.fifo.ready;
    // The fifo input is connected to the filler output
    d.fifo.data = q.filler.data;
    // The drainer is connected to the data output of the FIFO
    d.drainer.data = q.fifo.data;
    // The advance signal of the FIFO comes from the drainer output
    d.fifo.ready = q.drainer.ready;
    (q.drainer.valid, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_fifo_trace() -> miette::Result<()> {
        let uut = SyncTester::<16, 6>::default();
        let input = std::iter::repeat_n((), 1000)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<VcdFile>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo");
        std::fs::create_dir_all(&root).unwrap();
        vcd.dump_to_file(root.join("sync_fifo_ng.vcd")).unwrap();
        Ok(())
    }

    #[test]
    fn test_no_combinatorial_path() -> miette::Result<()> {
        let uut = SyncTester::<16, 6>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
