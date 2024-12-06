// A synchronous ram wrapped in an interface that accepts Option<T> for writing
use rhdl::prelude::*;

#[derive(Debug, Clone, Default, Synchronous, SynchronousDQ)]
pub struct U<T: Digital, const N: usize> {
    inner: super::synchronous::U<T, N>,
}

impl<T: Digital, const N: usize> U<T, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            inner: super::synchronous::U::new(initial),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct I<T: Digital, const N: usize> {
    pub read_addr: Bits<N>,
    pub write: Option<(Bits<N>, T)>,
}

impl<T: Digital, const N: usize> SynchronousIO for U<T, N> {
    type I = I<T, N>;
    type O = T;
    type Kernel = ram_kernel<T, N>;
}

#[kernel]
pub fn ram_kernel<T: Digital, const N: usize>(
    _cr: ClockReset,
    i: I<T, N>,
    q: Q<T, N>,
) -> (T, D<T, N>) {
    let mut d = D::<T, N>::dont_care();
    d.inner.write.enable = false;
    d.inner.write.addr = bits(0);
    d.inner.read_addr = i.read_addr;
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
    use rhdl::prelude::*;

    use super::*;
    use std::{iter::repeat, path::PathBuf};

    #[derive(Debug, Clone, PartialEq, Copy)]
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

    impl From<Cmd> for I<b8, 4> {
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
        type UC = U<b8, 4>;
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
        vcd.dump_to_file(&PathBuf::from("test_scan_out_option_ram.vcd"))
            .unwrap();
        let values = sim
            .glitch_check(|x| (x.value.0.clock, x.value.2))
            .sample_at_pos_edge(|x| x.value.0.clock)
            .skip(2)
            .map(|x| x.value.2);
        assert!(values.eq(expected));
        Ok(())
    }
}
