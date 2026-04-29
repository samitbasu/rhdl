//! IEEE 1284 ECP (Extended Capabilities Port) master, forward channel
//!
//! ECP is IEEE 1284 mode 5 — bidirectional, FIFO-buffered,
//! RLE-compressed parallel I/O.  Compared to [super::parallel_port_epp]
//! (which is a clean register-mapped 4-cycle bus) ECP adds:
//!
//! 1. **Run-length compression of the data stream** — repeated
//!    bytes are encoded as `(count, byte)` pairs by the
//!    [super::super::core::rle_encoder::RleEncoder] composed
//!    inside this widget.  Uncompressed (literal) bytes pass
//!    through unchanged.
//! 2. **`HostClk` encoding of byte type** — the wire's `HostClk`
//!    line distinguishes a *data* byte from a *command / RLE-count*
//!    byte.  The receiver looks at `HostClk` to decide whether to
//!    pass the byte to the application or interpret it as a run
//!    length.
//! 3. **Interlocked handshake per byte** — host drives `D[7:0]`
//!    + `HostClk` + asserts `nStrobe` low; device acknowledges
//!    via `PeriphAck`; host releases.  Same family as EPP's
//!    `nWAIT` interlock but using ECP-namespace pin names.
//! 4. **(In v2) FIFO buffering and a reverse channel** so the
//!    host can stream-write a compressed run while the device
//!    drains it asynchronously.
//!
//! **v1 scope:** *forward channel only* (host → device), composing
//! [super::super::core::rle_encoder::RleEncoder] for run-length
//! compression and a small handshake FSM for each output beat.
//! The reverse channel (device → host), the IEEE 1284 mode-select
//! negotiation, the ECP-A FIFO (deeper than the encoder's two-beat
//! buffer), and the `nReverseRequest` / `nPeriphRequest` /
//! `nAckReverse` lines are all v2.
//!
//! Composes [super::super::core::rle_encoder::RleEncoder].
//! The bidirectional `D` bus is exposed as `(d_oe, d_out)`; v2
//! adds `d_in` for the reverse channel.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +------+ParallelPortEcp+--------+
     |                               |
B<8> |                               | B<8>
+--->| in_data                d_out  +--->
bool |                               | bool
+--->| in_valid                d_oe  +--->
bool |                               | bool
+--->| flush             host_clk    +--->
bool |                               | bool
+--->| periph_ack_in       n_strobe  +--->
     |                          busy +--->
     |                          done +--->
     +-------------------------------+
")]
//!
//!# Internals
//!
//! Two sub-circuits and one FSM:
//!
//! - **`RleEncoder`** — consumes `in_data` / `in_valid` / `flush`,
//!   produces `(out_data, out_is_count)` beats.
//! - **Handshake FSM** — for each beat from RLE, runs a 4-state
//!   interlocked handshake (`Idle / Drive / WaitAck / Release`)
//!   with the device, mirroring the EPP `nWAIT` discipline.
//!
//! The combined widget is the *systemic* ECP forward channel:
//! upstream byte stream goes in, ECP-encoded wire signals come
//! out, with the encoder handling compression and the FSM
//! handling per-byte handshake.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/parallel_port_ecp.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/parallel_port_ecp.md")]
//!
//! And the auto-generated FSM diagram for the handshake FSM:
#![doc = include_str!("../../doc/parallel_port_ecp_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use super::super::core::rle_encoder::{RleEncoder, rle_encoder as rle_encoder_kernel};
use crate::core::dff;

#[allow(unused_imports)]
use rle_encoder_kernel as _;

/// Author-curated transition graph for the ECP handshake FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → Drive
    Transition {
        source_index: 1,
        target_index: 2,
    }, // Drive → WaitAck
    Transition {
        source_index: 2,
        target_index: 3,
    }, // WaitAck → Release
    Transition {
        source_index: 3,
        target_index: 0,
    }, // Release → Idle
];

/// ECP per-beat handshake FSM.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum EcpHandshake {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Driving D + HostClk + nStrobe low.
    #[fsm_state(label = "drive byte")]
    Drive,
    /// Waiting for the device to assert PeriphAck.
    #[fsm_state(label = "wait ack")]
    WaitAck,
    /// Releasing nStrobe; waiting for PeriphAck to drop.
    #[fsm_state(label = "release")]
    Release,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = EcpHandshake)]
