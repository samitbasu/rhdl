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
pub struct I<W: Domain, R: Domain, const DATA: usize = 32, const ADDR: usize = 32> {
    pub reset_n: Signal<ResetN, W>,
    pub clock: Signal<Clock, R>,
    pub axi: Signal<MOSI<DATA, ADDR>, R>,
}

#[derive(Digital, Timed)]
pub struct O<R: Domain, const REG_WIDTH: usize = 32, const DATA: usize = 32> {
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

    fn test_stream() -> impl Iterator<Item = TimedSample<I<Red, Blue>>> {
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
}
