//! Bursty FIFO Filler
//!
//! A bursty, random FIFO filler.  Uses a sequence of values from an XorShift128 to
//! fill a FIFO.  The lowest N bits of the output number are used as the data.  Based
//! on the random value, the filler will also decide to "sleep" for a number of clock
//! cycles.  This is to simulate a bursty data source.  Note that the behavior is
//! deterministic.  The number of sleep cycles is also fixed, so that a single parameter
//! can be used to control the "burstiness" of the data.
//!
//!# Schematic Symbol
//!
//! The [FIFOFiller] has the following symbol:
//!
#![doc = badascii_formal!(r"
++FIFOFiller++      
|            | ?bN  
|      data  +----->
|            |      
|            |      
|       full |<----+
|            |      
+------------+      
")]
//!
//! Internally, the [FIFOFiller] uses an [XorShift]
//! core to generate a sequence of pseudorandom 32
//! bit values.  These are used to both generate the
//! output data, and to determine if the core will sleep.
//!
use badascii_doc::badascii_formal;
use rhdl::prelude::*;

use crate::{
    core::{
        constant, dff,
        slice::{lsbs, msbs},
    },
    rng::xorshift::XorShift,
};

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// The FIFO Filler core
pub struct FIFOFiller<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    _marker: constant::Constant<Bits<N>>,
    rng: XorShift,
    sleep_counter: dff::DFF<Bits<4>>,
    sleep_len: constant::Constant<Bits<4>>,
    write_probability: constant::Constant<Bits<16>>,
}

/// The default configuration will sleep for 4 counts, with a roughly 50% probability
impl<const N: usize> Default for FIFOFiller<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            _marker: constant::Constant::new(bits(0)),
            rng: crate::rng::xorshift::XorShift::default(),
            sleep_counter: dff::DFF::new(b4(0)),
            sleep_len: constant::Constant::new(b4(4)),
            write_probability: constant::Constant::new(b16(0x8000)),
        }
    }
}

impl<const N: usize> FIFOFiller<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Create a new [FIFOFiller] which writes with probability
    /// `write_probability`, and sleeps otherwise, with a
    /// duration of `sleep_len` cycles.
    pub fn new(sleep_len: u8, write_probability: f32) -> Self {
        let write_probability = 65535.0 * write_probability.clamp(0.0, 1.0);
        Self {
            sleep_counter: dff::DFF::new(b4(0)),
            sleep_len: constant::Constant::new(b4(sleep_len as u128)),
            write_probability: constant::Constant::new(b16(write_probability as u128)),
            ..Default::default()
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to the [FIFOFiller] core
pub struct In {
    /// Input from the `full` signal of the FIFO
    pub full: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from the [FIFOFiller] core
pub struct Out<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The data from the filler to feed into the FIFO.
    pub data: Option<Bits<N>>,
}

impl<const N: usize> SynchronousIO for FIFOFiller<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In;
    type O = Out<N>;
    type Kernel = filler_kernel<N>;
}

#[kernel]
#[doc(hidden)]
pub fn filler_kernel<const N: usize>(cr: ClockReset, i: In, q: Q<N>) -> (Out<N>, D<N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<N>::dont_care();
    let mut o = Out::<N>::dont_care();
    d.rng = false;
    o.data = None;
    let is_full = i.full;
    d.sleep_counter = q.sleep_counter;
    // If the fifo is not full, and we are not sleeping, then write the next value to the FIFO
    if !is_full && q.sleep_counter == 0 {
        o.data = Some(lsbs::<N, 32>(q.rng));
        d.rng = true;
        let p = msbs::<16, 32>(q.rng);
        d.sleep_counter = if p > q.write_probability {
            q.sleep_len
        } else {
            b4(0)
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
    use expect_test::{expect, expect_file};

    use super::*;

    #[test]
    fn test_filler() -> miette::Result<()> {
        let uut = FIFOFiller::<6>::default();
        let input = std::iter::repeat_n(In { full: false }, 50)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<VcdFile>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("fifo")
            .join("filler");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["a0b231a45f5c2e7a8423587638f19a7618cc11fb1a2b5e4581017e45dca3e7e9"];
        let digest = vcd.dump_to_file(root.join("filler.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_filler_testbench() -> miette::Result<()> {
        let uut = FIFOFiller::<6>::default();
        let input = std::iter::repeat_n(In { full: false }, 50)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input).collect::<SynchronousTestBench<_, _>>();
        let test_module = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        let expect = expect_file!["filler.expect"];
        expect.assert_eq(&test_module.to_string());
        test_module.run_iverilog()?;
        let test_module = test_bench.ntl(&uut, &TestBenchOptions::default())?;
        test_module.run_iverilog()?;
        Ok(())
    }
}