/// IEEE 1284 ECP forward-channel master.
pub struct ParallelPortEcp {
    /// RLE encoder owns the compression layer.
    rle: RleEncoder,
    /// Handshake FSM walks through `Drive / WaitAck / Release` per beat.
    state: dff::DFF<EcpHandshake>,
    /// Latched beat data (snapshotted at Idle → Drive so the encoder can
    /// move on without disturbing the wire output mid-handshake).
    beat_data_reg: dff::DFF<Bits<8>>,
    /// Latched is_count flag for the current beat.
    beat_is_count_reg: dff::DFF<bool>,
    done_pulse: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [ParallelPortEcp].
pub struct In {
    /// Next input byte (consumed when `in_valid` is high and the encoder + handshake aren't stalled).
    pub in_data: Bits<8>,
    /// Strobe to consume `in_data`.
    pub in_valid: bool,
    /// Strobe when the input stream ends — pushes any in-progress run.
    pub flush: bool,
    /// Sampled `PeriphAck` from the device (already in FPGA clock domain).
    pub periph_ack_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [ParallelPortEcp].
pub struct Out {
    /// 8-bit data driven on D[7:0] (only meaningful while `d_oe == 1`).
    pub d_out: Bits<8>,
    /// True ⇒ host drives the D bus.
    pub d_oe: bool,
    /// `HostClk` line — distinguishes data (low) vs command/RLE-count (high) byte.
    pub host_clk: bool,
    /// `nStrobe` (active-low data-valid strobe).
    pub n_strobe: bool,
    /// True while compressing or transmitting.
    pub busy: bool,
    /// Pulses for one cycle when the input stream is fully drained
    /// (RLE encoder Idle and handshake FSM Idle simultaneously).
    pub done: bool,
}

impl SynchronousIO for ParallelPortEcp {
    type I = In;
    type O = Out;
    type Kernel = parallel_port_ecp;
}

#[kernel]
/// Kernel for [ParallelPortEcp].
pub fn parallel_port_ecp(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.beat_data_reg = q.beat_data_reg;
    d.beat_is_count_reg = q.beat_is_count_reg;
    d.done_pulse = false;

    // Wire RLE encoder inputs.  out_ready pulses only at the Idle → Drive
    // edge — once we've snapshotted the beat we let the encoder advance.
    // Default false; set to true on the cycle we transition to Drive.
    let mut rle_out_ready = false;

    // Read RLE encoder outputs (registered).
    let beat_data = q.rle.out_data;
    let beat_is_count = q.rle.out_is_count;
    let beat_valid = q.rle.out_valid;

    match q.state {
        EcpHandshake::Idle => {
            // If RLE has a beat for us, snapshot it and start the handshake.
            if beat_valid {
                d.beat_data_reg = beat_data;
                d.beat_is_count_reg = beat_is_count;
                d.state = EcpHandshake::Drive;
                rle_out_ready = true; // Let encoder advance to next beat.
            }
        }
        EcpHandshake::Drive => {
            // Single setup cycle; move to wait.
            d.state = EcpHandshake::WaitAck;
        }
        EcpHandshake::WaitAck => {
            if i.periph_ack_in {
                d.state = EcpHandshake::Release;
            }
        }
        EcpHandshake::Release => {
            // Wait for ack to deassert, then back to Idle (and pulse done).
            if !i.periph_ack_in {
                d.state = EcpHandshake::Idle;
                d.done_pulse = true;
            }
        }
    }

    d.rle = super::super::core::rle_encoder::In {
        in_data: i.in_data,
        in_valid: i.in_valid,
        flush: i.flush,
        out_ready: rle_out_ready,
    };

    if cr.reset.any() {
        d.state = EcpHandshake::Idle;
        d.beat_data_reg = bits::<8>(0);
        d.beat_is_count_reg = false;
        d.done_pulse = false;
    }

    let busy = (q.state != EcpHandshake::Idle) || beat_valid || q.rle.stalled;
    let drive_active = match q.state {
        EcpHandshake::Drive | EcpHandshake::WaitAck => true,
        _ => false,
    };

    let mut o = Out::dont_care();
    o.d_out = q.beat_data_reg;
    o.d_oe = drive_active;
    o.host_clk = q.beat_is_count_reg; // HostClk high = command/count, low = data.
    o.n_strobe = !drive_active;
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
            in_data: bits(0),
            in_valid: false,
            flush: false,
            periph_ack_in: false,
        }
    }

