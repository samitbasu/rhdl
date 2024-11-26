use rhdl::prelude::*;

use crate::axi4lite::basic::bridge;
use crate::axi4lite::basic::manager;
use crate::core::option::unpack;
use crate::core::ram;
use manager::read::ADDR;
use manager::read::DATA;
use manager::read::ID;

// This is a simple test harness that connects a basic manager and subordinate
// into a test fixture.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    manager: manager::read::U,
    subordinate: bridge::read::U<ID, DATA, ADDR>,
    memory: ram::synchronous::U<DATA, ADDR>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            manager: manager::read::U::default(),
            subordinate: bridge::read::U::default(),
            memory: ram::synchronous::U::new((0..256).map(|n| (bits(n), bits(n << 8 | n)))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I {
    pub run: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O {
    pub data: Option<DATA>,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = basic_test_kernel;
}

#[kernel]
pub fn basic_test_kernel(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::init();
    d.manager.axi = q.subordinate.axi;
    d.subordinate.axi = q.manager.axi;
    d.manager.run = i.run;
    d.subordinate.data = q.memory;
    // The read bridge uses a read strobe, but we will ignore that
    // for this test case, since the RAM does not care how many times
    // we read it.
    let (_, read_addr) = unpack::<Bits<ADDR>>(q.subordinate.read);
    let mut o = O {
        data: q.manager.data,
    };
    if cr.reset.any() {
        o.data = None;
    }
    d.memory.read_addr = read_addr;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_stream() -> impl Iterator<Item = TimedSample<(ClockReset, I)>> {
        std::iter::repeat(false)
            .take(5)
            .chain(std::iter::repeat(true).take(25))
            .chain(std::iter::repeat(false).take(100))
            .map(|run| I { run })
            .stream_after_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn test_transaction_trace() {
        let uut = U::default();
        let input = test_stream();
        let vcd = uut.run(input).collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("basic_read_test.vcd"))
            .unwrap();
    }

    #[test]
    fn test_that_reads_are_correct() {
        let uut = U::default();
        let input = test_stream();
        let io = uut.run(input);
        let io = io
            .sample_at_pos_edge(|x| x.value.0.clock)
            .flat_map(|x| x.value.2.data)
            .collect::<Vec<_>>();
        let expected = (0..256).map(|n| bits(n << 8 | n)).collect::<Vec<_>>();
        assert_eq!(io, expected[0..io.len()]);
        eprintln!("{:?}", io);
    }
}
