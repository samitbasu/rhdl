// A synchronous ram wrapped in an interface that accepts Option<T> for writing
use rhdl::prelude::*;

#[derive(PartialEq, Debug, Clone, Default, Synchronous, SynchronousDQ)]
pub struct U<T: Digital + Default, N: BitWidth> {
    inner: super::synchronous::U<T, N>,
}

impl<T: Digital + Default, N: BitWidth> U<T, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            inner: super::synchronous::U::new(initial),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
pub struct I<T: Digital + Default, N: BitWidth> {
    pub read_addr: Bits<N>,
    pub write: Option<(Bits<N>, T)>,
}

impl<T: Digital + Default, N: BitWidth> SynchronousIO for U<T, N> {
    type I = I<T, N>;
    type O = T;
    type Kernel = ram_kernel<T, N>;
}

#[kernel]
pub fn ram_kernel<T: Digital + Default, N: BitWidth>(
    _cr: ClockReset,
    i: I<T, N>,
    q: Q<T, N>,
) -> (T, D<T, N>) {
    let mut d = D::<T, N>::dont_care();
    d.inner.write.enable = false;
    d.inner.write.addr = bits(0);
    d.inner.read_addr = i.read_addr;
    d.inner.write.value = T::default();
    if let Some((addr, data)) = i.write {
        d.inner.write.addr = addr;
        d.inner.write.value = data;
        d.inner.write.enable = true;
    }
    let o = q.inner;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use rhdl::prelude::*;

    use super::*;
    use std::{iter::repeat, path::PathBuf};

    #[derive(PartialEq, Debug, Clone, Copy)]
    enum Cmd {
        Write(b4, b8),
        Read(b4),
    }

    fn rand_cmd() -> Cmd {
        if rand::random() {
            Cmd::Write(
                bits(rand::random::<u128>() % 16),
                bits(rand::random::<u128>() % 256),
            )
        } else {
            Cmd::Read(bits(rand::random::<u128>() % 16))
        }
    }

    struct TestItem(Cmd, b8);

    impl From<Cmd> for I<b8, W4> {
        fn from(cmd: Cmd) -> Self {
            match cmd {
                Cmd::Write(addr, value) => I {
                    read_addr: bits(0),
                    write: Some((addr, value)),
                },
                Cmd::Read(addr) => I {
                    read_addr: addr,
                    write: None,
                },
            }
        }
    }

    #[test]
    fn test_scan_out_ram() -> miette::Result<()> {
        type UC = U<b8, W4>;
        let uut: UC = U::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let test = (0..16)
            .cycle()
            .map(|ndx| TestItem(Cmd::Read(bits(ndx)), bits(15 - ndx)))
            .take(17);
        let inputs = test.clone().map(|item| item.0.into());
        let expected = test.map(|item| item.1).take(16);
        let stream = inputs.stream_after_reset(1).clock_pos_edge(100);
        let sim = uut.run(stream)?;
        let vcd = sim.clone().collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ram")
            .join("option_sync");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["6635cc6c6e6e04f46adeb41770025dc2e469580a516f84d469b44c1556405863"];
        let digest = vcd
            .dump_to_file(&root.join("test_scan_out_option_ram.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        let values = sim
            .glitch_check(|x| (x.value.0.clock, x.value.2))
            .synchronous_sample()
            .skip(2)
            .map(|x| x.value.2);
        assert!(values.eq(expected));
        Ok(())
    }

    fn random_command_stream(
        len: usize,
    ) -> impl Iterator<Item = TimedSample<(ClockReset, I<b8, W4>)>> {
        let inputs = (0..).map(|_| rand_cmd().into()).take(len);
        inputs.stream_after_reset(1).clock_pos_edge(100)
    }

    #[test]
    fn test_hdl_output() -> miette::Result<()> {
        type UC = U<b8, W4>;
        let uut: UC = U::new((0..).map(|ndx| (bits(ndx), bits(0))));
        let stream = random_command_stream(1000);
        let test_bench = uut.run(stream)?.collect::<SynchronousTestBench<_, _>>();
        let test_mod = test_bench.flow_graph(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_then_read() -> miette::Result<()> {
        type UC = U<b8, W4>;
        let uut: UC = U::new(repeat((bits(0), b8::from(0))).take(16));
        let test = vec![
            Cmd::Write(bits(0), bits(72)),
            Cmd::Write(bits(1), bits(99)),
            Cmd::Write(bits(2), bits(255)),
            Cmd::Read(bits(0)),
            Cmd::Read(bits(1)),
            Cmd::Read(bits(2)),
            Cmd::Read(bits(3)),
        ];
        let inputs = test
            .into_iter()
            .map(|x| x.into())
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let sim = uut.run(inputs)?;
        let outputs = sim
            .glitch_check(|x| (x.value.0.clock, x.value.2))
            .synchronous_sample()
            .skip(5)
            .take(3)
            .map(|x| x.value.2)
            .collect::<Vec<_>>();
        assert_eq!(outputs, vec![b8::from(72), b8::from(99), b8::from(255)]);
        Ok(())
    }
}
