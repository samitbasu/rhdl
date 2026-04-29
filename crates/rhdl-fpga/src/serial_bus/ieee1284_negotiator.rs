//! IEEE 1284 mode-select negotiator
//!
//! IEEE 1284 specifies a *negotiation handshake* the host runs
//! before switching the parallel port from the default
//! Compatibility mode into one of the bidirectional modes
//! (Nibble, Byte, EPP, ECP).  The handshake is short — a few
//! strobes and a mode byte — but it's the load-bearing piece
//! that lets a single physical port carry multiple protocols.
//!
//! **Standard mode bytes** (IEEE 1284-1994 Table 2):
//!
//! | Byte | Mode                                     |
//! |------|------------------------------------------|
//! | 0x00 | Compatibility (no extension; default)    |
//! | 0x10 | Nibble reverse channel                   |
//! | 0x20 | Byte reverse channel                     |
//! | 0x40 | ECP, no RLE                              |
//! | 0x50 | ECP with RLE                             |
//! | 0xC0 | EPP                                      |
//!
//! **Negotiation sequence** (high level):
//!
//! 1. Host writes the *mode byte* on `D[7:0]`.
//! 2. Host raises `nSelectIn` high *and* `nAutoFeed` high (the
//!    "1284 request" pattern).
//! 3. Device responds by pulling `nAck` low (it's listening).
//! 4. Host pulses `nStrobe` low for `t_strobe` cycles, then back
//!    high.
//! 5. Host releases `nSelectIn` / `nAutoFeed`.
//! 6. Device pulls `nAck` high (negotiation accepted).  At this
//!    point the wire is in the selected mode and the appropriate
//!    per-mode widget can take over.
//!
//! **v1 scope:** the basic mode-select handshake.  The widget
//! drives `mode_byte` + the request lines + the strobe and waits
//! for the device's `n_ack_in` to assert/deassert appropriately.
//! Failure handling (the device refuses by leaving `nAck` high) is
//! exposed as a `timeout` flag.  Extended-link IDs (the protocol
//! for negotiating *device-specific* features within a mode) and
//! the bus-arbitration glue that routes the pad to the active
//! per-mode widget are deferred to v2.
//!
//! Composes [super::super::core::dff::DFF] and [super::super::core::constant::Constant].
//! The host orchestrates the wire-level pad ownership: this widget
//! drives the negotiation pins, and after `done` pulses the host
//! gates the per-mode widget (`parallel_port_epp` / `parallel_port_ecp`)
//! onto the same physical pad.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+Ieee1284Negotiator+-------+
     |                                |
B<8> |                                | B<8>
+--->| mode_byte                d_out +--->
bool |                                | bool
+--->| start                  sel_in  +--->
bool |                                | bool
+--->| n_ack_in            auto_feed  +--->
     |                       n_strobe +--->
     |                           busy +--->
     |                           done +--->
     |                        timeout +--->
     +--------------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/ieee1284_negotiator.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/ieee1284_negotiator.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/ieee1284_negotiator_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the negotiation FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → DriveMode
    Transition {
        source_index: 1,
        target_index: 2,
    }, // DriveMode → WaitAckLow
    Transition {
        source_index: 1,
        target_index: 5,
    }, // DriveMode → Timeout (initial wait timed out)
    Transition {
        source_index: 2,
        target_index: 3,
    }, // WaitAckLow → StrobeLow
    Transition {
        source_index: 2,
        target_index: 5,
    }, // WaitAckLow → Timeout
    Transition {
        source_index: 3,
        target_index: 4,
    }, // StrobeLow → WaitAckHigh
    Transition {
        source_index: 4,
        target_index: 6,
    }, // WaitAckHigh → Done
    Transition {
        source_index: 4,
        target_index: 5,
    }, // WaitAckHigh → Timeout
    Transition {
        source_index: 5,
        target_index: 0,
    }, // Timeout → Idle
    Transition {
        source_index: 6,
        target_index: 0,
    }, // Done → Idle
];

