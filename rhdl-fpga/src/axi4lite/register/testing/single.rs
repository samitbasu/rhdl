// Create a fixture with a write manager and a read manager and a AXI register
use rhdl::prelude::*;

use crate::{
    axi4lite::types::{AXI4Error, AxilAddr, AxilData, WriteCommand},
    core::option::is_some,
};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    writer: crate::axi4lite::basic::manager::write::U,
    reader: crate::axi4lite::basic::manager::read::U,
    register: crate::axi4lite::register::single::U,
}

#[derive(PartialEq, Digital)]
pub struct I {
    pub write: Option<WriteCommand>,
    pub read: Option<AxilAddr>,
}

#[derive(PartialEq, Digital)]
pub struct O {
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

    d.register.axi.read = q.reader.axi;
    d.register.axi.write = q.writer.axi;
    d.reader.axi = q.register.axi.read;
    d.writer.axi = q.register.axi.write;

    // Set up auto advance
    d.reader.next = is_some::<Result<AxilData, AXI4Error>>(q.reader.data);
    d.writer.next = is_some::<Result<(), AXI4Error>>(q.writer.resp);

    o.read_data = q.reader.data;
    o.write_resp = q.writer.resp;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::axi4lite::types::StrobedData;

    use super::*;

    fn write_cmd(strobe: u8, val: u32) -> I {
        I {
            write: Some(WriteCommand {
                addr: bits(0),
                strobed_data: StrobedData {
                    data: bits(val as u128),
                    strobe: bits(strobe as u128),
                },
            }),
            read: None,
        }
    }

    fn read_cmd() -> I {
        I {
            write: None,
            read: Some(bits(0)),
        }
    }

    fn no_cmd() -> I {
        I {
            write: None,
            read: None,
        }
    }

    fn enable_read(i: I) -> I {
        I {
            write: i.write,
            read: Some(bits(0)),
        }
    }

    fn test_stream() -> impl Iterator<Item = I> {
        [
            write_cmd(0b1111, 42),
            read_cmd(),
            write_cmd(0b1111, 43),
            read_cmd(),
            write_cmd(0b1111, 45),
            write_cmd(0b1111, 42),
            read_cmd(),
            I {
                write: None,
                read: Some(bits(4)),
            },
            // Write DEADBEEF as 4 strobed writes
            write_cmd(0b0001, 0xAA55_AAEF),
            enable_read(write_cmd(0b0010, 0xAA55_BEAA)),
            enable_read(write_cmd(0b0100, 0xAAAD_55AA)),
            enable_read(write_cmd(0b1000, 0xDE55_AA55)),
            read_cmd(),
        ]
        .into_iter()
        .chain(std::iter::repeat(no_cmd()))
        .take(20)
    }

    #[test]
    fn test_register_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("register");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["9654b2cca87d35178fbbe5bdfa6e23d73093137e383104e4fa8aa12c4110206d"];
        let digest = vcd.dump_to_file(&root.join("register.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_compile_times() -> miette::Result<()> {
        env_logger::init();
        let tic = std::time::Instant::now();
        let uut = U::default();
        let _hdl = uut.flow_graph("top")?;
        let toc = tic.elapsed();
        println!("HDL generation took {:?}", toc);
        Ok(())
    }

    #[test]
    fn test_register_works() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().with_reset(1).clock_pos_edge(100);
        let io = uut.run(input)?.synchronous_sample();
        let io = io.filter_map(|x| x.value.2.read_data).collect::<Vec<_>>();
        assert_eq!(
            io,
            vec![
                Ok(bits(42)),
                Ok(bits(43)),
                Ok(bits(42)),
                Err(AXI4Error::DECERR),
                Ok(bits(0x00_00_00_EF)),
                Ok(bits(0x00_00_BE_EF)),
                Ok(bits(0x00_AD_BE_EF)),
                Ok(bits(0xDE_AD_BE_EF)),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
