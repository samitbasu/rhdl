//! IEEE 1284 mode-select negotiator (full corner-case coverage)
//!
//! IEEE 1284 specifies a *negotiation phase* the host runs before
//! switching the parallel port from the default Compatibility mode
//! into one of the bidirectional modes (Nibble, Byte, EPP, ECP).
//! It also specifies a *termination phase* the host runs to return
//! to Compatibility mode and a set of mandatory failure paths
//! (device-not-compliant, mode-rejected, response-timeout).
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
//! | 0x80–0xFF | Extended link ID request (variable) |
//!
//! **Negotiation Phase events** (IEEE 1284-1994 §6.3.4):
//!
//! 1. **Event 0 (Setup):** Host places mode byte on `D[7:0]`,
//!    sets `1284Active` (`nSelectIn`) high *and* `HostBusy`
//!    (`nAutoFeed`) high.
//! 2. **Event 1 (Device Ready):** Within 35 ms the device must
//!    set `PE` low, `Select` high, `AckDataReq` (`nAck`) low —
//!    these three together prove the device understands 1284.
//!    *Failure path:* if any condition fails within 35 ms the
//!    device is **not 1284-compliant**; the negotiator surfaces
//!    `not_compliant` and runs the termination sequence.
//! 3. **Event 2 (Strobe):** Host asserts `HostClk` (`nStrobe`)
//!    low for at least 0.5 µs, then back high.  Device captures
//!    the mode byte on the rising edge.
//! 4. **Event 3 (Device Acknowledges Mode):** Within 35 ms the
//!    device must set `AckDataReq` high, `PE` high, and the
//!    `Select` line per support: high = mode supported, low =
//!    mode rejected.  *Failure path:* timeout → `not_compliant`.
//! 5. **Event 4 (Verify Support):** Host inspects `Select`.
//!    - High → mode entered; transition to Done.
//!    - Low → device rejected this mode; surface `mode_rejected`
//!      and run termination.
//! 6. **Event 5 (Host Acknowledges):** Host sets `HostBusy`
//!    (`nAutoFeed`) low to confirm entry into the new mode.
//!
//! **Termination Phase** (always run on failure or `terminate` strobe):
//!
//! 1. Host sets `HostBusy` (`nAutoFeed`) high.
//! 2. Within 35 ms device sets `PE` low + `Select` high.
//! 3. Host sets `1284Active` (`nSelectIn`) low.
//! 4. Within 35 ms device sets `PE` high + `Select` low — back to
//!    Compatibility.
//!
//! **Extended Link ID** (mode bytes `0x80`–`0xFF`): the device
//! responds during Event 3 with an *ID byte* echoed on the data
//! lines (the host samples `d_in_eli`).  V1 captures and exposes
//! it as `eli_id`.
//!
//! Composes [super::super::core::dff::DFF] and [super::super::core::constant::Constant].
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+Ieee1284Negotiator+--------+
     |                                   |
B<8> |                                   | B<8>
+--->| mode_byte                   d_out +--->
B<8> |                                   | bool
+--->| d_in_eli                  sel_in  +--->
bool |                                   | bool
+--->| start                  auto_feed  +--->
bool |                                   | bool
+--->| terminate              n_strobe   +--->
bool |                                   | bool
+--->| n_ack_in                  n_init  +--->
bool |                                   | B<8>
+--->| pe_in                    eli_id   +--->
bool |                                   | bool
+--->| select_in                  busy   +--->
bool |                                   | bool
+--->| n_fault_in                  done  +--->
     |                          timeout  +--->
     |                    not_compliant  +--->
     |                    mode_rejected  +--->
     |                       eli_valid   +--->
     +-----------------------------------+
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
    Transition { source_index: 0, target_index: 1 },   // Idle → Setup
    Transition { source_index: 1, target_index: 2 },   // Setup → WaitDeviceReady
    Transition { source_index: 2, target_index: 3 },   // WaitDeviceReady → StrobeData
    Transition { source_index: 2, target_index: 11 },  // WaitDeviceReady → NotCompliant (timeout)
    Transition { source_index: 3, target_index: 4 },   // StrobeData → WaitDeviceAck
    Transition { source_index: 4, target_index: 5 },   // WaitDeviceAck → CheckMode
    Transition { source_index: 4, target_index: 11 },  // WaitDeviceAck → NotCompliant (timeout)
    Transition { source_index: 5, target_index: 6 },   // CheckMode → CaptureEli (extended link ID)
    Transition { source_index: 5, target_index: 7 },   // CheckMode → HostAck (mode supported)
    Transition { source_index: 5, target_index: 12 },  // CheckMode → ModeRejected (Select=0)
    Transition { source_index: 6, target_index: 7 },   // CaptureEli → HostAck
    Transition { source_index: 7, target_index: 8 },   // HostAck → Done
    Transition { source_index: 8, target_index: 9 },   // Done → TerminateReq (on terminate input)
    Transition { source_index: 8, target_index: 0 },   // Done → Idle (alternative path on terminate)
    Transition { source_index: 9, target_index: 10 },  // TerminateReq → TerminateWait
    Transition { source_index: 10, target_index: 0 },  // TerminateWait → Idle (success)
    Transition { source_index: 10, target_index: 13 }, // TerminateWait → Timeout (term hang)
    Transition { source_index: 11, target_index: 9 },  // NotCompliant → TerminateReq
    Transition { source_index: 12, target_index: 9 },  // ModeRejected → TerminateReq
    Transition { source_index: 13, target_index: 0 },  // Timeout → Idle
];