/// Negotiation FSM state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum NegState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Drive mode_byte on D, assert nSelectIn + nAutoFeed; wait for nAck low.
    #[fsm_state(label = "drive mode")]
    DriveMode,
    /// nAck went low (device listening); wait for the strobe-time setup.
    #[fsm_state(label = "wait ACK low")]
    WaitAckLow,
    /// Pulse nStrobe low.
    #[fsm_state(label = "strobe low")]
    StrobeLow,
    /// nStrobe back high; wait for device's final ack (nAck high).
    #[fsm_state(label = "wait ACK high")]
    WaitAckHigh,
    /// Negotiation timed out — device refused.
    #[fsm_state(label = "timeout")]
    Timeout,
    /// Negotiation succeeded; pulse done.
    #[fsm_state(label = "done")]
    Done,
}

/// Strobe / timeout parameters in *FPGA cycles*.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct NegTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// nStrobe low-pulse width.
    pub t_strobe_low: Bits<T_W>,
    /// Maximum cycles to wait for device responses (nAck transitions).
    pub t_response_timeout: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = NegState)]
/// IEEE 1284 mode-select negotiator.
pub struct Ieee1284Negotiator<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<NegState>,
    tick: dff::DFF<Bits<T_W>>,
    mode_reg: dff::DFF<Bits<8>>,
    timings: Constant<NegTimings<T_W>>,
}

impl<const T_W: usize> Ieee1284Negotiator<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Construct a negotiator with the given strobe / timeout parameters.
    pub fn new(timings: NegTimings<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            mode_reg: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Ieee1284Negotiator].
pub struct In {
    /// Mode byte to negotiate (per IEEE 1284-1994 Table 2).  Latched at `start`.
    pub mode_byte: Bits<8>,
    /// Strobe to begin negotiation.  Ignored while busy.
    pub start: bool,
    /// Sampled `nAck` line from the device (already in FPGA clock domain).
    pub n_ack_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Ieee1284Negotiator].
pub struct Out {
    /// Data bus output (drives the mode byte during negotiation).
    pub d_out: Bits<8>,
    /// `nSelectIn` (driven high during the request pattern).
    pub sel_in: bool,
    /// `nAutoFeed` (driven high during the request pattern).
    pub auto_feed: bool,
    /// `nStrobe` (active low during the StrobeLow state).
    pub n_strobe: bool,
    pub busy: bool,
    /// Pulses for one cycle when negotiation completes successfully.
    pub done: bool,
    /// Pulses for one cycle if the device failed to respond (refused).
    pub timeout: bool,
}

impl<const T_W: usize> SynchronousIO for Ieee1284Negotiator<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = ieee1284_negotiator<T_W>;
}

