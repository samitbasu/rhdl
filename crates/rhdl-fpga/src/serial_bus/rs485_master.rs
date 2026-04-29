//! RS-485 master / multidrop wrapper around the UART
//!
//! TIA/EIA-485-A is a differential serial bus used everywhere
//! industrial — Modbus RTU, BACnet MS/TP, DMX512, Profibus DP, hardware
//! variants of MIDI.  At the bit level it's just UART framing on a
//! differential pair; the additional protocol layer is the turnaround
//! handshake managing the DE (driver enable) and RE (receiver enable)
//! pins of the external transceiver chip (Maxim MAX485, TI SN65HVD7x,
//! Linear LTC2862).
//!
//! This widget composes the full-duplex [super::uart::Uart] with a
//! tiny FSM that drives DE (and the inverse, RE) so the host can do
//! request/response on a half-duplex bus without race conditions.
//!
//! **v1 scope:**
//! - Half-duplex (typical RS-485 usage).
//! - DE asserted while any TX byte is in flight, plus a configurable
//!   `t_de_holdoff` after the last byte's stop bit so the line settles
//!   before the master's transceiver releases.
//! - RE de-asserted while DE is asserted (a transceiver can't transmit
//!   and receive simultaneously through the same wire).  Some
//!   designs tie RE to ground; this widget surfaces it as a separate
//!   active-low output for board flexibility.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+Rs485Master+-------+
     |                           |
B<8> |                           | bool
+--->| tx_data              tx   +--->
bool |                           | bool
+--->| tx_push          de       +--->
bool |                           | bool
+--->| rx_pop           re_n     +--->
bool |                           | Option<B<8>>
+--->| rx               rx_data  +--->
     |                  busy     +--->
     |                  tx_full  +--->
     +---------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/rs485_master.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/rs485_master.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/rs485_master_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use super::uart::Uart;
use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the RS-485 turnaround FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition { source_index: 0, target_index: 1 }, // Idle → Driving (TX byte queued)
    Transition { source_index: 1, target_index: 1 }, // Driving self-loop (more bytes)
    Transition { source_index: 1, target_index: 2 }, // Driving → HoldOff (TX FIFO drained)
    Transition { source_index: 2, target_index: 0 }, // HoldOff → Idle (t_de_holdoff elapsed)
];

/// Turnaround state of the RS-485 transceiver enable line.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum Rs485State {
    /// No active transmission; DE de-asserted, RE asserted.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// TX byte(s) in flight; DE asserted, RE de-asserted.
    #[fsm_state(label = "driving")]
    Driving,
    /// All TX bytes flushed; holding DE asserted for `t_de_holdoff`
    /// to let the line settle before releasing the transceiver.
    #[fsm_state(label = "hold-off")]
    HoldOff,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = Rs485State)]
/// RS-485 master with DE/RE turnaround.
pub struct Rs485Master<const DIV_W: usize, const FIFO_W: usize, const HW: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
    rhdl::bits::W<HW>: BitWidth,
{
    uart: Uart<DIV_W, FIFO_W>,
    state: dff::DFF<Rs485State>,
    /// Tick counter inside HoldOff.
    tick: dff::DFF<Bits<HW>>,
    /// Hold-off in FPGA cycles after the TX FIFO drains.  Typical
    /// real value is 1–2 bit-times so the receiver is ready by the
    /// time the slave answers; e.g. at 1 MHz / 100 MHz host clock,
    /// 1 bit-time = 100 cycles.
    t_de_holdoff: Constant<Bits<HW>>,
}

