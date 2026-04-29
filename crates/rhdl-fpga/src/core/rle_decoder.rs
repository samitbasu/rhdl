//! Run-Length Encoding (RLE) byte decoder
//!
//! Inverse of [super::rle_encoder::RleEncoder] — consumes the
//! `(byte, is_count)` beat stream and emits a flat byte stream by
//! expanding runs.  Built primarily for the ECP reverse channel of
//! the IEEE 1284 parallel port (where the device sends compressed
//! data back to the host) but reusable wherever RLE-encoded byte
//! streams need expanding.
//!
//! **Decoding rule (ECP-compatible):**
//!
//! - **Literal:** `(in_data = byte, in_is_count = false)` — emit
//!   `byte` once.
//! - **Run encoding:** two input beats. First
//!   `(in_data = count - 1, in_is_count = true)` — latch the
//!   count.  Then `(in_data = byte, in_is_count = false)` —
//!   emit `byte` `count` times.
//!
//! **v1 scope:** byte-at-a-time streaming decoder.  No FIFO
//! between input and output (the encoder's emit-rate has its own
//! back-pressure via `out_ready`).
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +---------+RleDecoder+---------+
     |                              |
B<8> |                              | B<8>
+--->| in_data            out_data  +--->
bool |                              | bool
+--->| in_is_count       out_valid  +--->
bool |                              | bool
+--->| in_valid             stalled +--->
bool |                              |
+--->| out_ready                    |
     +------------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/rle_decoder.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/rle_decoder.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/rle_decoder_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::dff;

/// Author-curated transition graph for the RLE-decoder FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition { source_index: 0, target_index: 0 }, // Idle → Idle (single literal emit)
    Transition { source_index: 0, target_index: 1 }, // Idle → ExpectData (count beat received)
    Transition { source_index: 1, target_index: 2 }, // ExpectData → EmitRun
    Transition { source_index: 2, target_index: 0 }, // EmitRun → Idle (run drained)
    Transition { source_index: 2, target_index: 2 }, // EmitRun self-loop
];

/// Decoder FSM state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum RleDecodeState {
    /// Waiting for input — either a literal (emit immediately) or a count beat.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Latched the count; waiting for the data beat that follows.
    #[fsm_state(label = "expect data")]
    ExpectData,
    /// Emitting the run, one byte per `out_ready`.
    #[fsm_state(label = "emit run")]
    EmitRun,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = RleDecodeState)]