#[kernel]
/// Kernel for [Ieee1284Negotiator].
pub fn ieee1284_negotiator<const T_W: usize>(
    cr: ClockReset,
    i: In,
    q: Q<T_W>,
) -> (Out, D<T_W>)
where
    rhdl::bits::W<T_W>: BitWidth,
{
    let one_t: Bits<T_W> = bits::<T_W>(1);
    let zero_t: Bits<T_W> = bits::<T_W>(0);

    let t = q.timings;

    let mut d = D::<T_W>::dont_care();
    d.state = q.state;
    d.tick = q.tick + one_t;
    d.mode_reg = q.mode_reg;

    let mut done_pulse = false;
    let mut timeout_pulse = false;

    match q.state {
        NegState::Idle => {
            d.tick = zero_t;
            if i.start {
                d.mode_reg = i.mode_byte;
                d.state = NegState::DriveMode;
                d.tick = zero_t;
            }
        }
        NegState::DriveMode => {
            // Wait for device to assert nAck low.
            if !i.n_ack_in {
                d.state = NegState::WaitAckLow;
                d.tick = zero_t;
            } else if q.tick >= t.t_response_timeout {
                d.state = NegState::Timeout;
                d.tick = zero_t;
            }
        }
        NegState::WaitAckLow => {
            // One-cycle setup; transition to strobe.
            d.state = NegState::StrobeLow;
            d.tick = zero_t;
        }
        NegState::StrobeLow => {
            if q.tick == t.t_strobe_low {
                d.state = NegState::WaitAckHigh;
                d.tick = zero_t;
            }
        }
        NegState::WaitAckHigh => {
            // Wait for device to deassert nAck (ack high) — confirms negotiation.
            if i.n_ack_in {
                d.state = NegState::Done;
                d.tick = zero_t;
            } else if q.tick >= t.t_response_timeout {
                d.state = NegState::Timeout;
                d.tick = zero_t;
            }
        }
        NegState::Timeout => {
            timeout_pulse = true;
            d.state = NegState::Idle;
            d.tick = zero_t;
        }
        NegState::Done => {
            done_pulse = true;
            d.state = NegState::Idle;
            d.tick = zero_t;
        }
    }

    if cr.reset.any() {
        d.state = NegState::Idle;
        d.tick = zero_t;
        d.mode_reg = bits::<8>(0);
    }

    let busy = q.state != NegState::Idle;
    // sel_in / auto_feed are high (requested) during the active-negotiation
    // states; they go low once we've reached Done or Timeout.
    let request_active = match q.state {
        NegState::DriveMode | NegState::WaitAckLow | NegState::StrobeLow | NegState::WaitAckHigh => true,
        _ => false,
    };
    let strobe_active = q.state == NegState::StrobeLow;

    let mut o = Out::dont_care();
    o.d_out = q.mode_reg;
    o.sel_in = request_active;
    o.auto_feed = request_active;
    o.n_strobe = !strobe_active;
    o.busy = busy;
    o.done = done_pulse;
    o.timeout = timeout_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn test_timings() -> NegTimings<8> {
        NegTimings {
            t_strobe_low: bits(4),
            t_response_timeout: bits(40),
        }
    }

    fn idle_in() -> In {
        In {
            mode_byte: bits(0),
            start: false,
            n_ack_in: true,
        }
    }

    /// Build a stream that runs negotiation with a cooperating device:
    /// at `ack_low_at` cycles after start, the device pulls nAck low; it
    /// then releases nAck `ack_release_after` cycles after that.
    fn drive_with_device(mode: u8, ack_low_at: usize, ack_release_after: usize) -> Vec<In> {
        let mut out: Vec<In> = vec![In {
            mode_byte: bits(mode as u128),
            start: true,
            n_ack_in: true,
        }];
        for _ in 0..ack_low_at {
            out.push(In {
                mode_byte: bits(0),
                start: false,
                n_ack_in: true,
            });
        }
        // Device pulls nAck low and holds.
        for _ in 0..ack_release_after {
            out.push(In {
                mode_byte: bits(0),
                start: false,
                n_ack_in: false,
            });
        }
        // Device releases nAck.
        for _ in 0..30 {
            out.push(idle_in());
        }
        out
    }

    #[test]
    fn test_idle_request_lines_low() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let stream = std::iter::repeat_n(idle_in(), 16)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().all(|s| !s.output.sel_in));
        assert!(outputs.iter().all(|s| !s.output.auto_feed));
        assert!(outputs.iter().all(|s| s.output.n_strobe));
        Ok(())
    }

    #[test]
    fn test_negotiation_succeeds_with_cooperating_device() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let stream_in = drive_with_device(0xC0, 4, 12);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.done), "negotiation never completed");
        assert!(!outputs.iter().any(|s| s.output.timeout), "spurious timeout");
        // d_out must hold 0xC0 during the active-request window.
        let bad = outputs
            .iter()
            .any(|s| s.output.sel_in && s.output.d_out.raw() != 0xC0);
        assert!(!bad, "mode byte not held on D during negotiation");
        Ok(())
    }

    #[test]
    fn test_timeout_when_device_never_responds() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        // Device leaves nAck high forever — should hit timeout.
        let mut stream_in: Vec<In> = vec![In {
            mode_byte: bits(0xC0),
            start: true,
            n_ack_in: true,
        }];
        for _ in 0..80 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.timeout), "no timeout pulse");
        assert!(!outputs.iter().any(|s| s.output.done));
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["9632"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_ieee1284_negotiator_hdl_works() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let stream_in = drive_with_device(0xC0, 4, 12);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ieee1284_negotiator_trace() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let stream_in = drive_with_device(0xC0, 4, 12);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ieee1284_negotiator");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["9941ede83b650a444c47f0e665a65e4fdcf9aa6138394dac8ef521041379498e"];
        let digest = vcd
            .dump_to_file(root.join("ieee1284_negotiator.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = Ieee1284Negotiator::<8>::fsm_descriptor();
        assert_eq!(desc.widget_name, "Ieee1284Negotiator");
        assert_eq!(desc.variants().len(), 7);
        assert_eq!(desc.initial_index(), 0);
    }
}
