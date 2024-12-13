use rhdl::prelude::*;

use crate::axi4lite::types::{MISO, MOSI};

// The fixture allows you to take an AXI-interfaced synchronous core and
// connect it an actual AXI bus with a clock and asynchronous negative reset.
#[derive(Clone, Circuit, CircuitDQ, Default)]
pub struct U<
    W: Domain, // Clock domain for the reset signal
    R: Domain, // Clock domain for everything else
    const REG_WIDTH: usize = 8,
    const DATA: usize = 32,
    const ADDR: usize = 32,
> {
    pub resetn_conditioner: crate::reset::negating_conditioner::U<W, R>,
    pub register: Adapter<crate::axi4lite::register::single::U<REG_WIDTH, DATA, ADDR>, R>,
}

#[derive(Digital, Timed)]
pub struct I<W: Domain, R: Domain, const DATA: usize, const ADDR: usize> {
    pub reset_n: Signal<ResetN, W>,
    pub clock: Signal<Clock, R>,
    pub axi: Signal<MOSI<DATA, ADDR>, R>,
}

#[derive(Digital, Timed)]
pub struct O<R: Domain, const REG_WIDTH: usize, const DATA: usize> {
    pub axi: Signal<MISO<DATA>, R>,
    pub read_data: Signal<Bits<REG_WIDTH>, R>,
}

impl<W: Domain, R: Domain, const REG_WIDTH: usize, const DATA: usize, const ADDR: usize> CircuitIO
    for U<W, R, REG_WIDTH, DATA, ADDR>
{
    type I = I<W, R, DATA, ADDR>;
    type O = O<R, REG_WIDTH, DATA>;
    type Kernel = fixture_kernel<W, R, REG_WIDTH, DATA, ADDR>;
}

#[kernel]
pub fn fixture_kernel<
    W: Domain,
    R: Domain,
    const REG_WIDTH: usize,
    const DATA: usize,
    const ADDR: usize,
