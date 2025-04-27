//! A synchronous ram wrapped in an interface that accepts Option<T> for writing
//!
//! This version of the BRAM replaces the write enable with an Option<T> interface
//! which is more idiomatic for RHDL.  THe schematic symbol looks like this:
//!
#![doc = badascii_doc::badascii!(r#"
         +---+OptionBRAM+-----+       
    B<N> |                    | T     
  +----->|read_addr     output+------>
?(B<N>,T)|                    |       
  +----->|write               |       
         |                    |       
         +--------------------+       
"#)]
//!
//! Internally, the circuitry is very simple.  The option is decoded
//! (combinatorially) into a write enable flag, an address and a
//! data value.  These are then forwarded to the regular synchronous
//! BRAM.
#![doc = badascii_doc::badascii!(r#"
           +-+OptionSyncBRAM+-----------------------------------------+     
           |                                                          |     
           |                               +--+SyncBRAM+---------+    |     
 B<N>      |                         B<N>  |                     |    |  T  
+----------+------------------------------>|read_addr        out +----+---->
           |                     .0  B<N>  |                     |    |     
 ?(B<N>,T) |     ++Unpack+-+   +---------->|write.addr           |    |     
+----------+---->|         |   | .1   T    |                     |    |     
           |     | (B<N>,T)++--+---------->|write.value          |    |     
           |     |         |         bool  |                     |    |     
           |     |    valid+-------------->|write.enable         |    |     
           |     +---------+               |                     |    |     
           |                               +---------------------+    |     
           |                                                          |     
           +----------------------------------------------------------+     
"#)]
use rhdl::prelude::*;

#[derive(PartialEq, Debug, Clone, Default, Synchronous, SynchronousDQ)]
pub struct OptionSyncBRAM<T: Digital + Default, N: BitWidth> {
    inner: super::synchronous::SyncBRAM<T, N>,
}

impl<T: Digital + Default, N: BitWidth> OptionSyncBRAM<T, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            inner: super::synchronous::SyncBRAM::new(initial),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
pub struct In<T: Digital + Default, N: BitWidth> {
    pub read_addr: Bits<N>,
    pub write: Option<(Bits<N>, T)>,
}

impl<T: Digital + Default, N: BitWidth> SynchronousIO for OptionSyncBRAM<T, N> {
    type I = In<T, N>;
    type O = T;
    type Kernel = ram_kernel<T, N>;
}

#[kernel]
pub fn ram_kernel<T: Digital + Default, N: BitWidth>(
    _cr: ClockReset,
    i: In<T, N>,
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
    use std::path::PathBuf;

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

    impl From<Cmd> for In<b8, U4> {
        fn from(cmd: Cmd) -> Self {
            match cmd {
                Cmd::Write(addr, value) => In {
                    read_addr: bits(0),
                    write: Some((addr, value)),
                },
                Cmd::Read(addr) => In {
                    read_addr: addr,
                    write: None,
                },
            }
        }
    }

    #[test]
    fn test_scan_out_ram() -> miette::Result<()> {
        type UC = OptionSyncBRAM<b8, U4>;
        let uut: UC = OptionSyncBRAM::new(
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
        let expect = expect!["6ae49265b7848fe1c8bc2104362a0e21b17df25808b81663aeffff5476524191"];
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
    ) -> impl Iterator<Item = TimedSample<(ClockReset, In<b8, U4>)>> {
        let inputs = (0..).map(|_| rand_cmd().into()).take(len);
        inputs.stream_after_reset(1).clock_pos_edge(100)
    }

    #[test]
    fn test_hdl_output() -> miette::Result<()> {
        type UC = OptionSyncBRAM<b8, U4>;
        let uut: UC = OptionSyncBRAM::new((0..).map(|ndx| (bits(ndx), bits(0))));
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
        type UC = OptionSyncBRAM<b8, U4>;
        let uut: UC = OptionSyncBRAM::new(std::iter::repeat_n((bits(0), b8::from(0)), 16));
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