/// Full negotiation + termination FSM state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum NegState {
    /// Bus idle / no negotiation in progress.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Event 0 — drive mode byte + 1284Active + HostBusy high.
    #[fsm_state(label = "setup (Event 0)")]
    Setup,
    /// Event 1 — wait ≤ 35 ms for PE↓ + Select↑ + nAck↓.
    #[fsm_state(label = "wait device ready")]
    WaitDeviceReady,
    /// Event 2 — pulse nStrobe low for `t_strobe_low` cycles.
    #[fsm_state(label = "strobe data")]
    StrobeData,
    /// Event 3 — wait ≤ 35 ms for nAck↑ + PE↑ + Select per support.
    #[fsm_state(label = "wait device ack")]
    WaitDeviceAck,
    /// Event 4 — verify Select line; route to CaptureEli, HostAck, or ModeRejected.
    #[fsm_state(label = "check mode")]
    CheckMode,
    /// Extended Link ID request (mode_byte ≥ 0x80) — capture id from D bus.
    #[fsm_state(label = "capture ELI")]
    CaptureEli,
    /// Event 5 — host sets HostBusy low to confirm mode.
    #[fsm_state(label = "host ack")]
    HostAck,
    /// Negotiation complete; in the new mode.  Hold until terminate or new start.
    #[fsm_state(label = "done")]
    Done,
    /// Termination Event 1 — host sets HostBusy high to begin return-to-Compat.
    #[fsm_state(label = "terminate req")]
    TerminateReq,
    /// Termination Event 2 — wait ≤ 35 ms for device's confirmation.
    #[fsm_state(label = "terminate wait")]
    TerminateWait,
    /// Negotiation failure: device not 1284-compliant; transition to terminate.
    #[fsm_state(label = "not compliant")]
    NotCompliant,
    /// Negotiation failure: device rejected the requested mode; transition to terminate.
    #[fsm_state(label = "mode rejected")]
    ModeRejected,
    /// Termination timed out (device misbehaving) — give up, return to Idle.
    #[fsm_state(label = "timeout")]
    Timeout,
}

/// Strobe / timeout parameters in *FPGA cycles*.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct NegTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// nStrobe low-pulse width (≥ 0.5 µs per spec).
    pub t_strobe_low: Bits<T_W>,
    /// Maximum cycles to wait at every "wait for device" step (≤ 35 ms per spec).
    pub t_response_timeout: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = NegState)]
/// IEEE 1284 mode-select negotiator with full corner-case coverage.
pub struct Ieee1284Negotiator<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<NegState>,
    tick: dff::DFF<Bits<T_W>>,
    mode_reg: dff::DFF<Bits<8>>,
    /// Latched extended-link ID (refreshed on a successful ELI negotiation).
    eli_reg: dff::DFF<Bits<8>>,
    /// True if `eli_reg` holds a freshly-captured value from the most-recent ELI request.
    eli_valid_reg: dff::DFF<bool>,
    /// True after a successful `Done` entry — drives the in-mode line states.
    in_mode: dff::DFF<bool>,
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
            eli_reg: dff::DFF::default(),
            eli_valid_reg: dff::DFF::default(),
            in_mode: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Ieee1284Negotiator].
