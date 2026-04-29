//! PS/2 keyboard receiver (host side, receive-only v1)
//!
//! The PS/2 keyboard interface — IBM's 1987 PC-AT serial protocol
//! every PC keyboard used until USB took over in the mid-2000s,
//! and that USB keyboards still emulate via the BIOS legacy mode.
//! Two open-drain wires (`CLK` and `DATA`), each idling high
//! through a 10 kΩ pull-up.  The keyboard is the *clock master* —
//! it drives `CLK` at 10–16.7 kHz while shifting data MSB-first
//! into the data line.
//!
//! Each transaction is an 11-bit frame:
//!
//! ```text
//!   start (0) | data[0] | data[1] | ... | data[7] | parity | stop (1)
//! ```
//!
//! Data bits are LSB-first; `parity` is *odd parity over the 8
//! data bits* (i.e., `parity = !XOR(data[0..8])` so the total
//! number of `1`s in the 9-bit `data ∥ parity` field is odd).  The
//! host samples `DATA` on the **falling edge** of `CLK`.
//!
//! **v1 scope:** *receive-only*.  The widget detects falling edges
//! on `clk_in` (which the host has already passed through a
//! synchronizer chain to the FPGA clock domain), shifts `data_in`
//! into an 11-bit register, validates start / stop / parity, and
//! emits `(scan_code, valid)` on each correctly-framed byte.  A
//! framing or parity error pulses `frame_err` instead of `valid`.
//! The host-to-keyboard transmit path (set LEDs, set typematic
//! rate) is symmetric but defers to v2.
//!
//! Composes [super::super::core::dff::DFF] for the shift register
//! and bit counter; the host wraps `clk_in` and `data_in` with
//! [super::super::cdc::synchronizer_chain::BitSyncChain] before
//! feeding them in.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+Ps2Keyboard+--------+
     |                            |
bool |                            | B<8>
+--->| clk_in           scan_code +--->
bool |                            | bool
+--->| data_in              valid +--->
     |                  frame_err +--->
     |                       busy +--->
     +----------------------------+
")]
//!
//!# Internals
//!
//! The 11-bit shift register accumulates LSB-first as bits come
//! in (each new bit goes into the MSB; after 11 shifts, the start
//! bit ends up at position 0).  After the 11th falling edge the
//! widget validates:
//!
//! - bit 0 (start) = `0`,
//! - bit 10 (stop) = `1`,
//! - bit 9 (parity) = `!XOR(bits 1..9)`.
//!
//! All three pass → `valid` pulses with `scan_code = bits 1..9`.
//! Any fail → `frame_err` pulses, no `scan_code` update.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/ps2_keyboard.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/ps2_keyboard.md")]
//!
//! And the auto-generated FSM diagram for the bit-walker:
#![doc = include_str!("../../doc/ps2_keyboard_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::dff;

/// Author-curated transition graph for the PS/2 receiver FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    },
    Transition {
        source_index: 1,
        target_index: 1,
    },
    Transition {
        source_index: 1,
        target_index: 0,
    },
];

/// Receive-side state machine.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum Ps2State {
    /// Bus idle — waiting for the first falling clock edge.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Shifting bits in on each falling edge.
    #[fsm_state(label = "shift")]
    Shift,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = Ps2State)]
/// PS/2 keyboard receiver.
pub struct Ps2Keyboard {
    state: dff::DFF<Ps2State>,
    /// 11-bit shift register (start + data + parity + stop accumulate here).
    shift: dff::DFF<Bits<11>>,
    /// Number of bits shifted in so far (0..11).
    bit_idx: dff::DFF<Bits<4>>,
    /// Last-cycle value of `clk_in` for falling-edge detection.
    clk_prev: dff::DFF<bool>,
    /// Latched scan code on success.
    scan_code: dff::DFF<Bits<8>>,
    /// One-cycle pulses.
    valid_pulse: dff::DFF<bool>,
    err_pulse: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Ps2Keyboard].  Both signals must already be in the FPGA's
/// clock domain (wrap raw pads with `cdc::synchronizer_chain` first).
pub struct In {
    pub clk_in: bool,
    pub data_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Ps2Keyboard].
