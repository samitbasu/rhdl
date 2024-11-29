use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<const N: usize> {
    filler: crate::fifo::testing::filler::U<N>,
    relay1: crate::lid::option_carloni::U<Bits<N>>,
    relay2: crate::lid::option_carloni::U<Bits<N>>,
    drainer: crate::fifo::testing::drainer::U<N>,
}

impl<const N: usize> Default for U<N> {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::U::<N>::new(4, 0x8000),
            relay1: crate::lid::option_carloni::U::<Bits<N>>::default(),
            relay2: crate::lid::option_carloni::U::<Bits<N>>::default(),
            drainer: crate::fifo::testing::drainer::U::<N>::new(4, 0x8000),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = ();
    type O = bool;
    type Kernel = double_kernel<N>;
}

//
// To visualize the pipeline, we have:
//
//  Filler ----> Relay1 ----> Relay2 ----> Drainer
//  data  q  -> d       q ->  d     q ->  d
//  ready d <-  q       d <-  q     d <-  q
#[kernel]
pub fn double_kernel<const N: usize>(cr: ClockReset, i: (), q: Q<N>) -> (bool, D<N>) {
    let mut d = D::<N>::dont_care();
    // Fill the data values
    d.relay1.data = q.filler.data;
    d.relay2.data = q.relay1.data;
    d.drainer.data = q.relay2.data;
    // Fill the ready values
    d.relay2.ready = q.drainer.next;
    d.relay1.ready = q.relay2.ready;
    d.filler.full = !q.relay1.ready;
    let o = q.drainer.valid;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_trace() {
        let uut = U::<16>::default();
        let input = std::iter::repeat(())
            .take(5000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("double.vcd"))
            .unwrap();
    }

    #[test]
    fn test_double_is_valid() {
        let uut = U::<16>::default();
        let input = std::iter::repeat(())
            .take(100_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input).last().unwrap();
        assert!(last.value.2);
    }

    #[test]
    fn test_double_hdl() -> miette::Result<()> {
        let uut = U::<16>::default();
        let input = std::iter::repeat(())
            .take(500)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