    /// Drive a sequence of bytes through ECP.  Strategy: push all bytes
    /// back-to-back (no acks between — the bytes don't emit anything since
    /// the run is still building), then flush, then continuously alternate
    /// (idle, ack) pulses to drain whatever the encoder produced.
    /// Returns `(d_out, host_clk)` captured at every cycle where `done`
    /// pulsed (one entry per emitted beat).
    fn drive_and_capture(input: &[u8]) -> Vec<(u8, bool)> {
        let uut = ParallelPortEcp::default();
        let mut stream_in: Vec<In> = Vec::new();
        // Push all input bytes back-to-back, with periph_ack_in=false.
        for &b in input {
            stream_in.push(In {
                in_data: bits(b as u128),
                in_valid: true,
                flush: false,
                periph_ack_in: false,
            });
            // One quiet cycle so the encoder consumes (in_valid is single-cycle).
            stream_in.push(idle_in());
        }
        // Flush.
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: true,
            periph_ack_in: false,
        });
        // Drain phase: continuously alternate (idle, idle, ack, idle) so the
        // handshake can walk Drive → WaitAck → Release → Idle for each beat.
        for _ in 0..40 {
            stream_in.push(idle_in());
            stream_in.push(idle_in());
            stream_in.push(In {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                periph_ack_in: true,
            });
            stream_in.push(In {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                periph_ack_in: true,
            });
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        outputs
            .iter()
            .filter(|s| s.output.done)
            .map(|s| (s.output.d_out.raw() as u8, s.output.host_clk))
            .collect()
    }

    #[test]
    fn test_idle_no_strobe() -> miette::Result<()> {
        let uut = ParallelPortEcp::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_strobe = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| !s.output.n_strobe);
        assert!(!any_strobe, "no nStrobe activity in idle");
        Ok(())
    }

    #[test]
    fn test_single_literal_byte() -> miette::Result<()> {
        let captured = drive_and_capture(&[0x42]);
        // Single literal: one beat with host_clk=false (data byte).
        assert_eq!(captured, vec![(0x42, false)]);
        Ok(())
    }

    #[test]
    fn test_run_of_three_bytes() -> miette::Result<()> {
        let captured = drive_and_capture(&[0xAA, 0xAA, 0xAA]);
        // Run of 3 → two beats: count (host_clk=true, data=2), then byte
        // (host_clk=false, data=0xAA).
        assert_eq!(captured, vec![(2, true), (0xAA, false)]);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = ParallelPortEcp::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["19926"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_parallel_port_ecp_hdl_works() -> miette::Result<()> {
        let uut = ParallelPortEcp::default();
        let mut stream_in: Vec<In> = vec![In {
            in_data: bits(0x42),
            in_valid: true,
            flush: false,
            periph_ack_in: false,
        }];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        for _ in 0..2 {
            stream_in.push(In {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                periph_ack_in: true,
            });
        }
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: true,
            periph_ack_in: false,
        });
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_parallel_port_ecp_trace() -> miette::Result<()> {
        let uut = ParallelPortEcp::default();
        let mut stream_in: Vec<In> = Vec::new();
        for &b in &[0x42u8, 0xAA, 0xAA, 0xAA] {
            stream_in.push(In {
                in_data: bits(b as u128),
                in_valid: true,
                flush: false,
                periph_ack_in: false,
            });
            for _ in 0..6 {
                stream_in.push(idle_in());
            }
            for _ in 0..2 {
                stream_in.push(In {
                    in_data: bits(0),
                    in_valid: false,
                    flush: false,
                    periph_ack_in: true,
                });
            }
            for _ in 0..6 {
                stream_in.push(idle_in());
            }
        }
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: true,
            periph_ack_in: false,
        });
        for _ in 0..40 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("parallel_port_ecp");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["9228b655a613ac848899b36d83224aad77ca0c0c262b7c4cc864c33449c0bc56"];
        let digest = vcd
            .dump_to_file(root.join("parallel_port_ecp.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = ParallelPortEcp::fsm_descriptor();
        assert_eq!(desc.widget_name, "ParallelPortEcp");
        assert_eq!(desc.variants().len(), 4);
        assert_eq!(desc.initial_index(), 0);
    }
}
