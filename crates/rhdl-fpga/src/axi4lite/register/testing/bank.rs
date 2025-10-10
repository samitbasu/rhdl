// Create a fixture with a write manager and a read manager and a AXI bank of registers
use rhdl::prelude::*;

use crate::{
    axi4lite::types::{AXI4Error, AxilAddr, AxilData, WriteCommand},
    core::option::is_some,
};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    writer: crate::axi4lite::basic::manager::write::U,
    reader: crate::axi4lite::basic::manager::read::U,
    bank: crate::axi4lite::register::bank::U<8>,
}

#[derive(PartialEq, Clone, Digital)]
pub struct I {
    pub write: Option<WriteCommand>,
    pub read: Option<AxilAddr>,
}

#[derive(PartialEq, Clone, Digital)]
pub struct O {
    pub write_full: bool,
    pub read_full: bool,
    pub read_data: Option<Result<AxilData, AXI4Error>>,
    pub write_resp: Option<Result<(), AXI4Error>>,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = test_kernel;
}

#[kernel]
pub fn test_kernel(_cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    let mut o = O::dont_care();
    d.writer.cmd = i.write;
    d.reader.cmd = i.read;

    d.bank.axi.read = q.reader.axi;
    d.bank.axi.write = q.writer.axi;
    d.reader.axi = q.bank.axi.read;
    d.writer.axi = q.bank.axi.write;

    o.read_data = q.reader.data;
    o.write_resp = q.writer.resp;

    // Connect the next signals so that they auto-advance
    d.reader.next = is_some::<Result<AxilData, AXI4Error>>(q.reader.data);
    d.writer.next = is_some::<Result<(), AXI4Error>>(q.writer.resp);

    // Connect the full signals - ignored in the test
    o.read_full = q.reader.full;
    o.write_full = q.writer.full;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::axi4lite::types::StrobedData;

    use super::*;

    fn write_cmd(addr: i32, strobe: u8, val: u32) -> I {
        I {
            write: Some(WriteCommand {
                addr: bits(addr as u128),
                strobed_data: StrobedData {
                    data: bits(val as u128),
                    strobe: bits(strobe as u128),
                },
            }),
            read: None,
        }
    }

    fn read_cmd(addr: i32) -> I {
        I {
            write: None,
            read: Some(bits(addr as u128)),
        }
    }

    fn no_cmd() -> I {
        I {
            write: None,
            read: None,
        }
    }

    fn test_stream() -> impl Iterator<Item = I> {
        [
            write_cmd(0, 0b1111, 0x42),
            read_cmd(0),
            write_cmd(4, 0b1111, 0x43),
            read_cmd(4),
            write_cmd(8, 0b1111, 0x45),
            write_cmd(8, 0b1111, 0x42),
            read_cmd(8),
            read_cmd(80),
            write_cmd(0, 0b1000, 0xCA55_AA55),
            write_cmd(4, 0b0100, 0xAAFE_AA55),
            write_cmd(8, 0b0010, 0x55AA_BA55),
            write_cmd(12, 0b0001, 0x55AA_5ABE),
            read_cmd(0),
            read_cmd(4),
            read_cmd(8),
            read_cmd(12),
        ]
        .into_iter()
        .chain(std::iter::repeat(no_cmd()))
        .take(20)
    }

    #[test]
    fn test_register_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(input).collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("bank");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["bbc957efc4e773129b0d0053010845db2a6dea9367a912501fbe32a342187a64"];
        let digest = vcd.dump_to_file(&root.join("register.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_compile_times() -> miette::Result<()> {
        let tic = std::time::Instant::now();
        let uut = U::default();
        let fg = uut.flow_graph("top")?;
        let _top = fg.hdl("top")?;
        let toc = tic.elapsed();
        println!("HDL generation took {:?}", toc);
        Ok(())
    }

    #[test]
    fn test_bank_works() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().with_reset(1).clock_pos_edge(100);
        let io = uut.run(input).synchronous_sample();
        let io = io.filter_map(|x| x.value.2.read_data).collect::<Vec<_>>();
        assert_eq!(
            io,
            vec![
                Ok(bits(0x42)),
                Ok(bits(0x43)),
                Ok(bits(0x42)),
                Err(AXI4Error::DECERR),
                Ok(bits(0xCA00_0042)),
                Ok(bits(0x00FE_0043)),
                Ok(bits(0x0000_BA42)),
                Ok(bits(0x0000_00BE)),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(input).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("bank");
        std::fs::create_dir_all(&root).unwrap();
        let tm = test_bench.flow_graph(
            &uut,
            &TestBenchOptions::default().vcd(&root.join("rbank.vcd").to_string_lossy()),
        )?;
        tm.run_iverilog()?;
        Ok(())
    }
}
