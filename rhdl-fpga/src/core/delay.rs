use rhdl::prelude::*;

use super::dff;

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
pub struct U<T: Digital, const N: usize> {
    dffs: [dff::U<T>; N],
}

impl<T: Digital + Default, const N: usize> Default for U<T, N> {
    fn default() -> Self {
        Self {
            dffs: array_init::array_init(|_| dff::U::new(T::default())),
        }
    }
}

impl<T: Digital, const N: usize> SynchronousIO for U<T, N> {
    type I = T;
    type O = T;
    type Kernel = delay<T, N>;
}

#[kernel]
pub fn delay<T: Digital, const N: usize>(_cr: ClockReset, i: T, q: Q<T, N>) -> (T, D<T, N>) {
    let mut d = D::<T, N>::dont_care();
    d.dffs[0] = i;
    for i in 1..N {
        d.dffs[i] = q.dffs[i - 1];
    }
    let o = q.dffs[N - 1];
    (o, d)
}

#[cfg(test)]
mod tests {
    // Check that a single value propagates through the delay line

    use expect_test::expect;

    use super::*;

    fn test_pulse() -> impl Iterator<Item = TimedSample<(ClockReset, Option<Bits<W8>>)>> + Clone {
        std::iter::once(Some(bits(42)))
            .chain(std::iter::repeat(None))
            .take(100)
            .stream_after_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn test_delay_trace() -> miette::Result<()> {
        let uut = U::<Option<Bits<W8>>, 4>::default();
        let input = test_pulse();
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("delay");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["42d51db675648f83fd7aa81a700a198ca19e3612daa8d45b47768732461ebcdb"];
        let digest = vcd.dump_to_file(&root.join("delay.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_delay_works() -> miette::Result<()> {
        let uut = U::<Option<Bits<W8>>, 4>::default();
        let input = test_pulse();
        let output = uut.run(input)?.synchronous_sample();
        let count = output.clone().filter(|t| t.value.2.is_some()).count();
        assert!(count == 1);
        let start_delay = output
            .clone()
            .enumerate()
            .find_map(|(ndx, t)| t.value.1.map(|_| ndx))
            .unwrap();
        let end_delay = output
            .enumerate()
            .find_map(|(ndx, t)| t.value.2.map(|_| ndx))
            .unwrap();
        assert!(end_delay - start_delay == 4);
        Ok(())
    }

    #[test]
    fn test_delay_hdl_works() -> miette::Result<()> {
        let uut = U::<Option<Bits<W8>>, 4>::default();
        let input = test_pulse();
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
