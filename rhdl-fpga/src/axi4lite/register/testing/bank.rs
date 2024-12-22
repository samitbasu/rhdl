// Create a fixture with a write manager and a read manager and a AXI bank of registers
use rhdl::prelude::*;

use crate::{axi4lite::types::AXI4Error, core::option::is_some};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const DATA: usize, const ADDR: usize> {
    writer: crate::axi4lite::basic::manager::write::U<DATA, ADDR>,
    reader: crate::axi4lite::basic::manager::read::U<DATA, ADDR>,
    bank: crate::axi4lite::register::bank::U<8, DATA, DATA, ADDR>,
}

#[derive(Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    pub write: Option<(Bits<ADDR>, Bits<DATA>)>,
    pub read: Option<Bits<ADDR>>,
}

#[derive(Digital)]
pub struct O<const REG_WIDTH: usize> {
    pub write_full: bool,
    pub read_full: bool,
    pub read_data: Option<Result<Bits<REG_WIDTH>, AXI4Error>>,
    pub write_resp: Option<Result<(), AXI4Error>>,
}

impl<const DATA: usize, const ADDR: usize> SynchronousIO for U<DATA, ADDR> {
    type I = I<DATA, ADDR>;
    type O = O<DATA>;
    type Kernel = test_kernel<DATA, ADDR>;
}

#[kernel]
pub fn test_kernel<const DATA: usize, const ADDR: usize>(
    _cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<DATA, ADDR>,
) -> (O<DATA>, D<DATA, ADDR>) {
    let mut d = D::<DATA, ADDR>::dont_care();
    let mut o = O::<DATA>::dont_care();
    d.writer.cmd = i.write;
    d.reader.cmd = i.read;

    d.bank.axi.read = q.reader.axi;
    d.bank.axi.write = q.writer.axi;
    d.reader.axi = q.bank.axi.read;
    d.writer.axi = q.bank.axi.write;

    o.read_data = q.reader.data;
    o.write_resp = q.writer.resp;

    // Connect the next signals so that they auto-advance
    d.reader.next = is_some::<Result<Bits<DATA>, AXI4Error>>(q.reader.data);
    d.writer.next = is_some::<Result<(), AXI4Error>>(q.writer.resp);

    // Connect the full signals - ignored in the test
    o.read_full = q.reader.full;
    o.write_full = q.writer.full;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    fn write_cmd<const DATA: usize, const ADDR: usize>(addr: i32, val: i32) -> I<DATA, ADDR> {
        I {
            write: Some((bits(addr as u128), bits(val as u128))),
            read: None,
        }
    }

    fn read_cmd<const DATA: usize, const ADDR: usize>(addr: i32) -> I<DATA, ADDR> {
        I {
            write: None,
            read: Some(bits(addr as u128)),
        }
    }

    fn no_cmd<const DATA: usize, const ADDR: usize>() -> I<DATA, ADDR> {
        I {
            write: None,
            read: None,
        }
    }

    fn test_stream<const DATA: usize, const ADDR: usize>() -> impl Iterator<Item = I<DATA, ADDR>> {
        [
            write_cmd(0, 42),
            read_cmd(0),
            write_cmd(4, 43),
            read_cmd(4),
            write_cmd(8, 45),
            write_cmd(8, 42),
            read_cmd(8),
            read_cmd(80),
        ]
        .into_iter()
        .chain(std::iter::repeat(no_cmd()))
        .take(20)
    }

    #[test]
    fn test_register_trace() -> miette::Result<()> {
        let uut = U::<32, 32>::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("bank");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["85d37e292d1a3d2f2f5202aed2994ae64b73d00c815ac9841e4ba554f584480e"];
        let digest = vcd.dump_to_file(&root.join("register.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_compile_times() -> miette::Result<()> {
        let tic = std::time::Instant::now();
        let uut = U::<8, 8>::default();
        let fg = uut.flow_graph("top")?;
        let _top = fg.hdl("top")?;
        let toc = tic.elapsed();
        println!("HDL generation took {:?}", toc);
        Ok(())
    }

    #[test]
    fn test_bank_works() -> miette::Result<()> {
        let uut = U::<32, 32>::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let io = uut.run(input)?.synchronous_sample();
        let io = io.filter_map(|x| x.value.2.read_data).collect::<Vec<_>>();
        assert_eq!(
            io,
            vec![
                Ok(bits(42)),
                Ok(bits(43)),
                Ok(bits(42)),
                Err(AXI4Error::DECERR)
            ]
        );
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::<32, 32>::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        std::fs::write("bank_rtl.v", tm.to_string()).unwrap();
        tm.run_iverilog()?;
        let tm =
            test_bench.flow_graph(&uut, &TestBenchOptions::default().vcd("rbank.vcd").skip(!0))?;
        std::fs::write("test_bench.v", tm.to_string()).unwrap();
        //        tm.run_iverilog()?;
        Ok(())
    }
}
