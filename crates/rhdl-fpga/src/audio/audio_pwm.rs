//! Stereo PWM audio output (v1: naive PWM, no sigma-delta)
//!
//! Two parallel [super::pwm::PwmGenerator] channels driven by a
//! shared sample-rate clock divider.  The host feeds in the next
//! sample on each `sample_request` pulse; the widget latches and
//! holds it as the PWM duty until the next sample period.  An
//! external RC low-pass filter on each output pin recovers the
//! analog audio waveform.
//!
//! Sample format is **unsigned offset binary** in `Bits<N>`.  For
//! signed audio (e.g., 16-bit `int16`), XOR the high bit before
//! handing the sample to this widget — that converts twos-complement
//! to offset binary cleanly.
//!
//! Sigma-delta noise-shaping is intentionally deferred to v2.  The
//! naive PWM here is good for ~5–6 effective bits at moderate
//! carrier rates (good enough for hobbyist audio); for near-CD
//! quality, wrap with a 1st- or 2nd-order sigma-delta modulator
//! upstream of `next_sample_*`.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+StereoAudioPwm+-----+
     |                          |
B<N> |                          | bool
+--->| next_left   pwm_left     +--->
B<N> |                          | bool
+--->| next_right  pwm_right    +--->
bool |                          | bool
+--->| sample_valid sample_req  +--->
     +--------------------------+
")]
//!
//!# Internals
//!
//! - Two [PwmGenerator]s, one per channel, sharing the FPGA clock
//!   as the carrier.
//! - A `sample_counter` ticks each FPGA cycle; on
//!   `sample_period - 1` it wraps and pulses `sample_request`.
//! - On the same cycle, if `sample_valid` is high, the new
//!   `next_left` / `next_right` values are latched as the PWM
//!   duties for the next sample period.
//!
//!# Behavior
//!
//! - Carrier frequency = `f_clk / 2^N` (the PWM period inside each
//!   channel).
//! - Sample frequency = `f_clk / sample_period`.
//! - Pick `sample_period >> 2^N` so the PWM has many full carrier
//!   periods to integrate within a single sample.
//!
//!# Parameters
//!
//! - `N` — bit width of the PWM duty (= sample width)
//! - `RATE_W` — bit width of the sample-rate counter
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/audio_pwm.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/audio_pwm.md")]
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff, pwm::PwmGenerator};

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// Stereo PWM audio output core.
pub struct StereoAudioPwm<const N: usize, const RATE_W: usize>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<RATE_W>: BitWidth,
{
    pwm_left: PwmGenerator<N>,
    pwm_right: PwmGenerator<N>,
    sample_counter: dff::DFF<Bits<RATE_W>>,
    sample_left: dff::DFF<Bits<N>>,
    sample_right: dff::DFF<Bits<N>>,
    sample_period: Constant<Bits<RATE_W>>,
}

