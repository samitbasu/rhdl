use rhdl::prelude::*;

use crate::core::{
    constant, dff,
    slice::{lsbs, msbs},
};

/// A bursty, random FIFO filler.  Uses a sequence of values from an XorShift128 to
/// fill a FIFO.  The lowest N bits of the output number are used as the data.  Based
/// on the random value, the filler will also decide to "sleep" for a number of clock
/// cycles.  This is to simulate a bursty data source.  Note that the behavior is
/// deterministic.  The number of sleep cycles is also fixed, so that a single parameter
/// can be used to control the "burstiness" of the data.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<N: BitWidth> {
    _marker: constant::U<Bits<N>>,
    rng: crate::rng::xorshift::U,
    sleep_counter: dff::U<Bits<W4>>,
    sleep_len: constant::U<Bits<W4>>,
    write_probability: constant::U<Bits<W16>>,
}

/// The default configuration will sleep for 4 counts, with a roughly 50% probability
impl<N: BitWidth> Default for U<N> {
    fn default() -> Self {
        Self {
            _marker: constant::U::new(bits(0)),
            rng: crate::rng::xorshift::U::default(),
            sleep_counter: dff::U::new(bits(0)),
            sleep_len: constant::U::new(bits(4)),
            write_probability: constant::U::new(bits(0x8000)),
        }
    }
}

impl<N: BitWidth> U<N> {
    pub fn new(sleep_len: u8, write_probability: u16) -> Self {
        Self {
            sleep_counter: dff::U::new(bits(0)),
            sleep_len: constant::U::new(bits(sleep_len as u128)),
            write_probability: constant::U::new(bits(write_probability as u128)),
            ..Default::default()
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
pub struct I {
    pub full: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O<N: BitWidth> {
    pub data: Option<Bits<N>>,
}

impl<N: BitWidth> SynchronousIO for U<N> {
    type I = I;
    type O = O<N>;
    type Kernel = filler_kernel<N>;
}

#[kernel]
pub fn filler_kernel<N: BitWidth>(cr: ClockReset, i: I, q: Q<N>) -> (O<N>, D<N>) {
    let mut d = D::<N>::dont_care();
    let mut o = O::<N>::dont_care();
    d.rng = false;
    o.data = None;
    let is_full = i.full;
    d.sleep_counter = q.sleep_counter;
    // If the fifo is not full, and we are not sleeping, then write the next value to the FIFO
    if !is_full && q.sleep_counter == 0 {
        o.data = Some(lsbs::<N, W32>(q.rng));
        d.rng = true;
        let p = msbs::<W16, W32>(q.rng);
        d.sleep_counter = if p > q.write_probability {
            q.sleep_len
        } else {
            bits(0)
        };
    }
    if q.sleep_counter != 0 {
        d.sleep_counter = q.sleep_counter - 1;
    }
    if cr.reset.any() {
        o.data = None;
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_filler() -> miette::Result<()> {
        let uut = U::<W6>::default();
        let input = std::iter::repeat(I { full: false })
            .take(50)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo")
            .join("filler");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["4a05c32ed83ff8ecc2bcfd374f4ebce4d1ae032fc0b582b56dc8f9019c228935"];
        let digest = vcd.dump_to_file(&root.join("filler.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_filler_testbench() -> miette::Result<()> {
        let uut = U::<W6>::default();
        let input = std::iter::repeat(I { full: false })
            .take(50)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let test_module = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        std::fs::write("filler.v", test_module.to_string()).unwrap();
        test_module.run_iverilog()?;
        let test_module = test_bench.flow_graph(&uut, &TestBenchOptions::default())?;
        test_module.run_iverilog()?;
        Ok(())
    }
}