/// Streaming RLE decoder.
pub struct RleDecoder {
    state: dff::DFF<RleDecodeState>,
    /// Latched run count (count - 1 from the wire; we count this many *additional*
    /// emits beyond the first, so 0 means "1 emit total").
    count_remaining: dff::DFF<Bits<8>>,
    /// Latched data byte for the current run.
    data_byte: dff::DFF<Bits<8>>,
    /// Latched literal byte (also stored here when emitting a single literal).
    literal_byte: dff::DFF<Bits<8>>,
    /// True when `literal_byte` carries a fresh literal that hasn't been drained yet.
    literal_valid: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [RleDecoder].
pub struct In {
    /// Next input byte (count or data).
    pub in_data: Bits<8>,
    /// True ⇒ `in_data` is a count byte; false ⇒ data byte.
    pub in_is_count: bool,
    /// Strobe to consume `in_data`.  Stalled if the decoder is mid-run-emit.
    pub in_valid: bool,
    /// Downstream consumer pulses this to advance to the next output byte.
    pub out_ready: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [RleDecoder].
pub struct Out {
    /// Decoded byte.
    pub out_data: Bits<8>,
    /// True while a fresh `out_data` is available.
    pub out_valid: bool,
    /// True if the decoder cannot accept more input this cycle.
    pub stalled: bool,
}

impl SynchronousIO for RleDecoder {
    type I = In;
    type O = Out;
    type Kernel = rle_decoder;
}

#[kernel]
/// Kernel for [RleDecoder].
pub fn rle_decoder(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.count_remaining = q.count_remaining;
    d.data_byte = q.data_byte;
    d.literal_byte = q.literal_byte;
    d.literal_valid = q.literal_valid;

    let one_8: Bits<8> = bits::<8>(1);

    // Input is accepted when state is Idle (literal not pending) or ExpectData.
    let stalled = (q.state == RleDecodeState::EmitRun) || q.literal_valid;

    match q.state {
        RleDecodeState::Idle => {
            if i.in_valid && !stalled {
                if i.in_is_count {
                    // Count beat — latch and wait for data.
                    d.count_remaining = i.in_data;
                    d.state = RleDecodeState::ExpectData;
                } else {
                    // Literal — pulse it once.
                    d.literal_byte = i.in_data;
                    d.literal_valid = true;
                }
            } else if q.literal_valid && i.out_ready {
                // Drain the literal.
                d.literal_valid = false;
            }
        }
        RleDecodeState::ExpectData => {
            if i.in_valid {
                d.data_byte = i.in_data;
                d.state = RleDecodeState::EmitRun;
            }
        }
        RleDecodeState::EmitRun => {
            if i.out_ready {
                if q.count_remaining == bits::<8>(0) {
                    // Last byte of the run — back to Idle.
                    d.state = RleDecodeState::Idle;
                } else {
                    d.count_remaining = q.count_remaining - one_8;
                }
            }
        }
    }

    if cr.reset.any() {
        d.state = RleDecodeState::Idle;
        d.count_remaining = bits::<8>(0);
        d.data_byte = bits::<8>(0);
        d.literal_byte = bits::<8>(0);
        d.literal_valid = false;
    }

    let out_data: Bits<8> = match q.state {
        RleDecodeState::EmitRun => q.data_byte,
        _ => q.literal_byte,
    };
    let out_valid = (q.state == RleDecodeState::EmitRun) || q.literal_valid;

    let mut o = Out::dont_care();
    o.out_data = out_data;
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
            in_is_count: false,
            in_valid: false,
            out_ready: false,
        }
    }