pub struct In {
    /// Mode byte to negotiate (per IEEE 1284-1994 Table 2).  Latched at `start`.
    /// Mode bytes ≥ 0x80 are *Extended Link ID* requests.
    pub mode_byte: Bits<8>,
    /// Sampled D[7:0] from the device during ELI capture (only meaningful in CaptureEli state).
    pub d_in_eli: Bits<8>,
    /// Strobe to begin negotiation.  Ignored while busy (other than Done).
    pub start: bool,
    /// Strobe to begin the termination sequence (returns the bus to Compatibility).
    pub terminate: bool,
    /// Sampled `AckDataReq` (`nAck`) line from the device.
    pub n_ack_in: bool,
    /// Sampled `PE` (Paper End) line from the device.
    pub pe_in: bool,
    /// Sampled `Select` line — also the device's per-mode response (Select=high → mode supported).
    pub select_in: bool,
    /// Sampled `nFault` — high during normal operation; low during device-ready signaling.
    pub n_fault_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Ieee1284Negotiator].
pub struct Out {
    /// Data bus output (drives the mode byte during negotiation).
    pub d_out: Bits<8>,
    /// `1284Active` (`nSelectIn`) — driven high during the request pattern; held high in Done.
    pub sel_in: bool,
    /// `HostBusy` (`nAutoFeed`) — driven high during the request pattern; low in Done after HostAck.
    pub auto_feed: bool,
    /// `HostClk` (`nStrobe`) — active low during the StrobeData state.
    pub n_strobe: bool,
    /// `nInit` — host can pulse this for hard reset; v1 holds it high.
    pub n_init: bool,
    /// Captured Extended Link ID byte (if a request had `mode_byte ≥ 0x80`).
    pub eli_id: Bits<8>,
    /// True if `eli_id` carries a fresh value from the latest negotiation.
    pub eli_valid: bool,
    /// True while a negotiation or termination is in progress (state ≠ Idle/Done).
    pub busy: bool,
    /// Pulses for one cycle when negotiation completes successfully (state → Done).
    pub done: bool,
    /// Pulses for one cycle when termination times out (device misbehaving).
    pub timeout: bool,
    /// Pulses for one cycle when the device fails Event 1 (not 1284-compliant).
    pub not_compliant: bool,
    /// Pulses for one cycle when the device explicitly rejects the requested mode (Select=low).
    pub mode_rejected: bool,
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
    d.eli_reg = q.eli_reg;
    d.eli_valid_reg = q.eli_valid_reg;
    d.in_mode = q.in_mode;

    let mut done_pulse = false;
    let mut timeout_pulse = false;
    let mut not_compliant_pulse = false;
    let mut mode_rejected_pulse = false;

    // Helper: is the requested mode an extended link ID?
    let is_eli_request = (q.mode_reg & bits::<8>(0x80)) != bits::<8>(0);

