use rhdl::prelude::*;

use crate::axi4lite::basic::bridge;
use crate::axi4lite::basic::manager;
use manager::write::ADDR;
use manager::write::DATA;
use manager::write::ID;

// This is a simple test harness that connects a basic manager and subordinate
// into a test fixture.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    manager: manager::write::U,
    subordinate: bridge::write::U<ID, DATA, ADDR>,
}

#[derive(Debug, Digital)]
pub struct I {
    pub run: bool,
    pub full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = bool;
    type Kernel = basic_test_kernel;
}

#[kernel]
pub fn basic_test_kernel(cr: ClockReset, i: I, q: Q) -> (bool, D) {
    let mut d = D::dont_care();
    d.manager.axi = q.subordinate.axi;
    d.subordinate.axi = q.manager.axi;
    d.manager.run = i.run;
    d.subordinate.full = i.full;
    (true, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_stream() -> impl Iterator<Item = TimedSample<(ClockReset, I)>> {
        std::iter::repeat(false)
            .take(5)
            .chain(std::iter::repeat(true).take(25))
            .chain(std::iter::repeat(false).take(100))
            .map(|run| I { run, full: false })
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
        Ok(())
    }
}