impl<const DIV_W: usize, const FIFO_W: usize, const HW: usize>
    Rs485Master<DIV_W, FIFO_W, HW>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
    rhdl::bits::W<HW>: BitWidth,
{
    /// Create an RS-485 master with the given UART divisor and DE
    /// hold-off duration.
    pub fn new(divisor: Bits<DIV_W>, t_de_holdoff: Bits<HW>) -> Self {
        Self {
            uart: Uart::new(divisor),
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            t_de_holdoff: Constant::new(t_de_holdoff),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Rs485Master].
pub struct In {
    pub tx_data: Bits<8>,
    pub tx_push: bool,
    pub rx_pop: bool,
    pub rx: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Rs485Master].
pub struct Out {
    pub tx: bool,
    /// DE — driver enable to the external transceiver.  Active high.
    pub de: bool,
    /// RE — receiver enable, active low.  De-asserted (high) while
    /// DE is asserted; otherwise asserted (low).
    pub re_n: bool,
    pub rx_data: Option<Bits<8>>,
    pub busy: bool,
    pub tx_full: bool,
}

impl<const DIV_W: usize, const FIFO_W: usize, const HW: usize> SynchronousIO
    for Rs485Master<DIV_W, FIFO_W, HW>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
    rhdl::bits::W<HW>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = rs485_master<DIV_W, FIFO_W, HW>;
}

#[kernel]
/// Kernel for [Rs485Master].
pub fn rs485_master<const DIV_W: usize, const FIFO_W: usize, const HW: usize>(
    cr: ClockReset,
    i: In,
    q: Q<DIV_W, FIFO_W, HW>,
) -> (Out, D<DIV_W, FIFO_W, HW>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
    rhdl::bits::W<HW>: BitWidth,
{
    let one_h: Bits<HW> = bits::<HW>(1);
    let zero_h: Bits<HW> = bits::<HW>(0);

    let mut d = D::<DIV_W, FIFO_W, HW>::dont_care();
    d.uart = super::uart::In {
        tx_data: i.tx_data,
        tx_push: i.tx_push,
        rx_pop: i.rx_pop,
        rx: i.rx,
    };
    d.state = q.state;
    d.tick = q.tick + one_h;

    // Driver-enable FSM.  Idle → Driving when host pushes a byte
    // OR when the underlying UART has anything to send (so the
    // FIFO can't drain "secretly").  Stays Driving while the FIFO
    // has bytes; drops to HoldOff when the FIFO is empty AND the
    // UART core's TX is idle.
    let tx_pending = i.tx_push || q.uart.tx_full;

    match q.state {
        Rs485State::Idle => {
            d.tick = zero_h;
            if tx_pending {
                d.state = Rs485State::Driving;
                d.tick = zero_h;
            }
        }
        Rs485State::Driving => {
            // Reset hold-off whenever a byte is queued or in flight.
            if !tx_pending {
                d.state = Rs485State::HoldOff;
                d.tick = zero_h;
            }
        }
        Rs485State::HoldOff => {
            if q.tick == q.t_de_holdoff {
                d.state = Rs485State::Idle;
                d.tick = zero_h;
            }
        }
    }

    if cr.reset.any() {
        d.state = Rs485State::Idle;
        d.tick = zero_h;
    }

    let de = match q.state {
        Rs485State::Idle => false,
        Rs485State::Driving | Rs485State::HoldOff => true,
    };

    let mut o = Out::dont_care();
    o.tx = q.uart.tx;
    o.de = de;
    o.re_n = de; // RE is the inverse of DE; active-low output
    o.rx_data = q.uart.rx_data;
    o.busy = de;
    o.tx_full = q.uart.tx_full;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            tx_data: bits(0),
            tx_push: false,
            rx_pop: false,
            rx: true,
        }
    }

    #[test]
    fn test_idle_releases_driver() -> miette::Result<()> {
        let uut = Rs485Master::<6, 4, 8>::new(bits(6), bits(20));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_de = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.de);
        assert!(!any_de, "DE must stay de-asserted while idle");
        Ok(())
    }

    #[test]
    fn test_de_asserts_during_tx() -> miette::Result<()> {
        let divisor = 6;
        let uut = Rs485Master::<6, 4, 8>::new(bits(divisor as u128), bits(20));
        let mut stream_in: Vec<In> = vec![In {
            tx_data: bits(0xA5),
            tx_push: true,
            rx_pop: false,
            rx: true,
        }];
        for _ in 0..(15 * divisor + 50) {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // DE must assert at some point during the transmission.
        let any_de = outputs.iter().any(|s| s.output.de);
        assert!(any_de, "DE never asserted during TX");
        // DE must de-assert eventually.
        let post_de = outputs.iter().rev().take(8).all(|s| !s.output.de);
        assert!(post_de, "DE never de-asserted after hold-off");
        Ok(())
    }

    #[test]
    fn test_re_n_inverse_of_de() -> miette::Result<()> {
        let uut = Rs485Master::<6, 4, 8>::new(bits(6), bits(20));
        let mut stream_in: Vec<In> = vec![In {
            tx_data: bits(0x55),
            tx_push: true,
            rx_pop: false,
            rx: true,
        }];
        for _ in 0..200 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        for s in &outputs {
            assert_eq!(
                s.output.de, s.output.re_n,
                "DE / RE_n drift at cycle (de={}, re_n={})",
                s.output.de, s.output.re_n
            );
        }
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check.
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Rs485Master::<6, 4, 8>::new(bits(8), bits(20));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["61440"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip.
    #[test]
    fn test_rs485_master_hdl_works() -> miette::Result<()> {
        let uut = Rs485Master::<6, 4, 8>::new(bits(8), bits(20));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest.
    #[test]
    fn test_rs485_master_trace() -> miette::Result<()> {
        let uut = Rs485Master::<6, 4, 8>::new(bits(8), bits(20));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("rs485_master");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["56ac1140c83dfc85aad53bc0b3f992d44d91c81c7861ed18944b020525f02e2e"];
        let digest = vcd.dump_to_file(root.join("rs485_master.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
