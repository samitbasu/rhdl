//! WS2812 / NeoPixel driver
//!
//! Single-wire, daisy-chained RGB LED protocol used by WS2812B,
//! SK6812, WS2811, and friends.  Each LED consumes 24 bits (8 each
//! for the three color channels) and forwards subsequent bits down
//! the chain.  After every chain transmission, an idle low period
//! >50 µs latches the new colors.
//!
//! Bit encoding: each bit takes a fixed period (≈1.25 µs).  The
//! line is held high at the start, then driven low for the
//! remainder.  A short high time (~400 ns) encodes `0`; a long
//! high time (~800 ns) encodes `1`.
//!
//! This v1 sends a **single 24-bit pixel** per `send` strobe.  For
//! multi-LED chains, the host strobes `send` once per pixel in
//! tight succession, then asserts `latch` once at the end of the
//! frame to drive the inter-frame idle.  A future variant will
//! wrap a `fifo::synchronous` for hands-off streaming.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+Ws2812Driver+-----+
     |                        |
B<24>|                        | bool
+--->| pixel        data_out  +--->
bool |                        | bool
+--->| send             busy  +--->
bool |                        | bool
+--->| latch            done  +--->
     +------------------------+
")]
//!
//!# Internals
//!
//! Per-bit timing comes from three runtime constants:
//!
//! - `t0_high`: cycles the line is high for a `0` bit.
//! - `t1_high`: cycles the line is high for a `1` bit.
//! - `bit_period`: total cycles per bit.
//!
//! A separate `latch_period` controls the inter-frame idle low
//! pulse.  The host triggers it with the `latch` input.
//!
//! State machine: `Idle → Sending → Latching → Idle`.
//!
//!# Behavior
//!
//! - `send` (asserted while `Idle`): latch `pixel` and begin
//!   transmitting 24 bits MSB-first.  `busy` rises.
//! - During transmit: `data_out` walks the bit pattern.  When all
//!   24 bits are out, the driver returns to `Idle` (`busy` drops).
//!   The host can immediately strobe `send` again for the next
//!   pixel.
//! - `latch` (asserted while `Idle`): drives `data_out` low for
//!   `latch_period` cycles, then returns to `Idle` and pulses `done`.
//!
//!# Parameters
//!
//! - `CW` — bit width of the cycle counter.  Must hold the largest
//!   of `t0_high`, `t1_high`, `bit_period`, `latch_period`.  For
//!   100 MHz / WS2812B (`bit_period = 125`, `latch_period > 5000`),
//!   use `CW = 13`.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/ws2812.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/ws2812.md")]
//!
//! And the auto-generated FSM diagram for the per-pixel transmit walk:
#![doc = include_str!("../../doc/ws2812_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the WS2812 driver FSM.
///
/// Required by CLAUDE.md §12 rule 14.  Indices match `WsState`
/// declaration order (Idle=0, Sending=1, Latching=2).
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → Sending (on `send`)
    Transition {
        source_index: 1,
        target_index: 1,
    }, // Sending self-loop (per cycle / per bit)
    Transition {
        source_index: 1,
        target_index: 2,
    }, // Sending → Latching (after 24 bits)
    Transition {
        source_index: 2,
        target_index: 2,
    }, // Latching self-loop (per latch cycle)
    Transition {
        source_index: 2,
        target_index: 0,
    }, // Latching → Idle (latch period elapsed)
];

/// Driver state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum WsState {
    /// No active transmission; output line held low.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Clocking 24 bits out at the WS2812 timing format.
    #[fsm_state(label = "sending")]
    Sending,
    /// Driving the line low for the latch / reset period.
    #[fsm_state(label = "latching")]
    Latching,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = WsState)]
/// WS2812 single-pixel driver.
pub struct Ws2812Driver<const CW: usize>
where
    rhdl::bits::W<CW>: BitWidth,
{
    state: dff::DFF<WsState>,
    cycle_counter: dff::DFF<Bits<CW>>,
    bit_idx: dff::DFF<Bits<5>>,
    pixel_reg: dff::DFF<Bits<24>>,
    done_pulse: dff::DFF<bool>,
    t0_high: Constant<Bits<CW>>,
    t1_high: Constant<Bits<CW>>,
    bit_period: Constant<Bits<CW>>,
    latch_period: Constant<Bits<CW>>,
}

