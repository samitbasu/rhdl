use rhdl::prelude::*;

use crate::axi4lite::types::{AxilData, MISO, MOSI};

// The fixture allows you to take an AXI-interfaced synchronous core and
// connect it an actual AXI bus with a clock and asynchronous negative reset.
#[derive(Clone, Circuit, CircuitDQ, Default)]
pub struct U<
    W: Domain, // Clock domain for the reset signal
    R: Domain, // Clock domain for everything else
> {
    pub resetn_conditioner: crate::reset::negating_conditioner::NegatingConditioner<W, R>,
    pub register: Adapter<crate::axi4lite::register::single::U, R>,
}

#[derive(PartialEq, Digital, Timed, Clone)]
pub struct I<W: Domain, R: Domain> {
    pub reset_n: Signal<ResetN, W>,
    pub clock: Signal<Clock, R>,
    pub axi: Signal<MOSI, R>,
}

#[derive(PartialEq, Digital, Timed, Clone)]
pub struct O<R: Domain> {
    pub axi: Signal<MISO, R>,
    pub read_data: Signal<AxilData, R>,
}

impl<W: Domain, R: Domain> CircuitIO for U<W, R> {
    type I = I<W, R>;
    type O = O<R>;
    type Kernel = fixture_kernel<W, R>;
}