impl<const N: usize, const RATE_W: usize> StereoAudioPwm<N, RATE_W>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<RATE_W>: BitWidth,
{
    /// Create a stereo PWM audio output.  `sample_period` is the
    /// number of FPGA cycles between sample updates (i.e.,
    /// `f_clk / desired_sample_rate`).
    pub fn new(sample_period: Bits<RATE_W>) -> Self {
        Self {
            pwm_left: PwmGenerator::default(),
            pwm_right: PwmGenerator::default(),
            sample_counter: dff::DFF::default(),
            sample_left: dff::DFF::default(),
            sample_right: dff::DFF::default(),
            sample_period: Constant::new(sample_period),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [StereoAudioPwm].
pub struct In<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Next left-channel sample (latched on `sample_request` cycle if `sample_valid`).
    pub next_left: Bits<N>,
    /// Next right-channel sample.
    pub next_right: Bits<N>,
    /// True when the `next_*` inputs hold a valid fresh sample.
    pub sample_valid: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [StereoAudioPwm].
pub struct Out {
    /// Left-channel PWM output (apply external RC low-pass to recover analog).
    pub pwm_left: bool,
    /// Right-channel PWM output.
    pub pwm_right: bool,
    /// One-cycle pulse when the widget is ready to latch the next sample.
    pub sample_request: bool,
}

impl<const N: usize, const RATE_W: usize> SynchronousIO for StereoAudioPwm<N, RATE_W>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<RATE_W>: BitWidth,
{
    type I = In<N>;
    type O = Out;
    type Kernel = stereo_audio_pwm<N, RATE_W>;
}

#[kernel]
/// Kernel for [StereoAudioPwm].
pub fn stereo_audio_pwm<const N: usize, const RATE_W: usize>(
    cr: ClockReset,
    i: In<N>,
    q: Q<N, RATE_W>,
) -> (Out, D<N, RATE_W>)
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<RATE_W>: BitWidth,
{
    let one_rate: Bits<RATE_W> = bits::<RATE_W>(1);
    let zero_rate: Bits<RATE_W> = bits::<RATE_W>(0);
    let zero_n: Bits<N> = bits::<N>(0);

    let mut d = D::<N, RATE_W>::dont_care();
    // Sample-rate divider: count up; pulse + reset at period boundary.
    let sample_tick = q.sample_counter == (q.sample_period - one_rate);
    d.sample_counter = if sample_tick {
        zero_rate
    } else {
        q.sample_counter + one_rate
    };
    // Latch new samples on tick (when host signals valid).
    d.sample_left = if sample_tick && i.sample_valid {
        i.next_left
    } else {
        q.sample_left
    };
    d.sample_right = if sample_tick && i.sample_valid {
        i.next_right
    } else {
        q.sample_right
    };
    // Drive the two PWMs from the latched samples.
    d.pwm_left = q.sample_left;
    d.pwm_right = q.sample_right;

    if cr.reset.any() {
        d.sample_counter = zero_rate;
        d.sample_left = zero_n;
        d.sample_right = zero_n;
    }

    let mut o = Out::dont_care();
    o.pwm_left = q.pwm_left;
    o.pwm_right = q.pwm_right;
    o.sample_request = sample_tick;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In<4> {
        In {
            next_left: bits(0),
            next_right: bits(0),
            sample_valid: false,
        }
    }

    // Tier 2 — sample tick fires at the right cadence.
    #[test]
    fn test_sample_tick_cadence() -> miette::Result<()> {
        let sample_period = 7u128;
        let uut = StereoAudioPwm::<4, 4>::new(bits(sample_period));
        let n_cycles = 30usize;
        let stream = std::iter::repeat_n(idle_in(), n_cycles)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .map(|s| s.output.sample_request)
            .collect();
        // Sample request should pulse every `sample_period` cycles.
        let pulse_count = outputs.iter().filter(|x| **x).count();
        // We have n_cycles non-reset cycles; expected pulses ≈ n_cycles / sample_period.
        let expected_min = (n_cycles as u128 / sample_period - 1) as usize;
        let expected_max = (n_cycles as u128 / sample_period + 1) as usize;
        assert!(
            pulse_count >= expected_min && pulse_count <= expected_max,
            "expected ~{} pulses, got {}",
            n_cycles / sample_period as usize,
            pulse_count
        );
        Ok(())
    }

    // Tier 2 — sample latching: when sample_valid pulses with the request,
    // the channel duty changes for the next sample period.
    #[test]
    fn test_sample_latch_changes_duty() -> miette::Result<()> {
        let sample_period = 8u128;
        let uut = StereoAudioPwm::<4, 4>::new(bits(sample_period));
        let n_cycles = 60usize;
        // For the first half: no fresh samples.  For the second half:
        // continuously offer a high duty (15) for left, low (1) for right,
        // with sample_valid=true.
        let mut stream_in: Vec<In<4>> = Vec::with_capacity(n_cycles);
        for cycle in 0..n_cycles {
            let mut inp = idle_in();
            if cycle >= n_cycles / 2 {
                inp.next_left = bits(15);
                inp.next_right = bits(1);
                inp.sample_valid = true;
            }
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // After the second-half samples have latched, the left PWM should
        // be high almost-always (duty 15/16) and right should be low almost-always.
        // Look at the last 16 samples (one full PWM period).
        let last16 = &outputs[outputs.len() - 16..];
        let left_high = last16.iter().filter(|s| s.output.pwm_left).count();
        let right_high = last16.iter().filter(|s| s.output.pwm_right).count();
        assert!(
            left_high >= 14,
            "left should be high most of the time, got {left_high}/16"
        );
        assert!(
            right_high <= 2,
            "right should be low most of the time, got {right_high}/16"
        );
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = StereoAudioPwm::<4, 4>::new(bits(8));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8003"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_audio_pwm_hdl_works() -> miette::Result<()> {
        let uut = StereoAudioPwm::<4, 4>::new(bits(8));
        let mut stream_in: Vec<In<4>> = Vec::new();
        for cycle in 0..40 {
            let mut inp = idle_in();
            if cycle >= 20 {
                inp.next_left = bits(12);
                inp.next_right = bits(4);
                inp.sample_valid = true;
            }
            stream_in.push(inp);
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
    fn test_audio_pwm_trace() -> miette::Result<()> {
        let uut = StereoAudioPwm::<4, 4>::new(bits(8));
        let mut stream_in: Vec<In<4>> = Vec::new();
        for cycle in 0..60 {
            let mut inp = idle_in();
            if cycle >= 20 {
                inp.next_left = bits(12);
                inp.next_right = bits(4);
                inp.sample_valid = true;
            }
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("audio_pwm");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["329bb2b69035c10aa40b9a863bd87448c03559336d8f4d0b657d76e740deff37"];
        let digest = vcd.dump_to_file(root.join("audio_pwm.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
