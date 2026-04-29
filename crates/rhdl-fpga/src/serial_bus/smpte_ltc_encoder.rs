//! SMPTE LTC (Linear Timecode) bit-level encoder
//!
//! SMPTE 12M Linear Timecode is the time-of-day signal recorded
//! onto an audio track (or a dedicated wire) by every video editor
//! since the 1970s.  Each frame of video carries an 80-bit LTC
//! word — hours, minutes, seconds, frame-number, four 4-bit
//! user-data nibbles, eight binary-group flags, and a fixed 16-bit
//! sync word `0xBFFC` — encoded as **biphase mark code**.
//!
//! Biphase mark is the same line code as AES3 / S/PDIF audio:
//! every bit cell starts with a polarity transition; a `1` bit
//! adds a second transition in the middle of the cell.  This
//! self-clocks (every cell has at least one transition) and is
//! polarity-insensitive (an inverted line decodes identically),
//! which is why it's the canonical recording / wire choice.
//!
//! **v1 scope:** *bit-level* encoder.  The host strobes `bit_in`
//! once per frame-bit (the 80 bits of an LTC word, in their
//! defined LSB-first order); the widget toggles `line_out` on the
//! bit-cell start and (for a `1` bit) at the cell midpoint.  The
//! 80-bit frame builder, hours/minutes/seconds packing, the
//! drop-frame flag, the 16-bit sync word `0xBFFC`, and the
//! audio-band waveform driver (the host can hook `line_out` to a
//! [super::super::audio::audio_pwm::AudioPwm] mark/space generator
//! or directly to a digital pin) are deferred to v2.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+SmpteLtcEncoder+-------+
     |                               |
bool |                               | bool
+--->| bit_in              line_out  +--->
bool |                               | bool
+--->| cell_tick               busy  +--->
     |                          done +--->
     +-------------------------------+
")]
//!
//!# Internals
//!
//! Two cell phases per data bit.  For a `0` bit: line toggles
//! once at the *start* of phase A; for a `1` bit: line toggles at
//! the start of phase A *and* again at the boundary between A and B.
//! The host clocks the encoder via `cell_tick` — assert it for one
//! cycle to advance to the next half-cell.  This decouples the
//! widget from any specific bit rate (LTC's nominal rate is
//! 2400 Hz at 30 fps but ranges from 2000–2400 depending on
//! frame rate).
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/smpte_ltc_encoder.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/smpte_ltc_encoder.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/smpte_ltc_encoder_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::dff;

/// Author-curated transition graph for the LTC encoder FSM.
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
        target_index: 0,
    },
];

/// Internal state machine — half-cell granularity.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum LtcState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// First half of a cell — line was toggled at the start of this phase.
    #[fsm_state(label = "phase A")]
    PhaseA,
    /// Second half of a cell — for a `1` bit only, line was toggled at the
    /// boundary between phase A and B.
    #[fsm_state(label = "phase B")]
    PhaseB,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = LtcState)]
/// SMPTE LTC bit-level biphase mark encoder.
pub struct SmpteLtcEncoder {
    state: dff::DFF<LtcState>,
    /// Latched data bit for the current cell.
    cur_bit: dff::DFF<bool>,
    /// Toggling output line.
    line: dff::DFF<bool>,
    /// One-cycle done pulse at end of bit cell.
    done_pulse: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [SmpteLtcEncoder].
pub struct In {
    /// The data bit value to encode (sampled at the start of the cell).
    pub bit_in: bool,
    /// One-cycle strobe to advance to the next half-cell.
    pub cell_tick: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [SmpteLtcEncoder].
pub struct Out {
    /// Toggling line (the wire to record / transmit).
    pub line_out: bool,
    /// True while a bit cell is in flight (between PhaseA start and PhaseB end).
    pub busy: bool,
    /// Pulses high for one cycle at the end of each completed bit cell.
    pub done: bool,
}

impl SynchronousIO for SmpteLtcEncoder {
    type I = In;
    type O = Out;
    type Kernel = smpte_ltc_encoder;
}

#[kernel]
/// Kernel for [SmpteLtcEncoder].
pub fn smpte_ltc_encoder(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.cur_bit = q.cur_bit;
    d.line = q.line;
    d.done_pulse = false;

    match q.state {
        LtcState::Idle => {
            if i.cell_tick {
                // Start of a new cell — toggle the line and latch the bit.
                d.line = !q.line;
                d.cur_bit = i.bit_in;
                d.state = LtcState::PhaseA;
            }
        }
        LtcState::PhaseA => {
            if i.cell_tick {
                // Boundary into phase B — toggle again only if cur_bit is 1.
                // Done pulses here: by this point the entire cell's transition
                // pattern (one toggle for `0`, two for `1`) has been emitted.
                if q.cur_bit {
                    d.line = !q.line;
                }
                d.done_pulse = true;
                d.state = LtcState::PhaseB;
            }
        }
        LtcState::PhaseB => {
            if i.cell_tick {
                // Start of next cell — toggle, latch the next bit.
                d.line = !q.line;
                d.cur_bit = i.bit_in;
                d.state = LtcState::PhaseA;
            }
        }
    }

    if cr.reset.any() {
        d.state = LtcState::Idle;
        d.cur_bit = false;
        d.line = false;
        d.done_pulse = false;
    }

    let mut o = Out::dont_care();
    o.line_out = q.line;
    o.busy = q.state != LtcState::Idle;
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
            bit_in: false,
            cell_tick: false,
        }
    }

