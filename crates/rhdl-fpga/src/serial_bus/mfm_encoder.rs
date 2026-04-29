//! MFM (Modified Frequency Modulation) encoder
//!
//! MFM is the line-level encoding that virtually every floppy
//! controller (NEC µPD765 / Intel 8272 / WD1772 / Western Digital
//! WD1003 hard-disk controller) and the early PC ATA/IDE drives
//! used to lay bits on rotating media.  It encodes one *data bit*
//! into two *cells*; the cell pattern is chosen so that every cell
//! window contains at least one transition (which the read-side PLL
//! locks to) and the data is unambiguously recoverable.
//!
//! The encoding rule for the two cells `(C, D)` of a data bit `d`:
//!
//! - `D = d` — the data cell carries the data bit unchanged.
//! - `C = (NOT d) AND (NOT prev_d)` — the clock cell is `1` only
//!   when both the current and previous data bits are `0`.
//!
//! This produces three patterns:
//!
//! | prev | cur | (C, D) | meaning            |
//! |------|-----|--------|--------------------|
//! |  *   |  1  | (0, 1) | "1 bit"            |
//! |  1   |  0  | (0, 0) | "0 after 1"        |
//! |  0   |  0  | (1, 0) | "0 after 0" — clk  |
//!
//! On the wire each cell is a polarity transition (NRZI-style):
//! a `1` cell flips the line, a `0` cell holds.  This widget
//! emits the *raw cell stream* as `cell_out` (one bit per output
//! cycle), with a `valid` strobe that goes high once per cell.
//! The host (or a downstream NRZI register) handles the polarity
//! flip.
//!
//! **v1 scope:** byte-at-a-time encoder.  The host strobes `send`
//! with an 8-bit byte; the widget shifts MSB-first, emits 16 cells
//! (8 clock + 8 data, interleaved C-D-C-D-...), and pulses `done`.
//! No address-mark generation (the special clock-rule-violating
//! sync bytes 0xA1 and 0xC2), no CRC, no FIFO buffering — those
//! belong in a higher-level "floppy track formatter" widget.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +---------+MfmEncoder+----------+
     |                               |
B<8> |                               | bool
+--->| data                cell_out  +--->
bool |                               | bool
+--->| send                cell_valid+--->
     |                          busy +--->
     |                          done +--->
     +-------------------------------+
")]
//!
//!# Internals
//!
//! Two internal registers: an 8-bit shift register holding the
//! remaining data bits, a 4-bit cell counter (0..16) for the
//! interleaved C/D phases.  A 1-bit `prev_data` register feeds the
//! clock-cell rule.  The state machine has three states: `Idle`,
//! `EmitClock`, `EmitData`.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/mfm_encoder.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/mfm_encoder.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/mfm_encoder_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::dff;

/// Author-curated transition graph for the MFM encoder FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    },
    Transition {
        source_index: 1,
        target_index: 2,
    },
    Transition {
        source_index: 2,
        target_index: 1,
    },
    Transition {
        source_index: 2,
        target_index: 0,
    },
];

/// Internal state machine.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum MfmState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// About to emit the clock cell of the current bit.
    #[fsm_state(label = "emit clock")]
    EmitClock,
    /// About to emit the data cell of the current bit.
    #[fsm_state(label = "emit data")]
    EmitData,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = MfmState)]