>(
    i: I<W, R, DATA, ADDR>,
    q: Q<W, R, REG_WIDTH, DATA, ADDR>,
) -> (O<R, REG_WIDTH, DATA>, D<W, R, REG_WIDTH, DATA, ADDR>) {
    let mut d = D::<W, R, REG_WIDTH, DATA, ADDR>::dont_care();
    let mut o = O::<R, REG_WIDTH, DATA>::dont_care();
    // Connect the reset conditioner
    d.resetn_conditioner.reset_n = i.reset_n;
    d.resetn_conditioner.clock = i.clock;
    // Connect the register
    d.register.clock_reset = signal(ClockReset {
        clock: i.clock.val(),
        reset: q.resetn_conditioner.val(),
    });
    // Connect the register's axi bus inputs to the fixture's axi bus input
    d.register.input =
        signal(crate::axi4lite::register::single::I::<DATA, ADDR> { axi: i.axi.val() });
    // Connect the axi bus output signals
    o.axi = signal(q.register.val().axi);
    o.read_data = signal(q.register.val().read_data);
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::axi4lite::types::{ReadMOSI, WriteMOSI};

    use super::*;
    use rhdl::{
        core::hdl::ast::{component_instance, connection, declaration},
        prelude::*,
    };

    fn axi_null_cmd() -> MOSI {
        MOSI {
            write: WriteMOSI {
                awaddr: bits(0),
                awvalid: false,
                wdata: bits(0),
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

    fn test_stream() -> impl Iterator<Item = TimedSample<I<Red, Blue, 32, 32>>> {
        let red = (0..).stream_after_reset(1).clock_pos_edge(100);
        let blue = axi_test_seq().stream().clock_pos_edge(79);
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
        let vcd = uut.run(stream)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("register");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["deba13492917f79b9a98b74cba94bee46a8aabcee29a81e5b6b54ca452bdd986"];
        let digest = vcd
            .dump_to_file(&root.join("axi4lite_register.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_export() -> miette::Result<()> {
        let uut = U::<Red, Blue>::default();
        let hdl = uut.hdl("axi_register")?;
        let verilog = hdl.as_module();
        let i = I::<Red, Green, 32, 32>::dont_care();
        let o = O::<Green, 8, 32>::dont_care();
        let binds = export![
            input aclk => i.clock,                              // Master AXI4-Lite clock
            input aresetn => i.reset_n,                         // Master AXI4-Lite reset
            input s_axi_awaddr => i.axi.val().write.awaddr,     // AXI4-Lite slave: Write address
            input s_axi_awvalid => i.axi.val().write.awvalid,   // AXI4-Lite slave: Write address valid
            output s_axi_awready => o.axi.val().write.awready,  // AXI4-Lite slave: Write address ready
            input s_axi_wdata => i.axi.val().write.wdata,       // AXI4-Lite slave: Write data
            input s_axi_wvalid => i.axi.val().write.wvalid,     // AXI4-Lite slave: Write data valid
            output s_axi_wready => o.axi.val().write.wready,    // AXI4-Lite slave: Write data ready
            output s_axi_bresp => o.axi.val().write.bresp,      // AXI4-Lite slave: Write response
            output s_axi_bvalid => o.axi.val().write.bvalid,    // AXI4-Lite slave: Write response valid
            input s_axi_bready => i.axi.val().write.bready,     // AXI4-Lite slave: Write response ready
            input s_axi_araddr => i.axi.val().read.araddr,      // AXI4-Lite slave: Read address
            input s_axi_arvalid => i.axi.val().read.arvalid,    // AXI4-Lite slave: Read address valid
            output s_axi_arready => o.axi.val().read.arready,   // AXI4-Lite slave: Read address ready
            output s_axi_rdata => o.axi.val().read.rdata,       // AXI4-Lite slave: Read data
            output s_axi_rresp => o.axi.val().read.rresp,       // AXI4-Lite slave: Read data response
            output s_axi_rvalid => o.axi.val().read.rvalid,     // AXI4-Lite slave: Read data valid
            input s_axi_rready => i.axi.val().read.rready,       // AXI4-Lite slave: Read data ready
            output data => o.read_data                          // Register read data
        ];
        let ports = binds
            .iter()
            .map(|(dir, name, kind, path)| {
                let (range, _) = bit_range(*kind, path).unwrap();
                let width = unsigned_width(range.end - range.start);
                port(name, *dir, HDLKind::Wire, width)
            })
            .collect::<Vec<_>>();
        let mut i_cover = vec![false; i.kind().bits()];
        let mut o_cover = vec![false; o.kind().bits()];
        binds.iter().for_each(|(dir, _, kind, path)| {
            let (range, _) = bit_range(*kind, path).unwrap();
            match dir {
                Direction::Input => {
                    for bit in range {
                        i_cover[bit] = true;
                    }
                }
                Direction::Output => {
                    for bit in range {
                        o_cover[bit] = true;
                    }
                }
                Direction::Inout => todo!(),
            }
        });
        if i_cover.iter().any(|b| !b) {
            panic!("Uncovered input bits: {:?}", i_cover);
        }
        if o_cover.iter().any(|b| !b) {
            panic!("Uncovered output bits: {:?}", o_cover);
        }
        let declarations = vec![
            declaration(HDLKind::Wire, "i", unsigned_width(i.kind().bits()), None),
            declaration(HDLKind::Wire, "o", unsigned_width(o.kind().bits()), None),
        ];
        let mut statements = binds
            .iter()
            .map(|(dir, name, kind, path)| {
                let (range, _) = bit_range(*kind, path).unwrap();
                match dir {
                    Direction::Input => rhdl::core::hdl::ast::Statement::Custom(format!(
                        "assign i[{}:{}] = {name};",
                        range.end.saturating_sub(1),
                        range.start
                    )),
                    Direction::Output => rhdl::core::hdl::ast::Statement::Custom(format!(
                        "assign {name} = o[{}:{}];",
                        range.end.saturating_sub(1),
                        range.start
                    )),
                    Direction::Inout => todo!(),
                }
            })
            .collect::<Vec<_>>();
        statements.push(component_instance(
            &verilog.name,
            "sub",
            vec![connection("i", id("i")), connection("o", id("o"))],
        ));
        let wrapped = Module {
            name: "axi_register_wrap".to_string(),
            description: "AXI4-Lite register with a single register".to_string(),
            ports,
            declarations,
            statements,
            submodules: vec![verilog],
            ..Default::default()
        };
        std::fs::write("axi_register.v", wrapped.to_string()).unwrap();
        Ok(())
    }
}
