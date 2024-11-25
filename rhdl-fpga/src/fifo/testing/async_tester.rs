use rhdl::prelude::*;

#[derive(Clone, Circuit, CircuitDQ, Default)]
pub struct U<W: Domain, R: Domain, const N: usize, const Z: usize> {
    filler: Adapter<crate::fifo::testing::filler::U<N>, W>,
    fifo: crate::fifo::asynchronous::U<Bits<N>, W, R, Z>,
    drainer: Adapter<crate::fifo::testing::drainer::U<N>, R>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital, Timed)]
pub struct I<W: Domain, R: Domain> {
    pub cr_w: Signal<ClockReset, W>,
    pub cr_r: Signal<ClockReset, R>,
}

impl<W: Domain, R: Domain, const N: usize, const Z: usize> CircuitIO for U<W, R, N, Z> {
    type I = I<W, R>;
    type O = Signal<bool, R>;
    type Kernel = fixture_kernel<W, R, N, Z>;
}

#[kernel]
pub fn fixture_kernel<W: Domain, R: Domain, const N: usize, const Z: usize>(
    i: I<W, R>,
    q: Q<W, R, N, Z>,
) -> (Signal<bool, R>, D<W, R, N, Z>) {
    let mut d = D::<W, R, N, Z>::init();
    // The filler needs access to the full signal of the FIFO
    d.filler.clock_reset = i.cr_w;
    d.filler.input = signal(crate::fifo::testing::filler::I {
        full: q.fifo.full.val(),
    });
    // The fifo input is connected to the filler output
    d.fifo.cr_r = i.cr_r;
    d.fifo.cr_w = i.cr_w;
    d.fifo.data = signal(q.filler.val().data);
    // The drainer is connected to the data output of the FIFO
    d.drainer.clock_reset = i.cr_r;
    d.drainer.input = signal(crate::fifo::testing::drainer::I::<N> {
        data: q.fifo.data.val(),
    });
    // The advance signal of the FIFO comes from the drainer output
    d.fifo.next = signal(q.drainer.val().next);
    (signal(q.drainer.val().valid), d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_fifo_trace() {
        let uut = U::<Red, Blue, 16, 4> {
            drainer: Adapter::new(crate::fifo::testing::drainer::U::<16>::new(5, 0xD000)),
            ..Default::default()
        };
        let red_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(50);
        let blue_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(78);
        let input = red_input.merge(blue_input, |r, b| I {
            cr_w: signal(r.0),
            cr_r: signal(b.0),
        });
        let vcd = uut.run(input.take(10000)).collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("async_fifo_trace.vcd"))
            .unwrap();
    }

    #[test]
    fn test_async_fifo_works_fast_reader() {
        let uut: U<Red, Blue, 16, 4> = Default::default();
        let red_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(50);
        let blue_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(26);
        let input = red_input.merge(blue_input, |r, b| I {
            cr_w: signal(r.0),
            cr_r: signal(b.0),
        });
        let last = uut.run(input.take(10_000)).last().unwrap();
        assert!(last.value.1.val());
    }

    #[test]
    fn test_async_fifo_works_slow_reader() {
        let uut: U<Red, Blue, 16, 4> = Default::default();
        let red_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(50);
        let blue_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(126);
        let input = red_input.merge(blue_input, |r, b| I {
            cr_w: signal(r.0),
            cr_r: signal(b.0),
        });
        let last = uut.run(input.take(10_000)).last().unwrap();
        assert!(last.value.1.val());
    }

    #[test]
    fn test_async_fifo_test_hdl() -> miette::Result<()> {
        let uut: U<Red, Blue, 16, 4> = Default::default();
        let red_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(50);
        let blue_input = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(126);
        let input = red_input.merge(blue_input, |r, b| I {
            cr_w: signal(r.0),
            cr_r: signal(b.0),
        });
        let test_bench = uut.run(input.take(1_000)).collect::<TestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
