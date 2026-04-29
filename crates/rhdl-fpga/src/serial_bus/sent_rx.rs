//! SENT receiver — framing-helper v1
//!
//! Single Edge Nibble Transmission (SAE J2716) is an automotive
//! single-wire sensor interface for absolute-position, pressure,
//! and temperature sensors in modern engines and chassis systems.
//! The wire is asynchronous and unidirectional: the sensor pulse-
//! width-modulates successive 4-bit nibbles, with each nibble
//! delimited by a falling edge.  No clock signal accompanies the
//! data — the receiver auto-calibrates the SENT "tick" period
//! from a 56-tick **sync pulse** at the start of each frame.
//!
//! **v1 scope: framing helper only.**  This widget finds frame
//! boundaries (sync pulses) and emits per-nibble timing
//! measurements; **the host decodes the nibble value** from each
//! period (`nibble = (period / tick_period) - 12`).  The trade-off
//! is intentional — in-kernel division would either need a
//! non-trivial 28-deep iterative-subtract cascade or a 16-element
//! threshold-lookup table; either is straightforward but adds
//! complexity that isn't needed for the framing-helper use case
//! (e.g., a soft-CPU running a small SENT decoder in firmware can
//! easily do the decode).
//!
//! v2 follow-ups: in-kernel nibble decode, CRC-4 validation
//! (polynomial `0x1D`), tick-period auto-calibration via a moving
//! average over the sync pulse, optional pause-pulse detection,
//! short-frame / long-frame format selection.
//!
//! A SENT frame:
//!
//! | Field          | Length          | What it carries                          |
//! |----------------|-----------------|------------------------------------------|
//! | Sync           | 56 ticks        | calibration (sets the tick period)       |
//! | Status nibble  | 12..27 ticks    | 4 status / slow-channel bits             |
//! | Data nibbles   | 6 × (12..27)    | 24 bits of sensor data, MSB-first        |
//! | CRC nibble     | 12..27 ticks    | 4-bit CRC (polynomial `0x1D`)            |
//! | Pause (opt'l)  | varies          | spacing before the next frame            |
//!
//! 8 total nibbles after sync (status + 6 data + CRC).
//!
//! The line idles **high**.  Each nibble starts on a falling edge,
//! is held low for at least 5 ticks, then released high; the
//! receiver measures the time from one falling edge to the next.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+SentRx+-------+
     |                      |
bool |                      | B<T_W>
+--->| sent_in    last_period+>
     |                      | B<4>
     |                  nibble_idx+>
     |                      | bool
     |                  frame_strobe+>
     |                      | bool
     |                  nibble_strobe+>
     |                      | bool
     |                  valid+--->
     |                       | bool
     |                   busy+--->
     +----------------------+
")]
//!
//!# Internals
//!
//! Edge-driven FSM that measures the period between successive
//! falling edges of `sent_in`.  Long periods (≥ `t_sync_min`) are
//! sync pulses that begin a new frame; short periods (in
//! `[t_nibble_min, t_nibble_max]`) are data nibbles.  Anything
//! else aborts the frame.
//!
//! - `Idle`: line high, looking for the first sync pulse.  A
//!   falling edge starts measuring; the period is decided at the
//!   next falling edge.
//! - `Hunting`: between falling edges; tick counter ticks up.
//!   On the next falling edge, classify the period:
//!   - `t_sync_min ≤ period`: sync pulse → emit `frame_strobe`,
//!     reset `nibble_idx`, → `Collecting`.
//!   - `t_nibble_min ≤ period ≤ t_nibble_max` while in
//!     `Collecting`: nibble period → emit `nibble_strobe` with
//!     `last_period` and `nibble_idx`; advance `nibble_idx`.
//!     After the 8th nibble, emit `valid` and return to `Idle`.
//!   - anything else: abort → `Idle`.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/sent_rx.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/sent_rx.md")]
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// State machine.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default)]
pub enum SentState {
    /// Looking for the first sync pulse.
    #[default]
    Idle,
    /// Sync seen; collecting the 8 nibbles that follow.
    Collecting,
}

/// Bus-timing parameters, all in *FPGA cycles*.
///
/// At a 100 MHz clock with a 3 µs SENT tick, typical values are:
///
/// | Field          | Cycles            | Note                              |
/// |----------------|-------------------|-----------------------------------|
/// | `t_nibble_min` | `12 * tick`       | shortest valid nibble period      |
/// | `t_nibble_max` | `27 * tick`       | longest valid nibble period       |
/// | `t_sync_min`   | `~50 * tick`      | minimum period to count as sync   |
/// | `t_sync_max`   | `~62 * tick`      | maximum period for a sync pulse   |
///
/// The host pre-computes these from `tick_period_us * fpga_clock_hz / 1_000_000`.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct SentTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Shortest valid nibble period (typically `12 * tick_period`).
    pub t_nibble_min: Bits<T_W>,
    /// Longest valid nibble period (typically `27 * tick_period`).
    pub t_nibble_max: Bits<T_W>,
    /// Minimum period to be classified as a sync pulse.
    pub t_sync_min: Bits<T_W>,
    /// Maximum period for a sync pulse (anything longer is treated as a fault).
    pub t_sync_max: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// SENT receiver (framing-helper v1).
