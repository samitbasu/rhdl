//! DTMF (Dual-Tone Multi-Frequency) generator
//!
//! DTMF is the in-band touch-tone signalling system every wireline
//! telephone has used since 1963.  Each key on the keypad
//! corresponds to a *pair* of sinusoidal tones — one from the
//! *row* group (697, 770, 852, 941 Hz) and one from the *column*
//! group (1209, 1336, 1477, 1633 Hz).  The receiver demultiplexes
//! the key by detecting the two frequencies via Goertzel filters
//! (a job for [`super::audio_pwm`]'s sister widget, deferred to
//! v2).
//!
//! **v1 scope:** *square-wave* generator at the row and column
//! frequencies, summed to produce a 4-level staircase output.
//! The frequency content is correct DTMF — the row and column
//! tones land at exactly the spec-defined frequencies — but the
//! waveform is not a true sine; the harmonics ride above the
//! voiceband (4 kHz) and either an analog low-pass filter or a
//! sigma-delta DAC ([`super::audio_pwm::AudioPwm`]) is the
//! intended downstream consumer.  A true sine via small lookup
//! table is a v2 enhancement.
//!
//! The frequency control inputs `phase_inc_row` and
//! `phase_inc_col` are the *phase increment per audio sample* —
//! i.e., `freq_hz × 2^N / sample_rate_hz` rounded to the nearest
//! integer, where `N` is the phase-accumulator width.  The host
//! pre-computes these from the desired tone and the chosen
//! sample rate.  At a 48 kHz sample rate with N = 16:
//!
//! | Key |   Row Hz / inc_row | Col Hz / inc_col |
//! |-----|---------------------|------------------|
//! |  1  |   697 / 952         | 1209 / 1651      |
//! |  2  |   697 / 952         | 1336 / 1825      |
//! |  3  |   697 / 952         | 1477 / 2017      |
//! |  4  |   770 / 1052        | 1209 / 1651      |
//! | ... |   ...               | ...              |
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +--------+DtmfGenerator+---------+
     |                                |
B<N> |                                | B<2>
+--->| phase_inc_row    sample_out    +--->
B<N> |                                |
+--->| phase_inc_col                  |
bool |                                |
+--->| enable                         |
     +--------------------------------+
")]
//!
//!# Internals
//!
//! Two independent phase accumulators (row, col), each `N` bits.
//! Per audio sample, each accumulator advances by its respective
//! `phase_inc`, wrapping naturally on overflow.  The MSB of each
//! accumulator is the square-wave output at that tone's frequency;
//! the two MSBs are summed as `Bits<2>` and exposed as
//! `sample_out` (values 0..3 — a 4-level staircase that the host
//! treats as a 2-bit signed audio sample).
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/dtmf_generator.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/dtmf_generator.md")]
use rhdl::prelude::*;

use crate::core::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
/// DTMF dual-tone generator.
pub struct DtmfGenerator<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Row-tone phase accumulator.
    row_phase: dff::DFF<Bits<N>>,
    /// Column-tone phase accumulator.
    col_phase: dff::DFF<Bits<N>>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [DtmfGenerator].
pub struct In<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Row-tone phase increment per audio sample.
    pub phase_inc_row: Bits<N>,
    /// Column-tone phase increment per audio sample.
    pub phase_inc_col: Bits<N>,
    /// Strobe `enable` once per *audio sample tick* — between strobes
    /// the accumulators hold.  This decouples the widget from any
    /// specific FPGA clock rate; the host's audio-clock divider drives
    /// it.
    pub enable: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [DtmfGenerator].
pub struct Out {
    /// 2-bit audio sample (sum of the two MSBs — values 0, 1, or 2).
    pub sample_out: Bits<2>,
}

impl<const N: usize> SynchronousIO for DtmfGenerator<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<N>;
    type O = Out;
    type Kernel = dtmf_generator<N>;
}

