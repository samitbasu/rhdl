use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    filler: crate::fifo::testing::filler::U<U32>,
    source: crate::axi4lite::stream::source::U<Bits<U32>>,
    sink: crate::axi4lite::stream::sink::U<Bits<U32>>,
    drainer: crate::fifo::testing::drainer::U<U32>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::U::new(2, 0x8000),
            source: crate::axi4lite::stream::source::U::default(),
            sink: crate::axi4lite::stream::sink::U::default(),
            drainer: crate::fifo::testing::drainer::U::<U32>::new(2, 0x8000),
        }
    }
}

impl SynchronousIO for U {
    type I = ();
    type O = bool;
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> (bool, D) {
    let mut d = D::dont_care();
    // Feedback the full signal to the filler core
    d.filler.full = q.source.full;
    // The source data comes from the filler object
    d.source.data = q.filler.data;
    // The drainer data comes from the sink object
    d.drainer.data = q.sink.data;
    // The sink full signal comes from the drainer object
    d.sink.next = q.drainer.next;
    // The sink axi comes from the source axi
    d.sink.axi = q.source.axi;
    // The drainer axi comes from the sink axi
    d.source.axi = q.sink.axi;
    (q.drainer.valid, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_stream_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = std::iter::repeat(())
            .take(1000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("stream");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["b106432f21741bc5739e6488cd95bd30708fd7b7c0198d9578699682de4e6aa0"];
        let digest = vcd.dump_to_file(&root.join("stream.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_channel_is_valid() -> miette::Result<()> {
        let uut = U::default();
        let input = std::iter::repeat(())
            .take(100_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let last = uut.run(input)?.last().unwrap();
        assert!(last.value.2);
        Ok(())
    }
}