/// MFM encoder.
pub struct MfmEncoder {
    state: dff::DFF<MfmState>,
    /// Remaining data bits (MSB-first).
    shift: dff::DFF<Bits<8>>,
    /// Number of data bits emitted so far (0..8).
    bit_idx: dff::DFF<Bits<4>>,
    /// Previous data bit (for clock-cell rule).
    prev_data: dff::DFF<bool>,
    done_pulse: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [MfmEncoder].
pub struct In {
    /// Byte to encode.  Latched at `send`.
    pub data: Bits<8>,
    /// Strobe to begin a byte transfer.  Ignored while busy.
    pub send: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [MfmEncoder].
pub struct Out {
    /// The current cell (clock or data).  `1` ⇒ transition; `0` ⇒ hold.
    pub cell_out: bool,
    /// Pulses high for one cycle per emitted cell.
    pub cell_valid: bool,
    pub busy: bool,
    pub done: bool,
}

impl SynchronousIO for MfmEncoder {
    type I = In;
    type O = Out;
    type Kernel = mfm_encoder;
}

#[kernel]
/// Kernel for [MfmEncoder].
pub fn mfm_encoder(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.shift = q.shift;
    d.bit_idx = q.bit_idx;
    d.prev_data = q.prev_data;
    d.done_pulse = false;

    let cur_bit = (q.shift & bits::<8>(0x80)) != bits::<8>(0);

    // Default outputs (only meaningful when cell_valid is high).
    let mut cell_out = false;
    let mut cell_valid = false;

    match q.state {
        MfmState::Idle => {
            if i.send {
                d.shift = i.data;
                d.bit_idx = bits::<4>(0);
                d.state = MfmState::EmitClock;
                // prev_data starts at 0 — same convention as PC controllers
                // when the host strobes a fresh byte after idle.
                d.prev_data = false;
            }
        }
        MfmState::EmitClock => {
            // Clock cell: 1 if both prev and current data bits are 0.
            cell_out = !cur_bit && !q.prev_data;
            cell_valid = true;
            d.state = MfmState::EmitData;
        }
        MfmState::EmitData => {
            cell_out = cur_bit;
            cell_valid = true;
            // Advance: shift left, increment bit index, update prev_data.
            d.prev_data = cur_bit;
            d.shift = q.shift << bits::<8>(1);
            if q.bit_idx == bits::<4>(7) {
                // Last data bit emitted — done.
                d.state = MfmState::Idle;
                d.done_pulse = true;
                d.bit_idx = bits::<4>(0);
            } else {
                d.bit_idx = q.bit_idx + bits::<4>(1);
                d.state = MfmState::EmitClock;
            }
        }
    }

    if cr.reset.any() {
        d.state = MfmState::Idle;
        d.shift = bits::<8>(0);
        d.bit_idx = bits::<4>(0);
        d.prev_data = false;
        d.done_pulse = false;
    }

    let mut o = Out::dont_care();
    o.cell_out = cell_out;
    o.cell_valid = cell_valid;
    o.busy = q.state != MfmState::Idle;
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
            data: bits(0),
            send: false,
        }
    }

    /// Reference encoder in plain Rust — used to cross-check against the
    /// kernel.  Returns the 16-cell sequence (clock then data, repeated)
    /// for a byte, given the previous-data-bit start state.
    fn ref_encode(byte: u8, mut prev: bool) -> Vec<bool> {
        let mut cells = Vec::with_capacity(16);
        for i in 0..8 {
            let cur = (byte >> (7 - i)) & 1 == 1;
            let clk = !cur && !prev;
            cells.push(clk);
            cells.push(cur);
            prev = cur;
        }
        cells
    }

    #[test]
    fn test_idle_no_cells() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let stream = std::iter::repeat_n(idle_in(), 16)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_valid = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.cell_valid);
        assert!(!any_valid);
        Ok(())
    }

    #[test]
    fn test_byte_completes_with_done() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let valid_count = outputs.iter().filter(|s| s.output.cell_valid).count();
        assert_eq!(valid_count, 16, "must emit exactly 16 cells per byte");
        assert!(outputs.iter().any(|s| s.output.done));
        Ok(())
    }

    /// The byte that exercises every encoding rule:
    /// `0b10100101` — alternations between 0s and 1s force every clock-cell
    /// rule to be applied at least once.
    #[test]
    fn test_cell_pattern_matches_reference() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let cells: Vec<bool> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.cell_valid)
            .map(|s| s.output.cell_out)
            .collect();
        let expected = ref_encode(0xA5, false);
        assert_eq!(
            cells, expected,
            "cell pattern mismatch: got {cells:?}, expected {expected:?}"
        );
        Ok(())
    }

    #[test]
    fn test_all_zeros_byte() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0x00),
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let cells: Vec<bool> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.cell_valid)
            .map(|s| s.output.cell_out)
            .collect();
        // 0x00 with prev=0: every (C, D) is (1, 0) — clock cells all on,
        // data cells all off.  Pattern: 1010 1010 1010 1010.
        let expected = vec![
            true, false, true, false, true, false, true, false, true, false, true, false, true,
            false, true, false,
        ];
        assert_eq!(cells, expected);
        Ok(())
    }

    #[test]
    fn test_all_ones_byte() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xFF),
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let cells: Vec<bool> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.cell_valid)
            .map(|s| s.output.cell_out)
            .collect();
        // 0xFF: every cur bit is 1 → clock cells are all 0, data cells all 1.
        // Pattern: 0101 0101 0101 0101.
        let expected = vec![
            false, true, false, true, false, true, false, true, false, true, false, true, false,
            true, false, true,
        ];
        assert_eq!(cells, expected);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8233"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_mfm_encoder_hdl_works() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_mfm_encoder_trace() -> miette::Result<()> {
        let uut = MfmEncoder::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("mfm_encoder");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["2c6601eee83be7befe04347258766859557a1e67604df38ff0e25dbc69715345"];
        let digest = vcd.dump_to_file(root.join("mfm_encoder.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = MfmEncoder::fsm_descriptor();
        assert_eq!(desc.widget_name, "MfmEncoder");
        assert_eq!(desc.variants().len(), 3);
        assert_eq!(desc.initial_index(), 0);
    }
}