    /// Helper: feed a sequence of bits, two `cell_tick`s per bit (the start
    /// of each half-cell).  Idle frames between are inserted to keep the
    /// trace readable.
    fn drive_bits(bits_in: &[bool]) -> Vec<In> {
        let mut out = Vec::new();
        for &b in bits_in {
            // Tick 1: enter PhaseA (toggle line, latch bit).
            out.push(In {
                bit_in: b,
                cell_tick: true,
            });
            out.push(idle_in());
            // Tick 2: enter PhaseB (toggle if bit==1, pulse done).
            out.push(In {
                bit_in: b,
                cell_tick: true,
            });
            out.push(idle_in());
        }
        out
    }

    #[test]
    fn test_idle_no_toggles() -> miette::Result<()> {
        let uut = SmpteLtcEncoder::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // line_out should stay constant when no cell ticks happen.
        let initial = outputs.first().unwrap().output.line_out;
        assert!(outputs.iter().all(|s| s.output.line_out == initial));
        Ok(())
    }

    #[test]
    fn test_zero_bit_one_toggle() -> miette::Result<()> {
        let uut = SmpteLtcEncoder::default();
        let stream_in = drive_bits(&[false]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Count distinct line_out levels in the sequence — a `0` bit yields
        // one toggle (start-of-cell) so we observe two levels in the trace.
        let mut transitions = 0;
        for w in outputs.windows(2) {
            if w[0].output.line_out != w[1].output.line_out {
                transitions += 1;
            }
        }
        assert_eq!(transitions, 1, "0 bit must produce exactly one toggle");
        Ok(())
    }

    #[test]
    fn test_one_bit_two_toggles() -> miette::Result<()> {
        let uut = SmpteLtcEncoder::default();
        let stream_in = drive_bits(&[true]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let mut transitions = 0;
        for w in outputs.windows(2) {
            if w[0].output.line_out != w[1].output.line_out {
                transitions += 1;
            }
        }
        assert_eq!(transitions, 2, "1 bit must produce exactly two toggles");
        Ok(())
    }

    #[test]
    fn test_done_pulses_per_bit() -> miette::Result<()> {
        let uut = SmpteLtcEncoder::default();
        let stream_in = drive_bits(&[true, false, true, false]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_count = outputs.iter().filter(|s| s.output.done).count();
        assert_eq!(done_count, 4, "must pulse done once per bit");
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = SmpteLtcEncoder::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["6260"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_smpte_ltc_encoder_hdl_works() -> miette::Result<()> {
        let uut = SmpteLtcEncoder::default();
        let stream_in = drive_bits(&[true, false, true]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_smpte_ltc_encoder_trace() -> miette::Result<()> {
        let uut = SmpteLtcEncoder::default();
        let stream_in = drive_bits(&[true, false, true, false, true, true]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("smpte_ltc_encoder");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0590889bedbf8607c5a796ba983292ba5bccb736182aea4ad854a5e4ed574edb"];
        let digest = vcd
            .dump_to_file(root.join("smpte_ltc_encoder.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = SmpteLtcEncoder::fsm_descriptor();
        assert_eq!(desc.widget_name, "SmpteLtcEncoder");
        assert_eq!(desc.variants().len(), 3);
        assert_eq!(desc.initial_index(), 0);
    }
}