#[kernel]
pub fn fixture_kernel<W: Domain, R: Domain>(i: I<W, R>, q: Q<W, R>) -> (O<R>, D<W, R>) {
    let mut d = D::<W, R>::dont_care();
    let mut o = O::<R>::dont_care();
    // Connect the reset conditioner
    d.resetn_conditioner.reset_n = i.reset_n;
    d.resetn_conditioner.clock = i.clock;
    // Connect the register
    d.register.clock_reset = signal(ClockReset {
        clock: i.clock.val(),
        reset: q.resetn_conditioner.val(),
    });
    // Connect the register's axi bus inputs to the fixture's axi bus input
    d.register.input = signal(crate::axi4lite::register::single::I { axi: i.axi.val() });
    // Connect the axi bus output signals
    o.axi = signal(q.register.val().axi);
    o.read_data = signal(q.register.val().read_data);
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, expect_file};

    use crate::axi4lite::types::{ReadMOSI, WriteMOSI};

    use super::*;

    fn axi_null_cmd() -> MOSI {
        MOSI {
            write: WriteMOSI {
                awaddr: bits(0),
                awvalid: false,
                wdata: bits(0),
                wstrb: bits(0),
                wvalid: false,
                bready: true,
            },
            read: ReadMOSI {
                araddr: bits(0),
                arvalid: false,
                rready: true,
            },
        }
    }

    fn axi_write_cmd(addr: b32, data: b32) -> MOSI {
        MOSI {
            write: WriteMOSI {
                awaddr: addr,
                awvalid: true,
                wdata: data,
                wstrb: bits(0b1111),
                wvalid: true,
                bready: true,
            },
            read: ReadMOSI {
                araddr: bits(0),
                arvalid: false,
                rready: true,
            },
        }
    }

    fn axi_read_cmd(addr: b32) -> MOSI {
        MOSI {
            write: WriteMOSI {
                awaddr: bits(0),
                awvalid: false,
                wdata: bits(0),
                wstrb: bits(0),
                wvalid: false,
                bready: true,
            },
            read: ReadMOSI {
                araddr: addr,
                arvalid: true,
                rready: true,
            },
        }
    }

    // Create a test stream that writes 42, 47, 49 to address 0,
    // with reads after each one.
    fn axi_test_seq() -> impl Iterator<Item = MOSI> {
        [
            axi_null_cmd(),
            axi_null_cmd(),
            axi_null_cmd(),
            axi_write_cmd(bits(0), bits(42)),
            axi_read_cmd(bits(0)),
            axi_null_cmd(),
            axi_write_cmd(bits(0), bits(47)),
            axi_read_cmd(bits(0)),
            axi_write_cmd(bits(0), bits(49)),
            axi_read_cmd(bits(0)),
            axi_null_cmd(),
            axi_null_cmd(),
            axi_null_cmd(),
            axi_null_cmd(),
            axi_null_cmd(),
        ]
        .into_iter()
    }

    fn test_stream() -> impl Iterator<Item = TimedSample<I<Red, Blue>>> {
        let red = (0_usize..).with_reset(1).clock_pos_edge(100);
        let blue = axi_test_seq().without_reset().clock_pos_edge(79);
        red.merge(blue, |r, b| I {
            reset_n: signal(reset_n(!r.0.reset.any())),
            clock: signal(b.0.clock),
            axi: signal(b.1),
        })
    }

    #[test]
    fn test_trace() -> miette::Result<()> {
        let uut = U::<Red, Blue>::default();
        let stream = test_stream();
        let vcd = uut.run(stream).collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("register");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["b8ecb5217958a903bd08681020ac1773f66a59f8c5804275faa3473b07e0e9b2"];
        let digest = vcd
            .dump_to_file(&root.join("axi4lite_register.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_export_fixture() -> miette::Result<()> {
        let uut = U::<Red, Blue>::default();
        let mut top = Fixture::new("top", uut);
        top.pass_through_input("aclk", &path!(.clock))?; // Master AXI4-Lite clock
        top.pass_through_input("aresetn", &path!(.reset_n))?; // Master AXI4-Lite reset
        top.pass_through_input("s_axi_awaddr", &path!(.axi.val().write.awaddr))?; // AXI4-Lite slave: Write address
        top.pass_through_input("s_axi_awvalid", &path!(.axi.val().write.awvalid))?; // AXI4-Lite slave: Write address valid
        top.pass_through_output("s_axi_awready", &path!(.axi.val().write.awready))?; // AXI4-Lite slave: Write address ready
        top.pass_through_input("s_axi_wdata", &path!(.axi.val().write.wdata))?; // AXI4-Lite slave: Write data
        top.pass_through_input("s_axi_wstrb", &path!(.axi.val().write.wstrb))?; // AXI4-Lite slave: Write strobe
        top.pass_through_input("s_axi_wvalid", &path!(.axi.val().write.wvalid))?; // AXI4-Lite slave: Write data valid
        top.pass_through_output("s_axi_wready", &path!(.axi.val().write.wready))?; // AXI4-Lite slave: Write data ready
        top.pass_through_output("s_axi_bresp", &path!(.axi.val().write.bresp))?; // AXI4-Lite slave: Write response
        top.pass_through_output("s_axi_bvalid", &path!(.axi.val().write.bvalid))?; // AXI4-Lite slave: Write response valid
        top.pass_through_input("s_axi_bready", &path!(.axi.val().write.bready))?; // AXI4-Lite slave: Write response ready
        top.pass_through_input("s_axi_araddr", &path!(.axi.val().read.araddr))?; // AXI4-Lite slave: Read address
        top.pass_through_input("s_axi_arvalid", &path!(.axi.val().read.arvalid))?; // AXI4-Lite slave: Read address valid
        top.pass_through_output("s_axi_arready", &path!(.axi.val().read.arready))?; // AXI4-Lite slave: Read address ready
        top.pass_through_output("s_axi_rdata", &path!(.axi.val().read.rdata))?; // AXI4-Lite slave: Read data
        top.pass_through_output("s_axi_rresp", &path!(.axi.val().read.rresp))?; // AXI4-Lite slave: Read data response
        top.pass_through_output("s_axi_rvalid", &path!(.axi.val().read.rvalid))?; // AXI4-Lite slave: Read data valid
        top.pass_through_input("s_axi_rready", &path!(.axi.val().read.rready))?; // AXI4-Lite slave: Read data ready
        top.pass_through_output("data", &path!(.read_data))?; // Register read data))?;    Ok(())
        let module = top.module()?;
        let expect = expect_file!["axi_reg_module.v.expect"];
        expect.assert_eq(&module.to_string());
        Ok(())
    }
}
