// Create a fixture with a write manager and a read manager and a AXI register
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    writer: crate::axi4lite::basic::manager::write::U,
    reader: crate::axi4lite::basic::manager::read::U,
    register: crate::axi4lite::register::single::U<32, 32, 32>,
}

#[derive(Digital)]
pub struct I {
    pub write: Option<(b32, b32)>,
    pub read: Option<b32>,
}

#[derive(Digital)]
pub struct O {
    pub data: Option<b32>,
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
    d.register.read = q.reader.axi;
    d.register.write = q.writer.axi;
    d.reader.axi = q.register.read;
    d.writer.axi = q.register.write;
    o.data = q.reader.data;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_cmd(val: i32) -> I {
        I {
            write: Some((bits(0), bits(val as u128))),
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

    fn test_stream() -> impl Iterator<Item = I> {
        [
            write_cmd(42),
            read_cmd(),
            write_cmd(43),
            read_cmd(),
            write_cmd(44),
            write_cmd(42),
            read_cmd(),
        ]
        .into_iter()
        .chain(std::iter::repeat(no_cmd()))
        .take(20)
    }

    #[test]
    fn test_register_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("register.vcd"))
            .unwrap();
        Ok(())
    }

    #[test]
    fn test_compile_times() -> miette::Result<()> {
        let tic = std::time::Instant::now();
        let uut = U::default();
        let _hdl = uut.hdl("top")?;
        let toc = tic.elapsed();
        println!("HDL generation took {:?}", toc);
        Ok(())
    }

    #[test]
    fn test_register_works() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let io = uut.run(input)?.synchronous_sample();
        let io = io.filter_map(|x| x.value.2.data).collect::<Vec<_>>();
        assert_eq!(io, vec![bits(42), bits(43), bits(42)]);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        //        let tm = test_bench.rtl(&uut, &Default::default())?;
        //tm.run_iverilog()?;
        let tm = test_bench.flow_graph(
            &uut,
            &TestBenchOptions::default().vcd("register_testing.vcd"),
        )?;
        tm.run_iverilog()?;
        Ok(())
    }
}
