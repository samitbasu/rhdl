//! NTSC composite sync encoder (monochrome v1)
//!
//! Produces a 2-bit composite-video signal that, when fed into a
//! cheap 2-bit R-2R DAC (two FPGA pins → 1 video output), drives a
//! standard composite-input monitor or capture device with valid
//! NTSC-style horizontal and vertical sync.  The picture content is
//! a 2-bit luma sample provided by the host every cycle (so this
//! widget is the encoder, not the framebuffer).
//!
//! The 2-bit code on `composite` maps to standard composite levels:
//!
//! | Code | Level                         | IRE     |
//! |------|-------------------------------|---------|
//! | `00` | sync tip (during HSYNC/VSYNC) | 0       |
//! | `01` | blanking / black              | 7.5     |
//! | `10` | mid-gray (picture)            | ~50     |
//! | `11` | white (picture)               | 100     |
//!
//! **v1 scope: monochrome, simplified VSYNC.**
//!
//! - **Monochrome only.**  No color subcarrier, no colorburst, no
//!   chrominance modulation.  A real NTSC encoder needs a 3.579545
//!   MHz colorburst that's phase-locked to the horizontal scan and
//!   gated into the back porch of each line; the chrominance is
//!   then quadrature-modulated with the I/Q color-difference
//!   signals.  v2 follow-up.
//! - **Simplified VSYNC.**  Real NTSC vertical sync is a 9-line
//!   sequence: 6 equalizing pulses + 6 broad VSYNC pulses + 6
//!   equalizing pulses, each at half-line frequency.  v1 emits a
//!   single broad VSYNC pulse for the duration of the
//!   `VideoTimingCore` vsync region — sloppy but accepted by most
//!   capture equipment in "rough sync" mode.  v2 follow-up.
//! - **No interlace.**  v1 emits a progressive 262-line frame
//!   (NTSC is 525 lines interlaced; v1 produces "240p"-ish output).
//!   v2 follow-up.
//! - **Pixel clock = FPGA clock.**  At canonical NTSC timings the
//!   pixel clock is 14.318 MHz (4× the 3.579545 MHz subcarrier) or
//!   13.5 MHz (BT.601 standard); the FPGA needs to match.  v2
//!   follow-up to add an internal divider.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+NtscComposite+-----+
     |                         |
B<2> |                         | B<2>
+--->| pic_sample    composite +--->
     |                         | bool
     |                    active+--->
     |                         | bool
     |                     hsync+--->
     |                         | bool
     |                     vsync+--->
     +-------------------------+
")]
//!
//!# Internals
//!
//! Wraps the shipped [super::video_timing::VideoTimingCore] with
//! NTSC timings (the host picks via [Self::ntsc_240p]) and
//! composes a tiny 4-way multiplexer that selects the composite
//! output level based on `(hsync OR vsync, active)`:
//!
//! ```text
//! composite = if hsync || vsync   { 00 (sync tip) }
//!             else if active       { max(pic_sample, 01) }   // never below black
//!             else                 { 01 (blanking) }
//! ```
//!
//! `pic_sample` is gated to never drop below blanking (`01`) during
//! active pixels — a real video signal has a setup pedestal at
//! 7.5 IRE so that "black" reads correctly through the blanking
//! comparator on the receiving monitor.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/ntsc_composite.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/ntsc_composite.md")]
use rhdl::prelude::*;

use super::video_timing::{VideoTimingCore, video_timing as video_timing_kernel};

#[allow(unused_imports)]
use video_timing_kernel as _;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// NTSC composite sync encoder (monochrome v1).
pub struct NtscComposite<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    timing: VideoTimingCore<HW, VW>,
}

impl<const HW: usize, const VW: usize> NtscComposite<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Create a composite encoder with the given timing parameters.
    /// Use [Self::ntsc_240p] for canonical NTSC-progressive timings.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        h_total: Bits<HW>,
        h_active_end: Bits<HW>,
        h_sync_start: Bits<HW>,
        h_sync_end: Bits<HW>,
        v_total: Bits<VW>,
        v_active_end: Bits<VW>,
        v_sync_start: Bits<VW>,
        v_sync_end: Bits<VW>,
    ) -> Self {
        Self {
            timing: VideoTimingCore::new(
                h_total,
                h_active_end,
                h_sync_start,
                h_sync_end,
                v_total,
                v_active_end,
                v_sync_start,
                v_sync_end,
            ),
        }
    }

    /// Canonical NTSC "240p" progressive timings at 13.5 MHz pixel clock:
    /// 858 cycles × 262 lines, 720 active pixels × 240 active lines.
    ///
    /// Requires `HW >= 10` (h_total = 858 fits in 10 bits) and
    /// `VW >= 9` (v_total = 262 fits in 9 bits).
    pub fn ntsc_240p() -> Self
    where
        Bits<HW>: From<u128>,
        Bits<VW>: From<u128>,
    {
        Self::new(
            bits(858), // h_total (one scanline at 13.5 MHz)
            bits(720), // h_active_end
            bits(736), // h_sync_start (16-cycle front porch)
            bits(800), // h_sync_end (~4.7 µs sync pulse)
            bits(262), // v_total
            bits(240), // v_active_end
            bits(245), // v_sync_start
            bits(248), // v_sync_end (3-line broad VSYNC)
        )
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [NtscComposite].
pub struct In {
    /// 2-bit picture luma sample.  Gated to never drop below `01` (black) during
    /// active pixels.  Ignored during blanking.
    pub pic_sample: Bits<2>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [NtscComposite].
pub struct Out<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Composite-video output: `00` = sync tip, `01` = blank/black,
    /// `10`/`11` = picture luma.  Drive a 2-bit R-2R DAC for analog out.
    pub composite: Bits<2>,
    /// `true` during the active picture region.
    pub active: bool,
    /// Horizontal sync (sync tip is being asserted, low on the wire).
    pub hsync: bool,
    /// Vertical sync (broad VSYNC pulse, asserted for several lines).
    pub vsync: bool,
    /// Current horizontal pixel position.
    pub pixel_x: Bits<HW>,
    /// Current vertical line position.
    pub pixel_y: Bits<VW>,
}

impl<const HW: usize, const VW: usize> SynchronousIO for NtscComposite<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    type I = In;
    type O = Out<HW, VW>;
    type Kernel = ntsc_composite<HW, VW>;
}