    /// Drive a sequence of input beats and capture decoded bytes.
    fn drive_and_capture(beats: &[(u8, bool)]) -> Vec<u8> {
        let uut = RleDecoder::default();
        let mut stream_in: Vec<In> = Vec::new();
        for &(b, is_count) in beats {
            stream_in.push(In {
                in_data: bits(b as u128),
                in_is_count: is_count,
                in_valid: true,
                out_ready: true,
            });
            // Several quiet cycles with out_ready high to drain.
            for _ in 0..6 {
                stream_in.push(In {
                    in_data: bits(0),
                    in_is_count: false,
                    in_valid: false,
                    out_ready: true,
                });
            }
        }
        // Trailing settle.
        for _ in 0..16 {
            stream_in.push(In {
                in_data: bits(0),
                in_is_count: false,
                in_valid: false,
                out_ready: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        uut.run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.out_valid && s.input.1.out_ready)
            .map(|s| s.output.out_data.raw() as u8)
            .collect()
    }

    #[test]
    fn test_idle_no_output() -> miette::Result<()> {
        let uut = RleDecoder::default();
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
    fn test_single_literal() -> miette::Result<()> {
        let captured = drive_and_capture(&[(0x42, false)]);
        assert_eq!(captured, vec![0x42]);
        Ok(())
    }

    #[test]
    fn test_two_literals() -> miette::Result<()> {
        let captured = drive_and_capture(&[(0x42, false), (0x43, false)]);
        assert_eq!(captured, vec![0x42, 0x43]);
        Ok(())
    }

    #[test]
    fn test_run_of_three() -> miette::Result<()> {
        // Wire encoding for run-of-3: count=2 (run_length-1), data=0xAA.
        let captured = drive_and_capture(&[(2, true), (0xAA, false)]);
        assert_eq!(captured, vec![0xAA, 0xAA, 0xAA]);
        Ok(())
    }

    #[test]
    fn test_run_then_literal() -> miette::Result<()> {
        let captured = drive_and_capture(&[(2, true), (0xAA, false), (0xBB, false)]);
        assert_eq!(captured, vec![0xAA, 0xAA, 0xAA, 0xBB]);
        Ok(())
    }

    /// End-to-end round-trip: encode → decode should reproduce the original.
    #[test]
    fn test_roundtrip_with_encoder() -> miette::Result<()> {
        use crate::core::rle_encoder::{RleEncoder, In as EncIn};
        let original: Vec<u8> = vec![0x42, 0xAA, 0xAA, 0xAA, 0xBB, 0xCC, 0xCC];

        // Step 1: encode.
        let enc = RleEncoder::default();
        let mut enc_stream_in: Vec<EncIn> = Vec::new();
        for &b in &original {
            enc_stream_in.push(EncIn {
                in_data: bits(b as u128),
                in_valid: true,
                flush: false,
                out_ready: true,
            });
            for _ in 0..3 {
                enc_stream_in.push(EncIn {
                    in_data: bits(0),
                    in_valid: false,
                    flush: false,
                    out_ready: true,
                });
            }
        }
        enc_stream_in.push(EncIn {
            in_data: bits(0),
            in_valid: false,
            flush: true,
            out_ready: true,
        });
        for _ in 0..16 {
            enc_stream_in.push(EncIn {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                out_ready: true,
            });
        }
        let stream = enc_stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let beats: Vec<(u8, bool)> = enc
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.out_valid && s.input.1.out_ready)
            .map(|s| (s.output.out_data.raw() as u8, s.output.out_is_count))
            .collect();

        // Step 2: feed beats into the decoder.
        let recovered = drive_and_capture(&beats);
        assert_eq!(
            recovered, original,
            "round-trip mismatch: encoded {beats:02x?}, decoded {recovered:02x?}"
        );
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = RleDecoder::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8278"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_rle_decoder_hdl_works() -> miette::Result<()> {
        let uut = RleDecoder::default();
        let mut stream_in: Vec<In> = Vec::new();
        for &(b, is_count) in &[(2u8, true), (0xAA, false), (0xBB, false)] {
            stream_in.push(In {
                in_data: bits(b as u128),
                in_is_count: is_count,
                in_valid: true,
                out_ready: true,
            });
            for _ in 0..3 {
                stream_in.push(In {
                    in_data: bits(0),
                    in_is_count: false,
                    in_valid: false,
                    out_ready: true,
                });
            }
        }
        for _ in 0..16 {
            stream_in.push(In {
                in_data: bits(0),
                in_is_count: false,
                in_valid: false,
                out_ready: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_rle_decoder_trace() -> miette::Result<()> {
        let uut = RleDecoder::default();
        let mut stream_in: Vec<In> = Vec::new();
        for &(b, is_count) in &[(2u8, true), (0xAA, false), (0xBB, false)] {
            stream_in.push(In {
                in_data: bits(b as u128),
                in_is_count: is_count,
                in_valid: true,
                out_ready: true,
            });
            for _ in 0..3 {
                stream_in.push(In {
                    in_data: bits(0),
                    in_is_count: false,
                    in_valid: false,
                    out_ready: true,
                });
            }
        }
        for _ in 0..16 {
            stream_in.push(In {
                in_data: bits(0),
                in_is_count: false,
                in_valid: false,
                out_ready: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("rle_decoder");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["064f1105b1c600c4591d7f189211a61d15c049ce95f589e07ca28754aab9cfa2"];
        let digest = vcd.dump_to_file(root.join("rle_decoder.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = RleDecoder::fsm_descriptor();
        assert_eq!(desc.widget_name, "RleDecoder");
        assert_eq!(desc.variants().len(), 3);
        assert_eq!(desc.initial_index(), 0);
    }
}
