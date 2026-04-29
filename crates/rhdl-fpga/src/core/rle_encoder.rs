//! Run-Length Encoding (RLE) byte encoder
//!
//! Generic RLE encoder for byte streams.  Built primarily for the
//! ECP (Extended Capabilities Port) compression mode of the IEEE
//! 1284 parallel port, but reusable wherever streaming run-length
//! compression is wanted (storage prefilters, bandwidth-conserving
//! framing for low-rate links, simple compressed video frame
//! buffering).
//!
//! **Encoding format (ECP-compatible).**  The output stream is a
//! mixture of *literal bytes* and *run encodings*:
//!
//! - **Literal byte:** `(out_data = byte, out_is_count = false)`
//!   — a single byte that didn't repeat enough to compress.
//! - **Run encoding:** two output beats: first
//!   `(out_data = count - 1, out_is_count = true)` (with `count`
//!   in the range 2..=128), then `(out_data = byte,
//!   out_is_count = false)` carrying the actual byte value that
//!   repeated.
//!
//! This matches the IEEE 1284 ECP wire encoding bit-for-bit: the
//! `is_count` flag corresponds to the *RLE-cycle-type* bit on the
//! ECP wire, and `count - 1` matches the wire encoding of the run
//! length (so `count = 1` is impossible on the wire — single bytes
//! become literals).
//!
//! **v1 scope:** byte-at-a-time streaming encoder.  Host strobes
//! one byte per `in_valid` and pulses `flush` when the input
//! stream ends to push out the trailing in-progress run.  The
//! decoder is a separate widget (deferred to v2 along with a
//! `core::rle_decoder` module).
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +---------+RleEncoder+---------+
     |                              |
B<8> |                              | B<8>
+--->| in_data            out_data  +--->
bool |                              | bool
+--->| in_valid       out_is_count  +--->
bool |                              | bool
+--->| flush             out_valid  +--->
bool |                              |
+--->| out_ready                    |
     +------------------------------+
")]
//!
//!# Internals
//!
//! The encoder holds two registers — `prev_byte` (last byte
//! consumed) and `run_count` (how many consecutive `prev_byte`s
//! we've seen, 1..=128).  An output queue is a small FSM:
//!
//! - `Idle` — no output pending; waiting for the host to push a
//!   byte that breaks the run (or to flush).
//! - `EmitCount` — output beat 1 of a 2-beat run encoding (count
//!   byte).
//! - `EmitData` — output beat 2 of a run, OR a single literal
//!   byte; transitions back to Idle.
//!
//! Single-byte runs (no repeat) skip `EmitCount` and emit the
//! literal directly.  Runs of 2..=128 use both beats.  When
//! `run_count` saturates at 128 the encoder forces an emit even
//! mid-stream so the count never overflows.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/rle_encoder.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/rle_encoder.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/rle_encoder_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::dff;

/// Author-curated transition graph for the RLE-encoder FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → EmitCount (run ≥ 2 needs a count beat)
    Transition {
        source_index: 0,
        target_index: 2,
    }, // Idle → EmitData (single-byte literal)
    Transition {
        source_index: 1,
        target_index: 2,
    }, // EmitCount → EmitData
    Transition {
        source_index: 2,
        target_index: 0,
    }, // EmitData → Idle
];

/// Output sequencing state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum RleOut {
    /// Nothing pending — accumulate input.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Beat 1 of a run encoding: emit the count byte.
    #[fsm_state(label = "emit count")]
    EmitCount,
    /// Beat 2 of a run encoding (or sole beat of a literal): emit the data byte.
    #[fsm_state(label = "emit data")]
    EmitData,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = RleOut)]
