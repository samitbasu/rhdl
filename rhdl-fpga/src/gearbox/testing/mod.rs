// Create a test with a FIFO writer,
// a FIFO-RV
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    filler: crate::fifo::testing::filler::FIFOFiller<U16>,
    sender: crate::lid::fifo_to_rv::FIFOToReadyValid<b16>,
    reducer: crate::gearbox::reducer::U<U16, U8>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            filler: crate::fifo::testing::filler::FIFOFiller::new(4, 0.5),
            sender: crate::lid::fifo_to_rv::FIFOToReadyValid::default(),
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
        let expect = expect!["bf0dcab77368e53296d25bdf6ab3277e04406f30e0fbcddfb41d8050152a3b1b"];
        let digest = vcd.dump_to_file(&root.join("single.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_lsb_first() -> miette::Result<()> {
        // Instantiate the test circuit (a 16 -> 8 bit reducer with a RNG feeder)
        let uut = U::default();
        // Create an input stream of clock and reset signal
        let input = std::iter::repeat(())
            .take(50)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        // Use the Rust impl of the RNG to predict the 8-bit output values
        let validate = XorShift128::default().flat_map(|x| [x & 0xFF, (x & 0xFF00) >> 8]);
        // Run the Rust co-simulation of the design
        let vals = uut
            .run(input)?
            // Sample the output stream synchronously (on each positive clock edge)
            .synchronous_sample()
            // Extract the output value
            .flat_map(|x| x.value.2)
            // Convert to a raw 32 bit value
            .map(|x| x.raw() as u32);
        // Compare the two iterators by XOR-ing their values
        let mut test = vals.zip(validate).map(|(x, y)| x ^ y);
        // Assert that all is correct.
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
