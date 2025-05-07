// A simple single register with an AXI bus interface for reading
// and writing it.  Ignores the address information (i.e., the
// register is read or written for any address in the address space
// of the bus).  This is fine, since it is the responsibility of the
// interconnect to ensure non-overlapping address spaces.
use crate::{
    axi4lite::{
        basic::bridge,
        types::{strobe_to_mask, AXI4Error, AxilAddr, AxilData, WriteCommand, MISO, MOSI},
    },
    core::{dff, option::unpack},
};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    // We need a read bridge
    read_bridge: bridge::read::U,
    // And a register to hold the value
    reg: dff::DFF<AxilData>,
    // And a write bridge
    write_bridge: bridge::write::U,
}

#[derive(PartialEq, Debug, Digital)]
pub struct I {
    pub axi: MOSI,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O {
    pub axi: MISO,
    pub read_data: AxilData,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = single_kernel;
}

#[kernel]
pub fn single_kernel(_cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    let mut o = O::dont_care();
    // Connect the read bridge inputs and outputs to the bus
    d.read_bridge.axi = i.axi.read;
    o.axi.read = q.read_bridge.axi;
    // Connect the write bridge inputs and outputs to the bus
    d.write_bridge.axi = i.axi.write;
    o.axi.write = q.write_bridge.axi;
    // Connect the register
    d.reg = q.reg;
    // Determine if a read was requested
    let (read_requested, read_addr) = unpack::<AxilAddr>(q.read_bridge.cmd);
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
    let (write_requested, write_command) = unpack::<WriteCommand>(q.write_bridge.cmd);
    // We can only accept new write commands if the reply sender is not full
    d.write_bridge.cmd_next = false;
    d.write_bridge.reply = None;
    if !q.write_bridge.reply_full && write_requested {
        // Ack the command
        d.write_bridge.cmd_next = true;
        if write_command.addr == 0 {
            let mask = strobe_to_mask(write_command.strobed_data.strobe);
            d.reg = (write_command.strobed_data.data & mask) | (q.reg & !mask);
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

    fn write_val(addr: b32, val: b32) -> MOSI {
        MOSI {
            read: ReadMOSI {
                araddr: bits(0),
                arvalid: false,
                rready: true,
            },
            write: WriteMOSI {
                awaddr: addr,
                awvalid: true,
                wstrb: bits(0b1111),
                wdata: val,
                wvalid: true,
                bready: true,
            },
        }
    }

    fn idle_val() -> MOSI {
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
                wstrb: bits(0),
                wvalid: false,
                bready: true,
            },
        }
    }

    fn test_seq() -> impl Iterator<Item = MOSI> {
        [
            idle_val(),
            idle_val(),
            write_val(bits(0), bits(42)),
            idle_val(),
            write_val(bits(4), bits(49)),
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
        let uut = U::default();
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
        let expect = expect!["51280e3e71776abbbfe5f98ae210c26b3466bba0505fd86677e0e312bc6af442"];
        let digest = vcd
            .dump_to_file(&root.join("single_register_test.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::default();
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
