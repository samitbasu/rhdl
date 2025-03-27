// Create a test with a FIFO writer,
// a FIFO-RV
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    filler: crate::fifo::testing::filler::U<U16>,
    sender: crate::lid::fifo_to_rv::U<b16>,
    reducer: crate::gearbox::reducer::U<U16, U8>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::U::new(4, 0x8000),
            sender: crate::lid::fifo_to_rv::U::default(),
            reducer: crate::gearbox::reducer::U::default(),
        }
    }
}

impl SynchronousIO for U {
    type I = ();
    type O = Option<b8>;
    type Kernel = kernel;
}

//
// The chain is
//   filler  sender  reducer  receiver
//    d  q    d  q    d   q     d   q
#[kernel]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> (Option<b8>, D) {
    let mut d = D::dont_care();
    d.reducer.ready = true;
    d.reducer.data = q.sender.data;
    d.sender.ready = q.reducer.ready;
    d.filler.full = q.sender.full;
    d.sender.data = q.filler.data;
    (q.reducer.data, d)
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, expect_file};
    use rhdl::core::circuit::drc;

    use crate::rng::xorshift::XorShift128;

    use super::*;

    #[test]
    fn test_single_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = std::iter::repeat(())
            .take(5000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("gearbox");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["7b1ea08421646c2e13eea4b1183e191badf1fab4cc213c61015ec4e154e395ce"];
        let digest = vcd.dump_to_file(&root.join("single.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_lsb_first() -> miette::Result<()> {
        let uut = U::default();
        let input = std::iter::repeat(())
            .take(50)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let validate = XorShift128::default().flat_map(|x| [x & 0xFF, (x & 0xFF00) >> 8]);
        let vals = uut
            .run(input)?
            .synchronous_sample()
            .flat_map(|x| x.value.2)
            .map(|x| x.raw() as u32);
        let mut test = vals.zip(validate).map(|(x, y)| x ^ y);
        assert!(test.all(|x| x == 0));
        Ok(())
    }

    #[test]
    fn test_as_hdl() -> miette::Result<()> {
        let uut = crate::gearbox::reducer::U::<U16, U8>::default();
        let hdl = uut.hdl("top")?;
        let expect = expect_file!["hdl.expect"];
        expect.assert_eq(&hdl.as_module().to_string());
        Ok(())
    }

    #[test]
    fn test_hdl_testbench() -> miette::Result<()> {
        let uut = U::default();
        let input = std::iter::repeat(())
            .take(50)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
