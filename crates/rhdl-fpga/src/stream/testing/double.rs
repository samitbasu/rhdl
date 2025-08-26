use rhdl::prelude::*;

use crate::stream::ready;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<N: BitWidth> {
    filler: crate::fifo::testing::filler::FIFOFiller<N>,
    push_pull: crate::stream::fifo_to_stream::FIFOToStream<Bits<N>>,
    relay1: crate::stream::stream_buffer::StreamBuffer<Bits<N>>,
    relay2: crate::stream::stream_buffer::StreamBuffer<Bits<N>>,
    drainer: crate::fifo::testing::drainer::FIFODrainer<N>,
}

impl<N: BitWidth> Default for U<N> {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::FIFOFiller::<N>::new(4, 0.5),
            push_pull: crate::stream::fifo_to_stream::FIFOToStream::<Bits<N>>::default(),
            relay1: crate::stream::stream_buffer::StreamBuffer::<Bits<N>>::default(),
            relay2: crate::stream::stream_buffer::StreamBuffer::<Bits<N>>::default(),
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
    d.relay2.ready = ready::<Bits<N>>(q.drainer.next);
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
        let input = std::iter::repeat_n((), 5000)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("lid");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!("0747ee8fba925094285b9b9ae57cddd3f928642f6f07f43e4f017710a3b85457");
        let digest = vcd.dump_to_file(root.join("double.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_double_is_valid() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat_n((), 100_000)
            .with_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input)?.last().unwrap();
        assert!(last.value.2);
        Ok(())
    }

    #[test]
    fn test_double_hdl() -> miette::Result<()> {
        let uut = U::<U6>::default();
        let input = std::iter::repeat_n((), 500)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