    match q.state {
        NegState::Idle => {
            d.tick = zero_t;
            if i.start {
                d.mode_reg = i.mode_byte;
                d.eli_valid_reg = false;
                d.in_mode = false;
                d.state = NegState::Setup;
                d.tick = zero_t;
            }
        }
        NegState::Setup => {
            // Single-cycle setup; transition to wait for device ready.
            d.state = NegState::WaitDeviceReady;
            d.tick = zero_t;
        }
        NegState::WaitDeviceReady => {
            // Event 1: device must drive PE↓ + Select↑ + nAck↓ within 35 ms.
            // (n_fault_in is *informational* in v1 — Compliance requires nFault↓
            // too but real devices vary; we don't gate on it here.)
            let device_ready = !i.pe_in && i.select_in && !i.n_ack_in;
            if device_ready {
                d.state = NegState::StrobeData;
                d.tick = zero_t;
            } else if q.tick >= t.t_response_timeout {
                d.state = NegState::NotCompliant;
                d.tick = zero_t;
            }
        }
        NegState::StrobeData => {
            // Event 2: hold nStrobe low for t_strobe_low cycles.
            if q.tick >= t.t_strobe_low {
                d.state = NegState::WaitDeviceAck;
                d.tick = zero_t;
            }
        }
        NegState::WaitDeviceAck => {
            // Event 3: within 35 ms device drives nAck↑ + PE↑ + Select per support.
            let device_ack = i.n_ack_in && i.pe_in;
            if device_ack {
                d.state = NegState::CheckMode;
                d.tick = zero_t;
            } else if q.tick >= t.t_response_timeout {
                d.state = NegState::NotCompliant;
                d.tick = zero_t;
            }
        }
        NegState::CheckMode => {
            // Event 4: Select line indicates support.
            if !i.select_in {
                // Mode rejected by device.
                d.state = NegState::ModeRejected;
            } else if is_eli_request {
                // Extended-link ID request — capture the device's ID byte from D.
                d.state = NegState::CaptureEli;
            } else {
                // Standard mode supported — transition to host-ack.
                d.state = NegState::HostAck;
            }
            d.tick = zero_t;
        }
        NegState::CaptureEli => {
            // Latch the device's response on D (already in the FPGA clock domain).
            d.eli_reg = i.d_in_eli;
            d.eli_valid_reg = true;
            d.state = NegState::HostAck;
            d.tick = zero_t;
        }
        NegState::HostAck => {
            // Event 5: host releases HostBusy (auto_feed goes low) — drive
            // is implicit in the per-state output mapping below.  Transition
            // to Done.
            d.state = NegState::Done;
            d.in_mode = true;
            done_pulse = true;
            d.tick = zero_t;
        }
        NegState::Done => {
            d.tick = zero_t;
            // Stay in Done until host requests a new negotiation or termination.
            if i.terminate {
                d.state = NegState::TerminateReq;
                d.in_mode = false;
                d.tick = zero_t;
            } else if i.start {
                // Rolling negotiation: re-enter Setup with the new mode byte.
                d.mode_reg = i.mode_byte;
                d.eli_valid_reg = false;
                d.in_mode = false;
                d.state = NegState::Setup;
                d.tick = zero_t;
            }
        }
        NegState::TerminateReq => {
            // Termination Event 1 — drive HostBusy high (auto_feed high), wait
            // for device's PE↓ + Select↑ confirmation, then drop 1284Active.
            // Single-cycle setup; transition to wait.
            d.state = NegState::TerminateWait;
            d.tick = zero_t;
        }
        NegState::TerminateWait => {
            // Within 35 ms the device must drive PE↓ + Select↑ to confirm.
            let device_term_ack = !i.pe_in && i.select_in;
            if device_term_ack {
                // Termination accepted; we'll drop 1284Active in next cycle's
                // outputs (Idle holds everything low).
                d.state = NegState::Idle;
                d.tick = zero_t;
            } else if q.tick >= t.t_response_timeout {
                d.state = NegState::Timeout;
                d.tick = zero_t;
            }
        }
        NegState::NotCompliant => {
            not_compliant_pulse = true;
            // Auto-fallback: run the termination sequence to leave the bus clean.
            d.state = NegState::TerminateReq;
            d.tick = zero_t;
        }
        NegState::ModeRejected => {
            mode_rejected_pulse = true;
            d.state = NegState::TerminateReq;
            d.tick = zero_t;
        }
        NegState::Timeout => {
            timeout_pulse = true;
            d.state = NegState::Idle;
            d.tick = zero_t;
        }
    }

    if cr.reset.any() {
        d.state = NegState::Idle;
        d.tick = zero_t;
        d.mode_reg = bits::<8>(0);
        d.eli_reg = bits::<8>(0);
        d.eli_valid_reg = false;
        d.in_mode = false;
    }

    let busy = match q.state {
        NegState::Idle => false,
        NegState::Done => false,
        _ => true,
    };

    // 1284Active (sel_in) — high during all active-negotiation states; high
    // in Done (we're "actively in 1284 mode"); low in Idle / Timeout.
    let sel_in_active = match q.state {
        NegState::Setup
        | NegState::WaitDeviceReady
        | NegState::StrobeData
        | NegState::WaitDeviceAck
        | NegState::CheckMode
        | NegState::CaptureEli
        | NegState::HostAck
        | NegState::TerminateReq
        | NegState::TerminateWait => true,
        NegState::Done => true, // remain in 1284 mode until termination
        _ => false,
    };

