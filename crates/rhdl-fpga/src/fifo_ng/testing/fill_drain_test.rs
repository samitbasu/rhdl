use crate::fifo_ng::testing::{drainer::Drainer, prng::PRNG, staller::Staller};
use rhdl::prelude::*;

pub mod single_staller {
    use super::*;

    #[derive(Synchronous, SynchronousDQ)]
    #[rhdl(dq_no_prefix)]
    pub struct FillDrainTestFixture {
        pub source: PRNG<8>,
        pub staller: Staller<Bits<8>>,
        pub sink: Drainer<8>,
    }

    impl Default for FillDrainTestFixture {
        fn default() -> Self {
            Self {
                source: PRNG::default(),
                staller: Staller::new(0.3, 4),
                sink: Drainer::default(),
            }
        }
    }

    impl SynchronousIO for FillDrainTestFixture {
        type I = ();
        type O = bool;
        type Kernel = fill_drain_test;
    }

    #[kernel]
    #[doc(hidden)]
    pub fn fill_drain_test(_cr: ClockReset, _i: (), q: Q) -> (bool, D) {
        let mut d = D::dont_care();
        d.sink.data = q.staller.data;
        d.staller.ready = q.sink.ready;
        d.staller.data = q.source.data;
        d.source.ready = q.staller.ready;
        (q.sink.valid, d)
    }
}

pub mod double_staller {
    use super::*;

    #[derive(Synchronous, SynchronousDQ)]
    #[rhdl(dq_no_prefix)]
    pub struct FillDrainTestFixture {
        pub source: PRNG<8>,
        pub staller1: Staller<Bits<8>>,
        pub staller2: Staller<Bits<8>>,
        pub sink: Drainer<8>,
    }

    impl Default for FillDrainTestFixture {
        fn default() -> Self {
            Self {
                source: PRNG::default(),
                staller1: Staller::new(0.3, 4),
                staller2: Staller::new(0.5, 4),
                sink: Drainer::default(),
            }
        }
    }

    impl SynchronousIO for FillDrainTestFixture {
        type I = ();
        type O = bool;
        type Kernel = fill_drain_test;
    }

    #[kernel]
    #[doc(hidden)]
    pub fn fill_drain_test(_cr: ClockReset, _i: (), q: Q) -> (bool, D) {
        let mut d = D::dont_care();
        // data source -> staller1 -> staller2 -> sink
        d.sink.data = q.staller2.data;
        d.staller2.data = q.staller1.data;
        d.staller1.data = q.source.data;
        // ready flow
        d.source.ready = q.staller1.ready;
        d.staller1.ready = q.staller2.ready;
        d.staller2.ready = q.sink.ready;
        (q.sink.valid, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_drain() -> miette::Result<()> {
        let input = (0..5000).map(|_| ()).with_reset(1).clock_pos_edge(100);
        let uut = single_staller::FillDrainTestFixture::default();
        let last = uut.run(input)?.last().unwrap();
        assert!(last.output);
        Ok(())
    }

    #[test]
    fn test_fill_double_staller_drain() -> miette::Result<()> {
        let input = (0..5000).map(|_| ()).with_reset(1).clock_pos_edge(100);
        let uut = double_staller::FillDrainTestFixture::default();
        let vcd = uut.run(input)?.collect::<VcdFile>();
        vcd.dump_to_file("fill_double_staller_drain_test.vcd")
            .unwrap();
        //        let last = uut.run(input)?.last().unwrap();
        //assert!(last.output);
        Ok(())
    }
}