#[kernel]
/// Kernel for [DtmfGenerator].
pub fn dtmf_generator<const N: usize>(cr: ClockReset, i: In<N>, q: Q<N>) -> (Out, D<N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<N>::dont_care();
    d.row_phase = q.row_phase;
    d.col_phase = q.col_phase;

    if i.enable {
        d.row_phase = q.row_phase + i.phase_inc_row;
        d.col_phase = q.col_phase + i.phase_inc_col;
    }

    if cr.reset.any() {
        d.row_phase = bits::<N>(0);
        d.col_phase = bits::<N>(0);
    }

    // Extract MSBs as 0/1 bits; sum into 2-bit output.
    let n_minus_1: u128 = (N as u128) - 1;
    let row_msb_bits = (q.row_phase >> n_minus_1) & bits::<N>(1);
    let col_msb_bits = (q.col_phase >> n_minus_1) & bits::<N>(1);
    let row_msb: Bits<2> = if row_msb_bits != bits::<N>(0) {
        bits::<2>(1)
    } else {
        bits::<2>(0)
    };
    let col_msb: Bits<2> = if col_msb_bits != bits::<N>(0) {
        bits::<2>(1)
    } else {
        bits::<2>(0)
    };
    let mut o = Out::dont_care();
    o.sample_out = row_msb + col_msb;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In<16> {
        In {
            phase_inc_row: bits(0),
            phase_inc_col: bits(0),
            enable: false,
        }
    }

    #[test]
    fn test_idle_holds_phase() -> miette::Result<()> {
        let uut = DtmfGenerator::<16>::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // sample_out should be constant (0) when accumulators are still at reset.
        let initial = outputs.first().unwrap().output.sample_out.raw();
        assert!(outputs.iter().all(|s| s.output.sample_out.raw() == initial));
        Ok(())
    }

    /// Run a row tone alone and verify the MSB toggles at the expected rate.
    #[test]
    fn test_row_tone_only() -> miette::Result<()> {
        let uut = DtmfGenerator::<8>::default();
        // phase_inc = 8 → MSB toggles every 16 samples (period 32 samples).
        let mut stream_in: Vec<In<8>> = Vec::new();
        for _ in 0..128 {
            stream_in.push(In {
                phase_inc_row: bits(8),
                phase_inc_col: bits(0), // no col tone
                enable: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // sample_out values should oscillate between 0 and 1 (col MSB stays 0).
        let max = outputs
            .iter()
            .map(|s| s.output.sample_out.raw())
            .max()
            .unwrap();
        let min = outputs
            .iter()
            .map(|s| s.output.sample_out.raw())
            .min()
            .unwrap();
        assert_eq!(max, 1, "max sample with one tone should be 1");
        assert_eq!(min, 0);
        Ok(())
    }

    /// Both tones produce values in {0, 1, 2} — staircase confirmed.
    #[test]
    fn test_both_tones_staircase() -> miette::Result<()> {
        let uut = DtmfGenerator::<8>::default();
        let mut stream_in: Vec<In<8>> = Vec::new();
        for _ in 0..256 {
            stream_in.push(In {
                phase_inc_row: bits(8),
                phase_inc_col: bits(13),
                enable: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let max = outputs
            .iter()
            .map(|s| s.output.sample_out.raw())
            .max()
            .unwrap();
        let min = outputs
            .iter()
            .map(|s| s.output.sample_out.raw())
            .min()
            .unwrap();
        assert_eq!(max, 2, "two-tone max should reach 2");
        assert_eq!(min, 0);
        // We should observe all three values 0, 1, 2 across the trace.
        let saw_one = outputs.iter().any(|s| s.output.sample_out.raw() == 1);
        assert!(saw_one);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = DtmfGenerator::<8>::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["3944"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_dtmf_generator_hdl_works() -> miette::Result<()> {
        let uut = DtmfGenerator::<8>::default();
        let mut stream_in: Vec<In<8>> = Vec::new();
        for _ in 0..32 {
            stream_in.push(In {
                phase_inc_row: bits(8),
                phase_inc_col: bits(13),
                enable: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_dtmf_generator_trace() -> miette::Result<()> {
        let uut = DtmfGenerator::<8>::default();
        let mut stream_in: Vec<In<8>> = Vec::new();
        for _ in 0..64 {
            stream_in.push(In {
                phase_inc_row: bits(8),
                phase_inc_col: bits(13),
                enable: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("dtmf_generator");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["fdf04f60c79f23aff1727372b23a6a54c3f65f40c19b9075111ecbd6a0a8cc41"];
        let digest = vcd
            .dump_to_file(root.join("dtmf_generator.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
