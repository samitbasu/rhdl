//!# Negating Conditioner Core
//!
//! It's fairly frequent that you need to couple an
//! asynchronous (active-low) reset into RHDL, which uses
//! a synchronous active-high reset.  This core combines
//! the [ResetNegation] core with the [ResetConditioner]
//! to provide a simple way to generate a synchronous
//! clock and [Reset] signal from an asynchronous
//! [ResetN] input.
//!
//! The schematic symbol is
//!
#![doc = badascii_doc::badascii_formal!(r"
              Negation               
        +----+Conditioner+---+       
        |                    |       
+------>| reset_n     reset  |------>
        |                    |       
        |             clock  |<-----+
        |                    |       
        +--------------------+       
")]
//!
//!# Example
//!
//! Here is a simple example of the core being
//! used.
//!
//!```
#![doc = include_str!("../../examples/reset_neg_cond.rs")]
//!```
//!
//!With a trace as below.
//!
#![doc = include_str!("../../doc/reset_neg_cond.md")]

use rhdl::prelude::*;

#[derive(Debug, Clone, Default, Circuit, CircuitDQ)]
/// The [NegatingConditioner] core. The reset
/// comes from some domain `W`, and is crossed (and
/// inverted) into domain `R`.
pub struct NegatingConditioner<W: Domain, R: Domain> {
    neg: super::negation::ResetNegation<W>,
    cond: super::conditioner::ResetConditioner<W, R>,
}

#[derive(PartialEq, Digital, Copy, Timed, Clone)]
/// Inputs for the [NegatingConditioner].
pub struct In<W: Domain, R: Domain> {
    /// The active-low reset signal
    pub reset_n: Signal<ResetN, W>,
    /// The clock to synchronize the signal to
    pub clock: Signal<Clock, R>,
}

impl<W: Domain, R: Domain> CircuitIO for NegatingConditioner<W, R> {
    type I = In<W, R>;
    type O = Signal<Reset, R>;
    type Kernel = negating_conditioner_kernel<W, R>;
}

#[kernel]
#[doc(hidden)]
pub fn negating_conditioner_kernel<W: Domain, R: Domain>(
    i: In<W, R>,
    q: Q<W, R>,
) -> (Signal<Reset, R>, D<W, R>) {
    let mut d = D::<W, R>::dont_care();
    d.neg = i.reset_n;
    d.cond.reset = q.neg;
    d.cond.clock = i.clock;
    let o = q.cond;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use rand::{Rng, SeedableRng};

    fn istream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
        // Use a seeded RNG to get repeatable results
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xdead_beef);
        let red = (0..)
            .map(move |_| rng.random::<u8>() < 200)
            .take(100)
            .without_reset()
            .clock_pos_edge(100);
        let blue = std::iter::repeat(()).without_reset().clock_pos_edge(79);
        red.merge(blue, |r, b| In {
            reset_n: signal(reset_n(r.1)),
            clock: signal(b.0.clock),
        })
    }

    #[test]
    fn test_stream_function() -> miette::Result<()> {
        let uut = NegatingConditioner::<Red, Blue>::default();
        let stream = istream();
        let vcd = uut.run(stream).collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("reset")
            .join("negating_conditioner");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["c04cca7cff216750ac453291da72e2f23196e1764e7c73616c8abaad6897224a"];
        let digest = vcd
            .dump_to_file(root.join("negating_conditioner.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let expect = expect_test::expect_file!["negating_conditioner.v.expect"];
        let uut = NegatingConditioner::<Red, Blue>::default();
        let hdl = uut.descriptor("top".into())?.hdl()?.modules.pretty();
        expect.assert_eq(&hdl);
        let stream = istream();
        let tb = uut.run(stream).collect::<TestBench<_, _>>();
        let hdl = tb.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        hdl.run_iverilog()?;
        let fg = tb.ntl(&uut, &TestBenchOptions::default().skip(10))?;
        fg.run_iverilog()?;
        Ok(())
    }
}