/// Streaming RLE encoder.
pub struct RleEncoder {
    state: dff::DFF<RleOut>,
    /// Last byte consumed from the input.
    prev_byte: dff::DFF<Bits<8>>,
    /// Number of times `prev_byte` has been seen consecutively (1..=128).
    /// 0 means "no byte yet observed since reset".
    run_count: dff::DFF<Bits<8>>,
    /// True when prev_byte / run_count carry a real run waiting to be emitted.
    /// (Distinguishes "0 means empty" from "0 means saturated counter".)
    has_pending: dff::DFF<bool>,
    /// Latched count byte for the current emit (count - 1 to match ECP wire).
    emit_count_byte: dff::DFF<Bits<8>>,
    /// Latched data byte for the current emit.
    emit_data_byte: dff::DFF<Bits<8>>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [RleEncoder].
pub struct In {
    /// Next input byte (consumed when `in_valid` is high and not stalled).
    pub in_data: Bits<8>,
    /// Strobe to consume `in_data`.  Stalled if the encoder is mid-emit.
    pub in_valid: bool,
    /// Strobe when the input stream ends — pushes any in-progress run to output.
    pub flush: bool,
    /// Downstream consumer pulses this to advance to the next output beat.
    pub out_ready: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [RleEncoder].
pub struct Out {
    /// Output beat (count byte if `out_is_count`, else data byte).
    pub out_data: Bits<8>,
    /// True ⇒ `out_data` is a *run-count* byte (= run_length - 1).  False ⇒ a data byte.
    pub out_is_count: bool,
    /// True while a fresh `out_data` is available for the consumer.
    pub out_valid: bool,
    /// True if the encoder cannot accept more input this cycle (output stalled).
    pub stalled: bool,
}

impl SynchronousIO for RleEncoder {
    type I = In;
    type O = Out;
    type Kernel = rle_encoder;
}

#[kernel]
/// Kernel for [RleEncoder].
pub fn rle_encoder(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.prev_byte = q.prev_byte;
    d.run_count = q.run_count;
    d.has_pending = q.has_pending;
    d.emit_count_byte = q.emit_count_byte;
    d.emit_data_byte = q.emit_data_byte;

    let one_8: Bits<8> = bits::<8>(1);

    // Stalled = an emit is in progress.  We can take a byte only in Idle.
    let stalled = q.state != RleOut::Idle;

    // Output multiplexing.
    let out_data: Bits<8> = match q.state {
        RleOut::EmitCount => q.emit_count_byte,
        _ => q.emit_data_byte,
    };
    let out_is_count = q.state == RleOut::EmitCount;
    let out_valid = q.state != RleOut::Idle;

    match q.state {
        RleOut::Idle => {
            // Decide what to do based on (in_valid, flush, has_pending).
            if i.in_valid && !stalled {
                if !q.has_pending {
                    // First byte ever — just latch it.
                    d.prev_byte = i.in_data;
                    d.run_count = one_8;
                    d.has_pending = true;
                } else if i.in_data == q.prev_byte {
                    // Continued run.  Saturate at 128 — if we'd hit 129,
                    // emit the current run first and start a new one with
                    // this byte.
                    if q.run_count == bits::<8>(128) {
                        // Saturate: emit current run, then re-latch the
                        // incoming byte as a new fresh run of 1.
                        d.emit_count_byte = q.run_count - one_8;
                        d.emit_data_byte = q.prev_byte;
                        d.state = RleOut::EmitCount;
                        d.run_count = one_8;
                    } else {
                        d.run_count = q.run_count + one_8;
                    }
                } else {
                    // Run broken — emit the current run, then latch the new byte.
                    if q.run_count == one_8 {
                        // Single-byte literal: skip EmitCount, go straight to EmitData.
                        d.emit_data_byte = q.prev_byte;
                        d.state = RleOut::EmitData;
                    } else {
                        d.emit_count_byte = q.run_count - one_8;
                        d.emit_data_byte = q.prev_byte;
                        d.state = RleOut::EmitCount;
                    }
                    d.prev_byte = i.in_data;
                    d.run_count = one_8;
                }
            } else if i.flush && q.has_pending {
                // Push out the in-progress run.
                if q.run_count == one_8 {
                    d.emit_data_byte = q.prev_byte;
                    d.state = RleOut::EmitData;
                } else {
                    d.emit_count_byte = q.run_count - one_8;
                    d.emit_data_byte = q.prev_byte;
                    d.state = RleOut::EmitCount;
                }
                d.has_pending = false;
                d.run_count = bits::<8>(0);
            }
        }
        RleOut::EmitCount => {
            if i.out_ready {
                d.state = RleOut::EmitData;
            }
        }
        RleOut::EmitData => {
            if i.out_ready {
                d.state = RleOut::Idle;
            }
        }
    }

    if cr.reset.any() {
        d.state = RleOut::Idle;
        d.prev_byte = bits::<8>(0);
        d.run_count = bits::<8>(0);
        d.has_pending = false;
        d.emit_count_byte = bits::<8>(0);
        d.emit_data_byte = bits::<8>(0);
    }

    let mut o = Out::dont_care();
    o.out_data = out_data;
    o.out_is_count = out_is_count;
    o.out_valid = out_valid;
    o.stalled = stalled;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            in_data: bits(0),
            in_valid: false,
            flush: false,
            out_ready: false,
        }
    }