pub struct Out {
    /// Latched scan code from the most-recent successfully-received frame.
    pub scan_code: Bits<8>,
    /// Pulses for one cycle when `scan_code` is freshly captured.
    pub valid: bool,
    /// Pulses for one cycle on a framing or parity error.
    pub frame_err: bool,
    /// True while bits are accumulating (i.e., `state == Shift`).
    pub busy: bool,
}

impl SynchronousIO for Ps2Keyboard {
    type I = In;
    type O = Out;
    type Kernel = ps2_keyboard;
}

#[kernel]
/// Kernel for [Ps2Keyboard].
pub fn ps2_keyboard(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.shift = q.shift;
    d.bit_idx = q.bit_idx;
    d.clk_prev = i.clk_in;
    d.scan_code = q.scan_code;
    d.valid_pulse = false;
    d.err_pulse = false;

    // Falling edge on CLK = sample DATA.
    let falling = q.clk_prev && !i.clk_in;

    match q.state {
        Ps2State::Idle => {
            if falling {
                // First bit (start, expected 0).  Shift it into the LSB.
                let new_bit: Bits<11> = if i.data_in { bits::<11>(1) } else { bits::<11>(0) };
                d.shift = new_bit;
                d.bit_idx = bits::<4>(1);
                d.state = Ps2State::Shift;
            }
        }
        Ps2State::Shift => {
            if falling {
                // Shift the new bit into bit position bit_idx.  bit_idx is
                // Bits<4>; `<<` on Bits accepts a Bits-typed shift amount.
                let mask: Bits<11> = if i.data_in {
                    bits::<11>(1) << q.bit_idx
                } else {
                    bits::<11>(0)
                };
                let new_shift = q.shift | mask;
                d.shift = new_shift;
                if q.bit_idx == bits::<4>(10) {
                    // 11 bits accumulated — validate frame.
                    let start_bit = (new_shift & bits::<11>(0x001)) != bits::<11>(0);
                    let parity_bit = (new_shift & bits::<11>(0x200)) != bits::<11>(0);
                    let stop_bit = (new_shift & bits::<11>(0x400)) != bits::<11>(0);
                    // Data byte = bits 1..9, shifted down by 1.  Resize down
                    // to Bits<8> (the upper three bits of new_shift>>1 are
                    // parity + stop + 0; the resize discards them).
                    let data_byte: Bits<8> = (new_shift >> bits::<11>(1)).resize();
                    // popcount of data byte (8 single-bit additions).
                    let mut ones: Bits<4> = bits::<4>(0);
                    for k in 0..8 {
                        let bit = (data_byte >> (k as u128)) & bits::<8>(1);
                        if bit != bits::<8>(0) {
                            ones += bits::<4>(1);
                        }
                    }
                    // Odd parity: total of (data ones + parity_bit) must be odd.
                    let parity_one: Bits<4> = if parity_bit { bits::<4>(1) } else { bits::<4>(0) };
                    let total = ones + parity_one;
                    let total_is_odd = (total & bits::<4>(1)) != bits::<4>(0);
                    let frame_ok = !start_bit && stop_bit && total_is_odd;
                    if frame_ok {
                        d.scan_code = data_byte;
                        d.valid_pulse = true;
                    } else {
                        d.err_pulse = true;
                    }
                    d.state = Ps2State::Idle;
                    d.bit_idx = bits::<4>(0);
                    d.shift = bits::<11>(0);
                } else {
                    d.bit_idx = q.bit_idx + bits::<4>(1);
                }
            }
        }
    }

    if cr.reset.any() {
        d.state = Ps2State::Idle;
        d.shift = bits::<11>(0);
        d.bit_idx = bits::<4>(0);
        d.clk_prev = true;
        d.scan_code = bits::<8>(0);
        d.valid_pulse = false;
        d.err_pulse = false;
    }

    let mut o = Out::dont_care();
    o.scan_code = q.scan_code;
    o.valid = q.valid_pulse;
    o.frame_err = q.err_pulse;
    o.busy = q.state != Ps2State::Idle;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            clk_in: true,
            data_in: true,
        }
    }

    /// Build a stream that drives clk + data for one PS/2 frame carrying
    /// `data` (LSB-first), with correct odd parity.  Each "cell" is two
    /// host cycles (clk high + clk low) to give the falling edge.
    fn drive_frame(data: u8) -> Vec<In> {
        // Compute odd parity bit.
        let ones = data.count_ones();
        let parity = (ones % 2) == 0; // odd parity → parity = 1 if data has even ones
        // Frame bits, in transmission order: start(0), data LSB→MSB, parity, stop(1).
        let mut frame_bits: Vec<bool> = Vec::with_capacity(11);
        frame_bits.push(false); // start
        for i in 0..8 {
            frame_bits.push((data >> i) & 1 == 1);
        }
        frame_bits.push(parity);
        frame_bits.push(true); // stop

        let mut out = Vec::new();
        // Idle leading.
        for _ in 0..2 {
            out.push(idle_in());
        }
        // For each frame bit: hold clk high (data set), then clk low.
        for &b in &frame_bits {
            // Setup with clk high + data driven.
            for _ in 0..3 {
                out.push(In {
                    clk_in: true,
                    data_in: b,
                });
            }
            // Falling edge: clk goes low.
            for _ in 0..3 {
                out.push(In {
                    clk_in: false,
                    data_in: b,
                });
            }
        }
        // Trailing idle.
        for _ in 0..6 {
            out.push(idle_in());
        }
        out
    }

    #[test]
    fn test_idle_no_valid() -> miette::Result<()> {
        let uut = Ps2Keyboard::default();
        let stream = std::iter::repeat_n(idle_in(), 64)
            .with_reset(2)
            .clock_pos_edge(100);
        let any_valid = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.valid || s.output.frame_err);
        assert!(!any_valid);
        Ok(())
    }

    #[test]
    fn test_recv_byte_0x55() -> miette::Result<()> {
        let uut = Ps2Keyboard::default();
        let stream_in = drive_frame(0x55);
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let valid_idx = outputs.iter().position(|s| s.output.valid);
        assert!(valid_idx.is_some(), "no valid pulse for a well-formed frame");
        assert_eq!(outputs[valid_idx.unwrap()].output.scan_code.raw(), 0x55);
        Ok(())
    }

    #[test]
    fn test_recv_byte_0x1c() -> miette::Result<()> {
        // 0x1C = scan code for 'A' on Set 2 — a real-world byte.
        let uut = Ps2Keyboard::default();
        let stream_in = drive_frame(0x1C);
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let valid_idx = outputs.iter().position(|s| s.output.valid);
        assert!(valid_idx.is_some());
        assert_eq!(outputs[valid_idx.unwrap()].output.scan_code.raw(), 0x1C);
        Ok(())
    }

    #[test]
    fn test_bad_parity_is_frame_err() -> miette::Result<()> {
        // Build a frame for 0x55 then corrupt the parity bit before sending.
        let uut = Ps2Keyboard::default();
        let mut stream_in = drive_frame(0x55);
        // Find the parity bit (the 10th bit cell, i.e. cells start, d0..d7, parity, stop)
        // — index 9 (zero-based) in the bit list. Each cell is 6 cycles (3 high + 3 low).
        // Idle prefix is 2 cycles. Cell index 9 starts at: 2 + 9*6 = 56.
        // Flip data within that cell.
        let cell_start = 2 + 9 * 6;
        for k in 0..6 {
            let idx = cell_start + k;
            stream_in[idx].data_in = !stream_in[idx].data_in;
        }
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let any_valid = outputs.iter().any(|s| s.output.valid);
        let any_err = outputs.iter().any(|s| s.output.frame_err);
        assert!(!any_valid, "frame with bad parity must NOT pulse valid");
        assert!(any_err, "frame with bad parity must pulse frame_err");
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Ps2Keyboard::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["13844"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_ps2_keyboard_hdl_works() -> miette::Result<()> {
        let uut = Ps2Keyboard::default();
        let stream_in = drive_frame(0x55);
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ps2_keyboard_trace() -> miette::Result<()> {
        let uut = Ps2Keyboard::default();
        let stream_in = drive_frame(0x1C);
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ps2_keyboard");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["07e288de98676b5a6a7f42042b0d8b6202eb56421a68573317dd7ee939d49142"];
        let digest = vcd
            .dump_to_file(root.join("ps2_keyboard.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = Ps2Keyboard::fsm_descriptor();
        assert_eq!(desc.widget_name, "Ps2Keyboard");
        assert_eq!(desc.variants().len(), 2);
        assert_eq!(desc.initial_index(), 0);
    }
}