impl<const CW: usize> Ws2812Driver<CW>
where
    rhdl::bits::W<CW>: BitWidth,
{
    /// Create a driver with the given timings, all in FPGA clock cycles.
    pub fn new(
        t0_high: Bits<CW>,
        t1_high: Bits<CW>,
        bit_period: Bits<CW>,
        latch_period: Bits<CW>,
    ) -> Self {
        Self {
            state: dff::DFF::default(),
            cycle_counter: dff::DFF::default(),
            bit_idx: dff::DFF::default(),
            pixel_reg: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            t0_high: Constant::new(t0_high),
            t1_high: Constant::new(t1_high),
            bit_period: Constant::new(bit_period),
            latch_period: Constant::new(latch_period),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Ws2812Driver].
pub struct In {
    /// 24-bit pixel value.  Bit ordering is whatever the LED
    /// expects (typically GRB MSB-first); the widget just sends
    /// bit 23 first.
    pub pixel: Bits<24>,
    /// Strobe to begin transmitting `pixel`.  Ignored unless `Idle`.
    pub send: bool,
    /// Strobe to begin a latch idle.  Ignored unless `Idle`.
    pub latch: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Ws2812Driver].
pub struct Out {
    /// The WS2812 data line.
    pub data_out: bool,
    /// High while sending or latching.
    pub busy: bool,
    /// Pulses high for one cycle at end of latch.
    pub done: bool,
}

impl<const CW: usize> SynchronousIO for Ws2812Driver<CW>
where
    rhdl::bits::W<CW>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = ws2812<CW>;
}

#[kernel]
/// Kernel for [Ws2812Driver].
pub fn ws2812<const CW: usize>(cr: ClockReset, i: In, q: Q<CW>) -> (Out, D<CW>)
where
    rhdl::bits::W<CW>: BitWidth,
{
    let one_cw: Bits<CW> = bits::<CW>(1);
    let zero_cw: Bits<CW> = bits::<CW>(0);
    let one_b5: Bits<5> = bits::<5>(1);
    let zero_b5: Bits<5> = bits::<5>(0);
    let twenty_three_b5: Bits<5> = bits::<5>(23);
    let zero_b24: Bits<24> = bits::<24>(0);

    let mut d = D::<CW>::dont_care();
    d.state = q.state;
    d.cycle_counter = q.cycle_counter;
    d.bit_idx = q.bit_idx;
    d.pixel_reg = q.pixel_reg;
    d.done_pulse = false;

    // Current bit value (MSB of pixel_reg).
    let cur_bit_v = (q.pixel_reg >> bits::<24>(23)) & bits::<24>(1);
    let cur_bit = cur_bit_v != zero_b24;
    let high_time = if cur_bit { q.t1_high } else { q.t0_high };
    let in_high = q.cycle_counter < high_time;
    let bit_period_done = q.cycle_counter == (q.bit_period - one_cw);
    let latch_done = q.cycle_counter == (q.latch_period - one_cw);

    let data_out = match q.state {
        WsState::Idle | WsState::Latching => false,
        WsState::Sending => in_high,
    };

    match q.state {
        WsState::Idle => {
            if i.send {
                d.state = WsState::Sending;
                d.pixel_reg = i.pixel;
                d.cycle_counter = zero_cw;
                d.bit_idx = zero_b5;
            } else if i.latch {
                d.state = WsState::Latching;
                d.cycle_counter = zero_cw;
            }
        }
        WsState::Sending => {
            if bit_period_done {
                d.cycle_counter = zero_cw;
                if q.bit_idx == twenty_three_b5 {
                    // Last bit done — return to idle.
                    d.state = WsState::Idle;
                    d.bit_idx = zero_b5;
                } else {
                    d.bit_idx = q.bit_idx + one_b5;
                    // Shift pixel_reg left so next MSB is the next bit.
                    d.pixel_reg = q.pixel_reg << 1;
                }
            } else {
                d.cycle_counter = q.cycle_counter + one_cw;
            }
        }
        WsState::Latching => {
            if latch_done {
                d.cycle_counter = zero_cw;
                d.state = WsState::Idle;
                d.done_pulse = true;
            } else {
                d.cycle_counter = q.cycle_counter + one_cw;
            }
        }
    }

    if cr.reset.any() {
        d.state = WsState::Idle;
        d.cycle_counter = zero_cw;
        d.bit_idx = zero_b5;
        d.pixel_reg = zero_b24;
        d.done_pulse = false;
    }

    let busy = match q.state {
        WsState::Idle => false,
        WsState::Sending | WsState::Latching => true,
    };

    let mut o = Out::dont_care();
    o.data_out = data_out;
    o.busy = busy;
    o.done = q.done_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            pixel: bits(0),
            send: false,
            latch: false,
        }
    }

    /// Decode a recorded `data_out` sequence into the bits sent.
    /// Each bit takes `bit_period` cycles; the bit value = (high_count > t0_high).
    fn decode_bits(line: &[bool], bit_period: usize, threshold: usize, n_bits: usize) -> Vec<bool> {
        let mut out = Vec::new();
        for k in 0..n_bits {
            let start = k * bit_period;
            let end = start + bit_period;
            if end > line.len() {
                break;
            }
            let high_count = line[start..end].iter().filter(|&&b| b).count();
            out.push(high_count > threshold);
        }
        out
    }

    // Tier 2 — send a pixel and verify the encoded line matches.
    #[test]
    fn test_send_pixel_round_trip() -> miette::Result<()> {
        // Compact test: t0_high=2, t1_high=4, bit_period=8, latch_period=16.
        let pixel: u128 = 0xAABBCC;
        let uut = Ws2812Driver::<6>::new(bits(2), bits(4), bits(8), bits(16));
        let n_cycles = 24 * 8 + 4;
        let mut stream_in: Vec<In> = vec![In {
            pixel: bits(pixel),
            send: true,
            latch: false,
        }];
        for _ in 0..n_cycles {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<bool> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .map(|s| s.output.data_out)
            .collect();
        // The first cycle is idle (send=true latched but state still Idle),
        // so transmission starts at cycle 1.  Skip 1 cycle.
        let line = &outputs[1..];
        let decoded = decode_bits(line, 8, 3, 24);
        // Expected: bit 23 first (MSB), so 0xAABBCC = 0b1010_1010_1011_1011_1100_1100
        let expected: Vec<bool> = (0..24).rev().map(|k| ((pixel >> k) & 1) != 0).collect();
        assert_eq!(decoded, expected, "decoded line {decoded:?}");
        Ok(())
    }

    #[test]
    fn test_idle_line_is_low() -> miette::Result<()> {
        let uut = Ws2812Driver::<6>::new(bits(2), bits(4), bits(8), bits(16));
        let stream = std::iter::repeat_n(idle_in(), 16)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_high = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.data_out);
        assert!(!any_high);
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Ws2812Driver::<6>::new(bits(2), bits(4), bits(8), bits(16));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["10515"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_ws2812_hdl_works() -> miette::Result<()> {
        let uut = Ws2812Driver::<6>::new(bits(2), bits(4), bits(8), bits(16));
        let mut stream_in: Vec<In> = vec![In {
            pixel: bits(0xA5_5A_3C),
            send: true,
            latch: false,
        }];
        for _ in 0..220 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest
    #[test]
    fn test_ws2812_trace() -> miette::Result<()> {
        let uut = Ws2812Driver::<6>::new(bits(2), bits(4), bits(8), bits(16));
        let mut stream_in: Vec<In> = vec![In {
            pixel: bits(0xA5_5A_3C),
            send: true,
            latch: false,
        }];
        for _ in 0..220 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ws2812");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0021487b53cd6a38a6d36adc0ba5e12bacec72bae95b5cb44dafacc5552a88c9"];
        let digest = vcd.dump_to_file(root.join("ws2812.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
