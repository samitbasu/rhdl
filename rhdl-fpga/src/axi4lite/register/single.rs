// A simple single register with an AXI bus interface for reading
// and writing it.  Ignores the address information (i.e., the
// register is read or written for any address in the address space
// of the bus).  This is fine, since it is the responsibility of the
// interconnect to ensure non-overlapping address spaces.
use crate::{
    axi4lite::{
        basic::bridge,
        types::{AXI4Error, MISO, MOSI},
    },
    core::{dff, option::unpack},
};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> {
    // We need a read bridge
    read_bridge: bridge::read::U<DATA, ADDR>,
    // And a register to hold the value
    reg: dff::U<Bits<REG_WIDTH>>,
    // And a write bridge
    write_bridge: bridge::write::U<DATA, ADDR>,
}

#[derive(Debug, Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    pub axi: MOSI<DATA, ADDR>,
}

#[derive(Debug, Digital)]
pub struct O<const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> {
    pub axi: MISO<DATA>,
    pub read_data: Bits<REG_WIDTH>,
}

impl<const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> SynchronousIO
    for U<REG_WIDTH, DATA, ADDR>
{
    type I = I<DATA, ADDR>;
    type O = O<REG_WIDTH, DATA, ADDR>;
    type Kernel = single_kernel<REG_WIDTH, DATA, ADDR>;
}

#[kernel]
pub fn single_kernel<const REG_WIDTH: usize, const DATA: usize, const ADDR: usize>(
    _cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<REG_WIDTH, DATA, ADDR>,
) -> (O<REG_WIDTH, DATA, ADDR>, D<REG_WIDTH, DATA, ADDR>) {
    let mut d = D::<REG_WIDTH, DATA, ADDR>::dont_care();
    let mut o = O::<REG_WIDTH, DATA, ADDR>::dont_care();
    // Connect the read bridge inputs and outputs to the bus
    d.read_bridge.axi = i.axi.read;
    o.axi.read = q.read_bridge.axi;
    // Connect the write bridge inputs and outputs to the bus
    d.write_bridge.axi = i.axi.write;
    o.axi.write = q.write_bridge.axi;
    // Connect the register
    d.reg = q.reg;
    // Determine if a read was requested
    let (read_requested, read_addr) = unpack::<Bits<ADDR>>(q.read_bridge.cmd);
    // We can only accept new read commands if the reply sender is not full
    d.read_bridge.cmd_next = false;
    d.read_bridge.reply = None;
    if !q.read_bridge.reply_full && read_requested {
        // Ack the command
        d.read_bridge.cmd_next = true;
        if read_addr == 0 {
            d.read_bridge.reply = Some(Ok(q.reg.resize()));
        } else {
            d.read_bridge.reply = Some(Err(AXI4Error::DECERR));
        }
    }
    // Determine if a write was requested
    let (write_requested, (write_addr, write_data)) =
        unpack::<(Bits<ADDR>, Bits<DATA>)>(q.write_bridge.cmd);
    // We can only accept new write commands if the reply sender is not full
    d.write_bridge.cmd_next = false;
    d.write_bridge.reply = None;
    if !q.write_bridge.reply_full && write_requested {
        // Ack the command
        d.write_bridge.cmd_next = true;
        if write_addr == 0 {
            d.reg = write_data.resize();
            d.write_bridge.reply = Some(Ok(()));
        } else {
            d.write_bridge.reply = Some(Err(AXI4Error::DECERR));
        }
    }
    // Copy out the register
    o.read_data = q.reg;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::axi4lite::types::{ReadMOSI, WriteMOSI};

    use super::*;

    fn write_val(addr: b32, val: b32) -> MOSI<32, 32> {
        MOSI {
            read: ReadMOSI {
                araddr: bits(0),
                arvalid: false,
                rready: true,
            },
            write: WriteMOSI {
                awaddr: addr,
                awvalid: true,
                wdata: val,
                wvalid: true,
                bready: true,
            },
        }
    }

    fn idle_val() -> MOSI<32, 32> {
        MOSI {
            read: ReadMOSI {
                araddr: bits(0),
                arvalid: false,
                rready: true,
            },
            write: WriteMOSI {
                awaddr: bits(0),
                awvalid: false,
                wdata: bits(0),
                wvalid: false,
                bready: true,
            },
        }
    }

    fn test_seq() -> impl Iterator<Item = MOSI<32, 32>> {
        [
            idle_val(),
            idle_val(),
            write_val(bits(0), bits(42)),
            idle_val(),
            write_val(bits(2), bits(49)),
            idle_val(),
            write_val(bits(8), bits(20)),
            idle_val(),
            idle_val(),
            idle_val(),
            idle_val(),
        ]
        .into_iter()
    }

    #[test]
    fn test_register_trace() -> miette::Result<()> {
        let uut = U::<32, 32, 32>::default();
        let input = test_seq()
            .map(|x| I { axi: x })
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("register");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["6a465d35627788b15d2831b75d89c67c56cb3c1d193619ca82da89b0d99df38b"];
        let digest = vcd
            .dump_to_file(&root.join("single_register_test.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::<32, 32, 32>::default();
        let input = test_seq()
            .map(|x| I { axi: x })
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
