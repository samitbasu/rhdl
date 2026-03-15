use crate::fifo_ng::{
    synchronous::SyncFIFO,
    testing::{drainer::Drainer, prng::PRNG, staller::Staller},
};
use rhdl::prelude::*;

/*
source -> staller -> fifo -> staller2 -> sink
*/

#[derive(Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
pub struct FillFifoDrainTestFixture {
    pub source: PRNG<8>,
    pub staller: Staller<Bits<8>>,
    pub fifo: SyncFIFO<Bits<8>, 3>,
    pub staller2: Staller<Bits<8>>,
    pub sink: Drainer<8>,
}

impl Default for FillFifoDrainTestFixture {
    fn default() -> Self {
        Self {
            source: PRNG::default(),
            staller: Staller::new(0.3, 4),
            fifo: SyncFIFO::default(),
            staller2: Staller::new(0.6, 4),
            sink: Drainer::default(),
        }
    }
}

impl SynchronousIO for FillFifoDrainTestFixture {
    type I = ();
    type O = bool;
    type Kernel = fill_fifo_drain_test;
}

#[kernel]
#[doc(hidden)]
pub fn fill_fifo_drain_test(_cr: ClockReset, _i: (), q: Q) -> (bool, D) {
    let mut d = D::dont_care();
    // Data flow source -> staller -> fifo -> staller2 -> sink
    d.sink.data = q.staller2.data;
    d.staller2.data = q.fifo.data;
    d.fifo.data = q.staller.data;
    d.staller.data = q.source.data;
    // ready flow
    d.source.ready = q.staller.ready;
    d.staller.ready = q.fifo.ready;
    d.fifo.ready = q.staller2.ready;
    d.staller2.ready = q.sink.ready;
    (q.sink.valid, d)
}

#[cfg(test)]
mod tests {
    use crate::stream::stream_io;

    use super::*;

    #[test]
    fn test_fill_drain() -> miette::Result<()> {
        let input = (0..5_000).map(|_| ()).with_reset(1).clock_pos_edge(100);
        let uut = FillFifoDrainTestFixture::default();
        let last = uut.run(input)?.last().unwrap();
        assert!(last.output);
        Ok(())
    }

    #[test]
    fn test_fill_fifo_full() -> miette::Result<()> {
        let input = (0..16)
            .map(|ndx| stream_io::<b8, b8>(Some(b8(ndx)), false))
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = SyncFIFO::<Bits<8>, 4>::default();
        let vcd = uut.run(input)?.collect::<VcdFile>();
        vcd.dump_to_file("fill_fifo_full_test.vcd").unwrap();
        Ok(())
    }
}