pub struct SentRx<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<SentState>,
    /// Tick counter — measures FPGA cycles since the last falling edge.
    tick: dff::DFF<Bits<T_W>>,
    /// Previous-cycle value of `sent_in` (for edge detection).
    prev_in: dff::DFF<bool>,
    /// 0..7 — index of the nibble within the current frame.
    nibble_idx: dff::DFF<Bits<4>>,
    /// Period (FPGA cycles) of the most-recently-completed inter-edge interval.
    last_period: dff::DFF<Bits<T_W>>,
    /// One-cycle frame_strobe pulse.
    frame_strobe: dff::DFF<bool>,
    /// One-cycle nibble_strobe pulse.
    nibble_strobe: dff::DFF<bool>,
    /// One-cycle valid pulse (after the 8th nibble of a frame).
    valid_pulse: dff::DFF<bool>,
    timings: Constant<SentTimings<T_W>>,
}

impl<const T_W: usize> SentRx<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Create a SENT receiver with the given timings (in FPGA cycles).
    pub fn new(timings: SentTimings<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            prev_in: dff::DFF::new(true),
            nibble_idx: dff::DFF::default(),
            last_period: dff::DFF::default(),
            frame_strobe: dff::DFF::default(),
            nibble_strobe: dff::DFF::default(),
            valid_pulse: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [SentRx].
pub struct In {
    /// SENT wire input.  Idles high; falling edges delimit nibbles.
    pub sent_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [SentRx].
pub struct Out<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Period in FPGA cycles of the most-recent inter-edge interval.
    /// Valid in the cycle that `nibble_strobe` or `frame_strobe` pulses.
    pub last_period: Bits<T_W>,
    /// 0..7 — index of the nibble that `nibble_strobe` corresponds to.
    pub nibble_idx: Bits<4>,
    /// Pulses for one cycle when a sync pulse has been detected
    /// (start of a frame).
    pub frame_strobe: bool,
    /// Pulses for one cycle when a nibble period has been measured.
    pub nibble_strobe: bool,
    /// Pulses for one cycle after the 8th nibble of a complete frame.
    pub valid: bool,
    /// `true` while between sync and end-of-frame.
    pub busy: bool,
}

impl<const T_W: usize> SynchronousIO for SentRx<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out<T_W>;
    type Kernel = sent_rx<T_W>;
}