    // HostBusy (auto_feed):
    //   - High during Setup → WaitDeviceAck (request pattern)
    //   - Low after HostAck → during Done (mode entered)
    //   - High during TerminateReq / TerminateWait (signal termination)
    //   - Low otherwise
    let auto_feed_active = match q.state {
        NegState::Setup
        | NegState::WaitDeviceReady
        | NegState::StrobeData
        | NegState::WaitDeviceAck
        | NegState::CheckMode
        | NegState::CaptureEli
        | NegState::TerminateReq
        | NegState::TerminateWait => true,
        _ => false,
    };
    let strobe_active = q.state == NegState::StrobeData;

    let mut o = Out::dont_care();
    o.d_out = q.mode_reg;
    o.sel_in = sel_in_active;
    o.auto_feed = auto_feed_active;
    o.n_strobe = !strobe_active;
    o.n_init = true; // held high in v1 — the host pulses this externally for hard reset
    o.eli_id = q.eli_reg;
    o.eli_valid = q.eli_valid_reg;
    o.busy = busy;
    o.done = done_pulse;
    o.timeout = timeout_pulse;
    o.not_compliant = not_compliant_pulse;
    o.mode_rejected = mode_rejected_pulse;
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
            d_in_eli: bits(0),
            start: false,
            terminate: false,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        }
    }

    /// A *cooperating* device: responds with all the right line transitions
    /// at the right times.  Returns the input stream for one full negotiation
    /// (mode acknowledged + host-ack + done).
    fn cooperating_device(mode: u8, eli_response: u8) -> Vec<In> {
        let mut out: Vec<In> = vec![In {
            mode_byte: bits(mode as u128),
            d_in_eli: bits(0),
            start: true,
            terminate: false,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        }];
        // After Setup → WaitDeviceReady, device responds (PE↓ + Select↑ + nAck↓)
        // after a few cycles.
        for _ in 0..5 {
            out.push(idle_in());
        }
        // Device ready response.
        for _ in 0..3 {
            out.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(0),
                start: false,
                terminate: false,
                n_ack_in: false,
                pe_in: false,
                select_in: true,
                n_fault_in: false,
            });
        }
        // After StrobeData → WaitDeviceAck, device acks (nAck↑ + PE↑ +
        // Select↑ if mode supported).  Provide several "still working" cycles
        // before the ack.
        for _ in 0..5 {
            out.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(0),
                start: false,
                terminate: false,
                n_ack_in: false,
                pe_in: false,
                select_in: true,
                n_fault_in: false,
            });
        }
        // Device acknowledges mode supported.  d_in_eli only matters for ELI request.
        for _ in 0..6 {
            out.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(eli_response as u128),
                start: false,
                terminate: false,
                n_ack_in: true,
                pe_in: true,
                select_in: true,
                n_fault_in: true,
            });
        }
        // After Done — keep device in steady state.
        for _ in 0..6 {
            out.push(idle_in());
        }
        out
    }

    /// A device that rejects a mode (Select stays low at Event 4).
    fn rejecting_device(mode: u8) -> Vec<In> {
        let mut out: Vec<In> = vec![In {
            mode_byte: bits(mode as u128),
            d_in_eli: bits(0),
            start: true,
            terminate: false,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        }];
        for _ in 0..5 {
            out.push(idle_in());
        }
        // Device ready (Event 1).
        for _ in 0..3 {
            out.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(0),
                start: false,
                terminate: false,
                n_ack_in: false,
                pe_in: false,
                select_in: true,
                n_fault_in: false,
            });
        }
        for _ in 0..5 {
            out.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(0),
                start: false,
                terminate: false,
                n_ack_in: false,
                pe_in: false,
                select_in: true,
                n_fault_in: false,
            });
        }
        // Device acks but Select=low → mode rejected.
        for _ in 0..6 {
            out.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(0),
                start: false,
                terminate: false,
                n_ack_in: true,
                pe_in: true,
                select_in: false, // rejection
                n_fault_in: true,
            });
        }
        // After ModeRejected the FSM auto-runs termination — the device
        // confirms termination by PE↓ + Select↑.
        for _ in 0..3 {
            out.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(0),
                start: false,
                terminate: false,
                n_ack_in: true,
                pe_in: false,
                select_in: true,
                n_fault_in: true,
            });
        }
        for _ in 0..6 {
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
        let stream_in = cooperating_device(0xC0, 0);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.done), "no done pulse");
        assert!(!outputs.iter().any(|s| s.output.not_compliant));
        assert!(!outputs.iter().any(|s| s.output.mode_rejected));
        Ok(())
    }

    /// Device-not-compliant corner case: device fails to respond to Event 1
    /// within the response timeout.
    #[test]
    fn test_not_compliant_corner_case() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        // Device leaves all lines at default — never asserts the ready pattern.
        let mut stream_in: Vec<In> = vec![In {
            mode_byte: bits(0xC0),
            d_in_eli: bits(0),
            start: true,
            terminate: false,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        }];
        for _ in 0..120 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.not_compliant));
        // After auto-fallback termination times out (no device confirmation),
        // we should also see a `timeout` pulse.
        assert!(outputs.iter().any(|s| s.output.timeout));
        // Should NOT see done.
        assert!(!outputs.iter().any(|s| s.output.done));
        Ok(())
    }

    /// Mode-rejected corner case: device responds OK but Select=low at Event 4.
    #[test]
    fn test_mode_rejected_corner_case() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let stream_in = rejecting_device(0xC0);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.mode_rejected));
        assert!(!outputs.iter().any(|s| s.output.done));
        Ok(())
    }

    /// Extended Link ID corner case: mode_byte ≥ 0x80 → device responds
    /// with an ID byte that we capture into eli_id.
    #[test]
    fn test_eli_corner_case() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        // ELI request 0x80 + something; device echoes 0x42 as its ID.
        let stream_in = cooperating_device(0x80, 0x42);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_idx = outputs.iter().position(|s| s.output.done).unwrap();
        // Right at done, eli_valid + eli_id should be set.
        let post_done = &outputs[done_idx];
        assert!(post_done.output.eli_valid);
        assert_eq!(post_done.output.eli_id.raw(), 0x42);
        Ok(())
    }

    /// Termination corner case: after a successful negotiation, the host
    /// strobes `terminate` and the FSM walks the termination handshake.
    #[test]
    fn test_termination_after_done() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let mut stream_in = cooperating_device(0xC0, 0);
        // Strobe terminate after Done.
        stream_in.push(In {
            mode_byte: bits(0),
            d_in_eli: bits(0),
            start: false,
            terminate: true,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        });
        // Device responds to termination after a few cycles.
        for _ in 0..3 {
            stream_in.push(idle_in());
        }
        for _ in 0..6 {
            stream_in.push(In {
                mode_byte: bits(0),
                d_in_eli: bits(0),
                start: false,
                terminate: false,
                n_ack_in: true,
                pe_in: false,
                select_in: true,
                n_fault_in: true,
            });
        }
        for _ in 0..10 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Final state: no busy, no sel_in (back to Compatibility / Idle).
        let last = outputs.last().unwrap();
        assert!(!last.output.busy);
        assert!(!last.output.sel_in);
        assert!(!last.output.auto_feed);
        Ok(())
    }

    /// Termination-timeout corner case: device fails to confirm termination
    /// within the response window → Timeout state pulses `timeout`.
    #[test]
    fn test_termination_timeout_corner_case() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let mut stream_in = cooperating_device(0xC0, 0);
        // Strobe terminate but never confirm.
        stream_in.push(In {
            mode_byte: bits(0),
            d_in_eli: bits(0),
            start: false,
            terminate: true,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        });
        for _ in 0..120 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.timeout));
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["22060"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_ieee1284_negotiator_hdl_works() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let stream_in = cooperating_device(0xC0, 0);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ieee1284_negotiator_trace() -> miette::Result<()> {
        let uut = Ieee1284Negotiator::<8>::new(test_timings());
        let stream_in = cooperating_device(0xC0, 0);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ieee1284_negotiator");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["dec6b32c725f99b88c50888b39c1e9d750f6c6e1d406f73a5ea976e03e218c04"];
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
        assert_eq!(desc.variants().len(), 14);
        assert_eq!(desc.initial_index(), 0);
    }
}
