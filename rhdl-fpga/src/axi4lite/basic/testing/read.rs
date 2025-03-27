use rhdl::prelude::*;

use crate::axi4lite::basic::bridge;
use crate::axi4lite::basic::manager;
use crate::axi4lite::types::AXI4Error;
use crate::core::dff;
use crate::core::option::unpack;
use crate::core::ram;

//const RAM_ADDR: usize = 8;
type RAM_ADDR = U8;

// This is a simple test harness that connects a basic manager and subordinate
// into a test fixture.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    manager: manager::read::U,
    subordinate: bridge::read::U,
    memory: ram::synchronous::U<Bits<U32>, RAM_ADDR>,
    read_pending: dff::U<bool>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            manager: manager::read::U::default(),
            subordinate: bridge::read::U::default(),
            memory: ram::synchronous::U::new((0..256).map(|n| (bits(n), bits(n << 8 | n)))),
            read_pending: dff::U::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
pub struct I {
    pub cmd: Option<b32>,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O {
    pub data: Option<Result<Bits<U32>, AXI4Error>>,
    pub full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = basic_test_kernel;
}

#[kernel]
pub fn basic_test_kernel(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    d.memory.write.addr = Bits::<RAM_ADDR>::default();
    d.memory.write.value = bits(0);
    d.memory.write.enable = false;
    d.manager.axi = q.subordinate.axi;
    d.subordinate.axi = q.manager.axi;
    d.manager.cmd = i.cmd;
    d.subordinate.cmd_full = false;
    d.read_pending = q.read_pending;
    d.subordinate.reply = None;
    let will_reply = q.subordinate.reply_ready && q.read_pending;
    if will_reply {
        d.subordinate.reply = Some(Ok(q.memory));
        d.read_pending = false;
    }
    let slot_will_be_free = !q.read_pending || will_reply;
    // The read bridge uses a read strobe, but we will ignore that
    // for this test case, since the RAM does not care how many times
    // we read it.
    let (read_request, axi_addr) = unpack::<Bits<U32>>(q.subordinate.cmd);
    let will_issue_read_request = read_request && slot_will_be_free;
    if will_issue_read_request {
        d.read_pending = true;
    }
    let ready = slot_will_be_free || !read_request;
    let read_addr = (axi_addr >> 3).resize();
    let mut o = O {
        data: q.manager.data,
        ready,
    };
    if cr.reset.any() {
        o.data = None;
    }
    d.memory.read_addr = read_addr;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    fn test_stream() -> impl Iterator<Item = TimedSample<(ClockReset, I)>> {
        (0..5)
            .map(|n| Some(bits(n << 3)))
            .chain(std::iter::repeat(None))
            .map(|x| I { cmd: x })
            .take(100)
            .stream_after_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn test_transaction_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream();
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("basic");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["052d0a5c9d9ae4604a4b210631bbfbd6419276a160263228e507a2cf5521f54a"];
        let digest = vcd.dump_to_file(&root.join("basic_read_test.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_that_reads_are_correct() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream();
        let io = uut.run(input)?;
        let io = io
            .synchronous_sample()
            .flat_map(|x| x.value.2.data)
            .collect::<Vec<_>>();
        let expected = (0..256).map(|n| Ok(bits(n << 8 | n))).collect::<Vec<_>>();
        assert_eq!(io, expected[0..io.len()]);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
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
