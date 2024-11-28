use rhdl::prelude::*;

// This is a test harness that connects a random filler, a random drainer
// and a synchronous fifo into a single fixture.  It is easy to monitor the
// output - a single "valid" bit that drops low if the fifo ever yields an
// invalid value.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const N: usize, const Z: usize> {
    filler: crate::fifo::testing::filler::U<N>,
    fifo: crate::fifo::synchronous::U<Bits<N>, Z>,
    drainer: crate::fifo::testing::drainer::U<N>,
}

impl<const N: usize, const Z: usize> SynchronousIO for U<N, Z> {
    type I = ();
    type O = bool;
    type Kernel = fixture_kernel<N, Z>;
}

#[kernel]
pub fn fixture_kernel<const N: usize, const Z: usize>(
    _cr: ClockReset,
    _i: (),
    q: Q<N, Z>,
) -> (bool, D<N, Z>) {
    let mut d = D::<N, Z>::maybe_init();
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
    use super::*;

    #[test]
    fn test_sync_fifo_trace() {
        let uut = U::<16, 6>::default();
        let input = std::iter::repeat(())
            .take(1000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("sync_fifo.vcd"))
            .unwrap();
    }

    #[test]
    fn test_sync_fifo_valid() {
        let uut = U::<16, 6>::default();
        let input = std::iter::repeat(())
            .take(100_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input).last().unwrap();
        assert!(last.value.2);
    }
}
