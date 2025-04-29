use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<N: BitWidth> {
    filler: crate::fifo::testing::filler::FIFOFiller<N>,
    push_pull: crate::lid::fifo_to_rv::U<Bits<N>>,
    relay1: crate::lid::option_carloni::U<Bits<N>>,
    relay2: crate::lid::option_carloni::U<Bits<N>>,
    drainer: crate::fifo::testing::drainer::FIFODrainer<N>,
}

impl<N: BitWidth> Default for U<N> {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::FIFOFiller::<N>::new(4, 0.5),
            push_pull: crate::lid::fifo_to_rv::U::<Bits<N>>::default(),
            relay1: crate::lid::option_carloni::U::<Bits<N>>::default(),
            relay2: crate::lid::option_carloni::U::<Bits<N>>::default(),
            drainer: crate::fifo::testing::drainer::FIFODrainer::<N>::new(4, 0.5),
        }
    }
}

impl<N: BitWidth> SynchronousIO for U<N> {
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
pub fn double_kernel<N: BitWidth>(_cr: ClockReset, _i: (), q: Q<N>) -> (bool, D<N>) {
    let mut d = D::<N>::dont_care();
    // Fill the data values
    d.push_pull.data = q.filler.data;
    d.relay1.data = q.push_pull.data;
    d.relay2.data = q.relay1.data;
    d.drainer.data = q.relay2.data;
    // Fill the ready values
    d.relay2.ready = q.drainer.next;
    d.relay1.ready = q.relay2.ready;
    d.push_pull.ready = q.relay1.ready;
    d.filler.full = q.push_pull.full;
    let o = q.drainer.valid;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_double_trace() -> miette::Result<()> {
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
        let expect = expect!("eeb3fdebc6a63887ec424a5aabb3b5a57e8dbd5603a6da6575cf5d5fadbba76d");
        let digest = vcd.dump_to_file(&root.join("double.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_double_is_valid() -> miette::Result<()> {
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
    fn test_double_hdl() -> miette::Result<()> {
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