#[kernel]
/// Kernel for [NtscComposite].
pub fn ntsc_composite<const HW: usize, const VW: usize>(
    _cr: ClockReset,
    i: In,
    q: Q<HW, VW>,
) -> (Out<HW, VW>, D<HW, VW>)
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    let mut d = D::<HW, VW>::dont_care();
    d.timing = ();

    let hsync = q.timing.hsync;
    let vsync = q.timing.vsync;
    let active = q.timing.active;

    // Composite level selection.  Sync (HSYNC or VSYNC) → 00.
    // Active → max(pic_sample, blanking) → 01..11.
    // Blanking → 01.
    let in_sync = hsync || vsync;
    let pic_or_black = if i.pic_sample == bits::<2>(0) {
        bits::<2>(1) // gate to blanking minimum
    } else {
        i.pic_sample
    };
    let composite = if in_sync {
        bits::<2>(0)
    } else if active {
        pic_or_black
    } else {
        bits::<2>(1)
    };

    let mut o = Out::<HW, VW>::dont_care();
    o.composite = composite;
    o.active = active;
    o.hsync = hsync;
    o.vsync = vsync;
    o.pixel_x = q.timing.pixel_x;
    o.pixel_y = q.timing.pixel_y;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    /// Mini timings: 32-cycle line × 6 lines, with an 8-cycle blanking +
    /// 4-cycle sync region per line and a 1-line VSYNC near the end.
    fn mini() -> NtscComposite<6, 4> {
        NtscComposite::new(
            bits(32), // h_total
            bits(20), // h_active_end
            bits(24), // h_sync_start
            bits(28), // h_sync_end
            bits(6),  // v_total
            bits(4),  // v_active_end
            bits(5),  // v_sync_start
            bits(6),  // v_sync_end
        )
    }

    fn idle_in() -> In {
        In {
            pic_sample: bits(2), // mid-gray default
        }
    }

    #[test]
    fn test_sync_tip_during_hsync() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_bad = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.hsync || s.output.vsync)
            .any(|s| s.output.composite.raw() != 0);
        assert!(!any_bad, "composite must be 00 (sync tip) during HSYNC/VSYNC");
        Ok(())
    }

    #[test]
    fn test_blanking_is_black() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_bad = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| !s.output.active && !s.output.hsync && !s.output.vsync)
            .any(|s| s.output.composite.raw() != 1);
        assert!(!any_bad, "composite must be 01 (blanking) outside sync and active");
        Ok(())
    }

    #[test]
    fn test_active_passes_pic_sample() -> miette::Result<()> {
        let uut = mini();
        // Drive pic_sample = 3 (white) throughout.  In active region we expect
        // composite = 11 (white).
        let stream = std::iter::repeat_n(In { pic_sample: bits(3) }, 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let active_samples: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.active)
            .map(|s| s.output.composite.raw())
            .collect();
        assert!(!active_samples.is_empty(), "no active samples observed");
        assert!(
            active_samples.iter().all(|&c| c == 3),
            "expected all-white during active, got: {active_samples:?}"
        );
        Ok(())
    }

    #[test]
    fn test_pic_sample_zero_gates_to_black() -> miette::Result<()> {
        // pic_sample = 0 should be gated to 01 (blanking) in active region.
        let uut = mini();
        let stream = std::iter::repeat_n(In { pic_sample: bits(0) }, 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_zero_active = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.active)
            .any(|s| s.output.composite.raw() == 0);
        assert!(!any_zero_active, "pic_sample=0 must gate to 01, never 00 (sync tip)");
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check.
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = mini();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8090"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip.
    #[test]
    fn test_ntsc_composite_hdl_works() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest.
    #[test]
    fn test_ntsc_composite_trace() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ntsc_composite");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["897193d66819edffe7dd2145431968494e3b8c4939a8425862c3c3197b5a639e"];
        let digest = vcd.dump_to_file(root.join("ntsc_composite.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