#[kernel]
/// Kernel for [SentRx].
pub fn sent_rx<const T_W: usize>(cr: ClockReset, i: In, q: Q<T_W>) -> (Out<T_W>, D<T_W>)
where
    rhdl::bits::W<T_W>: BitWidth,
{
    let one_t: Bits<T_W> = bits::<T_W>(1);
    let zero_t: Bits<T_W> = bits::<T_W>(0);
    let one_b4: Bits<4> = bits::<4>(1);
    let zero_b4: Bits<4> = bits::<4>(0);

    let t = q.timings;

    let mut d = D::<T_W>::dont_care();
    d.state = q.state;
    d.tick = q.tick + one_t;
    d.prev_in = i.sent_in;
    d.nibble_idx = q.nibble_idx;
    d.last_period = q.last_period;
    d.frame_strobe = false;
    d.nibble_strobe = false;
    d.valid_pulse = false;

    let falling = q.prev_in && !i.sent_in;

    if falling {
        // Period is the cycle count from the previous falling edge to this
        // one.  `q.tick` was reset to 0 on the previous falling edge and
        // increments each cycle, so it under-reads by 1; add it back here.
        let period = q.tick + one_t;
        d.last_period = period;
        d.tick = zero_t;

        let is_sync = period >= t.t_sync_min && period <= t.t_sync_max;
        let is_nibble = period >= t.t_nibble_min && period <= t.t_nibble_max;

        if is_sync {
            d.frame_strobe = true;
            d.nibble_idx = zero_b4;
            d.state = SentState::Collecting;
        } else if is_nibble && q.state == SentState::Collecting {
            d.nibble_strobe = true;
            let next_idx = q.nibble_idx + one_b4;
            if next_idx == bits::<4>(8) {
                // 8th nibble of frame: emit valid and return to Idle.
                d.valid_pulse = true;
                d.nibble_idx = zero_b4;
                d.state = SentState::Idle;
            } else {
                d.nibble_idx = next_idx;
            }
        } else {
            // Out-of-range or unexpected period — abandon frame.
            d.state = SentState::Idle;
            d.nibble_idx = zero_b4;
        }
    }

    if cr.reset.any() {
        d.state = SentState::Idle;
        d.tick = zero_t;
        d.prev_in = true;
        d.nibble_idx = zero_b4;
        d.last_period = zero_t;
        d.frame_strobe = false;
        d.nibble_strobe = false;
        d.valid_pulse = false;
    }

    let mut o = Out::<T_W>::dont_care();
    o.last_period = q.last_period;
    o.nibble_idx = q.nibble_idx;
    o.frame_strobe = q.frame_strobe;
    o.nibble_strobe = q.nibble_strobe;
    o.valid = q.valid_pulse;
    o.busy = q.state != SentState::Idle;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    /// Compact test timings — 1 SENT tick = 4 FPGA cycles.  Standard
    /// SENT ratios preserved (sync = 56 ticks, nibble range 12..27).
    fn test_timings() -> SentTimings<10> {
        SentTimings {
            t_nibble_min: bits(12 * 4),
            t_nibble_max: bits(27 * 4),
            t_sync_min: bits(50 * 4),
            t_sync_max: bits(62 * 4),
        }
    }

    fn idle_in() -> In {
        In { sent_in: true }
    }

    /// Drive the FSM with a hand-crafted SENT frame: sync (56 ticks)
    /// followed by 8 nibble pulses with the given nibble values.
    /// Each pulse: 5 ticks low + (12 + N - 5) ticks high = (12 + N) ticks total.
    fn sent_waveform(nibbles: [u8; 8]) -> Vec<In> {
        let tick_cycles: u32 = 4;
        let mut v = Vec::new();
        let push_low = |v: &mut Vec<In>, n: u32| {
            for _ in 0..n {
                v.push(In { sent_in: false });
            }
        };
        let push_high = |v: &mut Vec<In>, n: u32| {
            for _ in 0..n {
                v.push(In { sent_in: true });
            }
        };
        // Initial idle-high to settle prev_in.
        push_high(&mut v, 16);
        // Sync pulse: 5 low + 51 high = 56 ticks.
        push_low(&mut v, 5 * tick_cycles);
        push_high(&mut v, 51 * tick_cycles);
        // Each nibble pulse: 5 low + (12 + N - 5) high.
        for n in nibbles {
            let total = 12 + n as u32;
            push_low(&mut v, 5 * tick_cycles);
            push_high(&mut v, (total - 5) * tick_cycles);
        }
        // Trailing falling edge so the 8th nibble's measurement completes.
        push_low(&mut v, 5 * tick_cycles);
        push_high(&mut v, 80);
        v
    }

    #[test]
    fn test_idle_emits_no_pulses() -> miette::Result<()> {
        let uut = SentRx::<10>::new(test_timings());
        let stream = std::iter::repeat_n(idle_in(), 64)
            .with_reset(1)
            .clock_pos_edge(100);
        let any = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.frame_strobe || s.output.nibble_strobe || s.output.valid);
        assert!(!any, "idle should not emit any strobes");
        Ok(())
    }

    #[test]
    fn test_full_frame() -> miette::Result<()> {
        let uut = SentRx::<10>::new(test_timings());
        let stream_in = sent_waveform([0, 1, 2, 3, 4, 5, 6, 7]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let frame_count = outputs.iter().filter(|s| s.output.frame_strobe).count();
        let nibble_count = outputs.iter().filter(|s| s.output.nibble_strobe).count();
        let valid_count = outputs.iter().filter(|s| s.output.valid).count();
        assert!(
            frame_count >= 1 && nibble_count == 8,
            "frame={frame_count}, nibble={nibble_count}, valid={valid_count}: expected frame≥1, nibble=8"
        );
        let valid_count = outputs.iter().filter(|s| s.output.valid).count();
        assert_eq!(valid_count, 1, "expected exactly one valid pulse");
        Ok(())
    }

    #[test]
    fn test_nibble_periods_match_input() -> miette::Result<()> {
        // Each nibble's measured period should match (12 + N) * tick_cycles.
        // tick_cycles = 4, so nibble N → period 4 * (12 + N).
        let uut = SentRx::<10>::new(test_timings());
        let nibbles: [u8; 8] = [0, 5, 10, 15, 3, 8, 12, 7];
        let stream_in = sent_waveform(nibbles);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let nibble_strobes: Vec<_> = outputs
            .iter()
            .filter(|s| s.output.nibble_strobe)
            .map(|s| s.output.last_period.raw())
            .collect();
        assert_eq!(nibble_strobes.len(), 8);
        for (i, &period) in nibble_strobes.iter().enumerate() {
            let expected = 4 * (12 + nibbles[i] as u128);
            assert_eq!(
                period, expected,
                "nibble {i} period mismatch: got {period}, expected {expected}"
            );
        }
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check.
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = SentRx::<10>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["10563"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip.
    #[test]
    fn test_sent_rx_hdl_works() -> miette::Result<()> {
        let uut = SentRx::<10>::new(test_timings());
        let stream_in = sent_waveform([0, 1, 2, 3, 4, 5, 6, 7]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest.
    #[test]
    fn test_sent_rx_trace() -> miette::Result<()> {
        let uut = SentRx::<10>::new(test_timings());
        let stream_in = sent_waveform([0, 1, 2, 3, 4, 5, 6, 7]);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("sent_rx");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["fb8fce095832ec7c866b26f306e1ae1797ff1c34ec11cd5ed8ce5c6a483ae546"];
        let digest = vcd.dump_to_file(root.join("sent_rx.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
