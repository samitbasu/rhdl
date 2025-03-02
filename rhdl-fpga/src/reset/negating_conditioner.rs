use rhdl::prelude::*;

#[derive(Debug, Clone, Default, Circuit, CircuitDQ)]
pub struct U<W: Domain, R: Domain> {
    neg: super::negation::U<W>,
    cond: super::conditioner::U<W, R>,
}

#[derive(PartialEq, Digital, Timed)]
pub struct I<W: Domain, R: Domain> {
    pub reset_n: Signal<ResetN, W>,
    pub clock: Signal<Clock, R>,
}

impl<W: Domain, R: Domain> CircuitIO for U<W, R> {
    type I = I<W, R>;
    type O = Signal<Reset, R>;
    type Kernel = negating_conditioner_kernel<W, R>;
}

#[kernel]
pub fn negating_conditioner_kernel<W: Domain, R: Domain>(
    i: I<W, R>,
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

    fn istream() -> impl Iterator<Item = TimedSample<I<Red, Blue>>> {
        // Use a seeded RNG to get repeatable results
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xdead_beef);
        let red = (0..)
            .map(move |_| rng.gen::<u8>() < 200)
            .take(100)
            .stream()
            .clock_pos_edge(100);
        let blue = std::iter::repeat(()).stream().clock_pos_edge(79);
        red.merge(blue, |r, b| I {
            reset_n: signal(reset_n(r.1)),
            clock: signal(b.0.clock),
        })
    }

    #[test]
    fn test_stream_function() -> miette::Result<()> {
        let uut = U::<Red, Blue>::default();
        let stream = istream();
        let vcd = uut.run(stream)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("reset")
            .join("negating_conditioner");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["d540b8b97b19eb4b62624b97c88ccdc7dcf3ed57c157c1d9e4331c485861db49"];
        let digest = vcd
            .dump_to_file(&root.join("negating_conditioner.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::<Red, Blue>::default();
        let stream = istream();
        let tb = uut.run(stream)?.collect::<TestBench<_, _>>();
        let hdl = tb.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        hdl.run_iverilog()?;
        let fg = tb.flow_graph(&uut, &TestBenchOptions::default().skip(10))?;
        fg.run_iverilog()?;
        Ok(())
    }
}
