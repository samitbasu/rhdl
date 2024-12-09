use rhdl::prelude::*;

use crate::axi4lite::basic::bridge;
use crate::axi4lite::basic::manager;

// This is a simple test harness that connects a basic manager and subordinate
// into a test fixture.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    manager: manager::write::U,
    subordinate: bridge::write::U,
}

#[derive(Debug, Digital)]
pub struct I {
    pub cmd: Option<(b32, b32)>,
    pub full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = bool;
    type Kernel = basic_test_kernel;
}

#[kernel]
pub fn basic_test_kernel(_cr: ClockReset, i: I, q: Q) -> (bool, D) {
    let mut d = D::dont_care();
    d.manager.axi = q.subordinate.axi;
    d.subordinate.axi = q.manager.axi;
    d.manager.cmd = i.cmd;
    d.subordinate.full = i.full;
    (true, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_stream() -> impl Iterator<Item = TimedSample<(ClockReset, I)>> {
        std::iter::repeat(None)
            .take(5)
            .chain((0..5).map(|n| Some((bits(n << 3), bits(n)))))
            .chain(std::iter::repeat(None).take(10))
            .map(|x| I {
                cmd: x,
                full: false,
            })
            .stream_after_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn test_transaction_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream();
        let vcd = uut.run(input)?.collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("basic_write_test.vcd"))
            .unwrap();
        Ok(())
    }

    #[test]
    fn test_transaction_hdl() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream();
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
