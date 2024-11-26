use rhdl::prelude::*;

use super::manager::ADDR;
use super::manager::DATA;
use super::manager::ID;

// This is a simple test harness that connects a basic manager and subordinate
// into a test fixture.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    manager: super::manager::U,
    subordinate: super::write_bridge::U<ID, DATA, ADDR>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I {
    pub run: bool,
    pub full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = ();
    type Kernel = basic_test_kernel;
}

#[kernel]
pub fn basic_test_kernel(cr: ClockReset, i: I, q: Q) -> ((), D) {
    let mut d = D::init();
    d.manager.axi = q.subordinate.axi;
    d.subordinate.axi = q.manager.axi;
    d.manager.run = i.run;
    d.subordinate.full = i.full;
    ((), d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_trace() {
        let uut = U::default();
        let input = std::iter::repeat(false)
            .take(5)
            .chain(std::iter::repeat(true).take(25))
            .chain(std::iter::repeat(false).take(100))
            .enumerate()
            .map(|(ndx, run)| I { run, full: false })
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input).collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("basic_test.vcd"))
            .unwrap();
    }
}