    /// Drive a sequence of input bytes (with `out_ready` always asserted to
    /// drain immediately) and capture every emitted (data, is_count) tuple.
    fn drive_and_capture(uut: &RleEncoder, input: &[u8]) -> Vec<(u8, bool)> {
        let mut stream_in: Vec<In> = Vec::new();
        for &b in input {
            // Push byte; loop while stalled (the test setup just pushes
            // each byte and lets the FSM consume it on a quiet cycle).
            stream_in.push(In {
                in_data: bits(b as u128),
                in_valid: true,
                flush: false,
                out_ready: true,
            });
            // Two quiet cycles to let the FSM emit if needed.
            for _ in 0..3 {
                stream_in.push(In {
                    in_data: bits(0),
                    in_valid: false,
                    flush: false,
                    out_ready: true,
                });
            }
        }
        // Flush to push final run.
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: true,
            out_ready: true,
        });
        // Keep out_ready high during trailing settle so the FSM can drain.
        for _ in 0..16 {
            stream_in.push(In {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                out_ready: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        uut.run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.out_valid && s.input.1.out_ready)
            .map(|s| {
                (
                    s.output.out_data.raw() as u8,
                    s.output.out_is_count,
                )
            })
            .collect()
    }

    #[test]
    fn test_idle_no_output() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let stream = std::iter::repeat_n(idle_in(), 16)
            .with_reset(1)
            .clock_pos_edge(100);
        let any = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.out_valid);
        assert!(!any);
        Ok(())
    }

    #[test]
    fn test_single_literal_byte() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let captured = drive_and_capture(&uut, &[0x42]);
        // Single byte → one literal beat: (0x42, false).
        assert_eq!(captured, vec![(0x42, false)]);
        Ok(())
    }

    #[test]
    fn test_two_distinct_bytes() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let captured = drive_and_capture(&uut, &[0x42, 0x43]);
        // Two literals: (0x42, false), (0x43, false).
        assert_eq!(captured, vec![(0x42, false), (0x43, false)]);
        Ok(())
    }

    #[test]
    fn test_run_of_three() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let captured = drive_and_capture(&uut, &[0xAA, 0xAA, 0xAA]);
        // Three identical → one run encoding: count = 3 (wire = 2), data = 0xAA.
        assert_eq!(captured, vec![(2, true), (0xAA, false)]);
        Ok(())
    }

    #[test]
    fn test_run_then_literal() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let captured = drive_and_capture(&uut, &[0xAA, 0xAA, 0xAA, 0xBB]);
        // Run of 3 then literal — count=3 (wire=2), data=0xAA, then literal 0xBB.
        assert_eq!(captured, vec![(2, true), (0xAA, false), (0xBB, false)]);
        Ok(())
    }

    #[test]
    fn test_long_run_saturates_at_128() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let input: Vec<u8> = std::iter::repeat(0x55).take(130).collect();
        let captured = drive_and_capture(&uut, &input);
        // First 128 → one run encoding (wire count = 127).
        // Remaining 2 → second run encoding (wire count = 1).
        assert_eq!(
            captured,
            vec![
                (127, true),
                (0x55, false),
                (1, true),
                (0x55, false),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["11703"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_rle_encoder_hdl_works() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let mut stream_in: Vec<In> = Vec::new();
        for &b in &[0x42u8, 0xAA, 0xAA, 0xAA, 0xBB] {
            stream_in.push(In {
                in_data: bits(b as u128),
                in_valid: true,
                flush: false,
                out_ready: true,
            });
            for _ in 0..3 {
                stream_in.push(In {
                    in_data: bits(0),
                    in_valid: false,
                    flush: false,
                    out_ready: true,
                });
            }
        }
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: true,
            out_ready: true,
        });
        for _ in 0..6 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_rle_encoder_trace() -> miette::Result<()> {
        let uut = RleEncoder::default();
        let mut stream_in: Vec<In> = Vec::new();
        for &b in &[0x42u8, 0xAA, 0xAA, 0xAA, 0xBB] {
            stream_in.push(In {
                in_data: bits(b as u128),
                in_valid: true,
                flush: false,
                out_ready: true,
            });
            for _ in 0..3 {
                stream_in.push(In {
                    in_data: bits(0),
                    in_valid: false,
                    flush: false,
                    out_ready: true,
                });
            }
        }
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: true,
            out_ready: true,
        });
        for _ in 0..6 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("rle_encoder");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["14e589f4de9e6ddad1d57e8b6837c0b9da4f14437adba91a71fa85f7528f3eb1"];
        let digest = vcd.dump_to_file(root.join("rle_encoder.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = RleEncoder::fsm_descriptor();
        assert_eq!(desc.widget_name, "RleEncoder");
        assert_eq!(desc.variants().len(), 3);
        assert_eq!(desc.initial_index(), 0);
    }
}
