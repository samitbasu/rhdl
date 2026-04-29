//! IBM CGA digital RGBI video output (test-pattern v1)
//!
//! Wraps the shipped [super::video_timing::VideoTimingCore] with
//! CGA's 320×200 @ 60 Hz timings and produces the 4-bit RGBI digital
//! output that drives the original IBM CGA monitor (and its many
//! clones) directly.  The `Intensity` (I) bit doubles the brightness
//! of each of the R, G, B channels — yielding the canonical 16-color
//! CGA palette.
//!
//! **v1 scope:**
//! - Test-pattern generator: 16 vertical color bars sweeping the full
//!   palette across the active region.  Demonstrates the full RGBI
//!   color space and validates the timing core's H/V counters.
//! - **No framebuffer**, **no character ROM**, **no attribute byte
//!   decoding**.  Each is the natural next layer (compose this widget
//!   with `core::ram` for the framebuffer + `core::ram` for the font
//!   ROM); they are tracked as v2 follow-ups.
//! - Pixel clock is the FPGA clock — at the canonical 14.318 MHz
//!   pixel-clock rate, the FPGA needs to be clocked at 14.318 MHz
//!   (or use a `Constant`-driven divider, future work).
//! - **Digital output only.**  The composite-NTSC artifact-color
//!   path is a separate widget tracked as #39.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+CgaRgbi+-------+
     |                       |
     |                  hsync+--->
     |                       |
     |                  vsync+--->
     |                       |
     |                  r,g,b+--->
     |                       |
     |                      i+--->
     |                       |
     |                 active+--->
     |                       |
     |               pixel_x +--->
     |                       |
     |               pixel_y +--->
     +-----------------------+
")]
//!
//!# Internals
//!
//! The widget is thin: a single `VideoTimingCore` sub-circuit, plus
//! pure-combinational logic that maps `(pixel_x, active)` to the
//! 4-bit RGBI test pattern.  The pattern divides the active region
//! into 16 vertical bars, with bar `i` emitting RGBI = `i` (treating
//! the bit positions as `[I, R, G, B]` so bar 0 = black, bar 1 =
//! blue, bar 2 = green, …, bar 15 = bright white).
//!
//!# CGA timing reference
//!
//! Standard 320×200 @ 60 Hz CGA timings (pixel clock 14.318 MHz):
//!
//! | Parameter        | Value            |
//! |------------------|------------------|
//! | `h_total`        | 912 pixel clocks |
//! | `h_active_end`   | 640              |
//! | `h_sync_start`   | 668              |
//! | `h_sync_end`     | 768              |
//! | `v_total`        | 262 lines        |
//! | `v_active_end`   | 200              |
//! | `v_sync_start`   | 224              |
//! | `v_sync_end`     | 230              |
//!
//! Note: the original CGA renders 320 horizontal pixels by clocking
//! out two "color pixels" per character cell at 14.318 MHz for an
//! effective horizontal active of 640 pixel clocks.  We follow the
//! `pixel-clock = active = 640` convention here so the RGBI test
//! pattern's vertical bars line up with character-cell boundaries.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/cga_rgbi.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/cga_rgbi.md")]
use rhdl::prelude::*;

use super::video_timing::{video_timing as video_timing_kernel, VideoTimingCore};

// Re-export the kernel function so it's visible inside the kernel macro
// expansion (the macro generates code that references super::video_timing
// as a kernel function — without this `use`, name resolution fails inside
// nested kernels).
#[allow(unused_imports)]
use video_timing_kernel as _;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// CGA digital RGBI video output with test-pattern generator (v1).
pub struct CgaRgbi<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    timing: VideoTimingCore<HW, VW>,
}

impl<const HW: usize, const VW: usize> CgaRgbi<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Create a CGA RGBI output with the given timing parameters.
    /// Use [Self::cga_320x200_60hz] for the canonical IBM-CGA timings.
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

    /// Canonical 320×200 @ 60 Hz CGA timings.  Requires `HW >= 10`
    /// (h_total = 912 fits in 10 bits) and `VW >= 9` (v_total = 262 fits in 9 bits).
    pub fn cga_320x200_60hz() -> Self
    where
        Bits<HW>: From<u128>,
        Bits<VW>: From<u128>,
    {
        Self::new(
            bits(912), // h_total
            bits(640), // h_active_end
            bits(668), // h_sync_start
            bits(768), // h_sync_end
            bits(262), // v_total
            bits(200), // v_active_end
            bits(224), // v_sync_start
            bits(230), // v_sync_end
        )
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [CgaRgbi].
pub struct Out<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Horizontal sync pulse (high during sync region).
    pub hsync: bool,
    /// Vertical sync pulse (high during sync region).
    pub vsync: bool,
    /// High during the active display region.
    pub active: bool,
    /// CGA red bit (one of the 4 RGBI digital outputs).
    pub r: bool,
    /// CGA green bit.
    pub g: bool,
    /// CGA blue bit.
    pub b: bool,
    /// CGA intensity bit (doubles the brightness).
    pub i: bool,
    /// Current horizontal pixel position.
    pub pixel_x: Bits<HW>,
    /// Current vertical line position.
    pub pixel_y: Bits<VW>,
}

impl<const HW: usize, const VW: usize> SynchronousIO for CgaRgbi<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    type I = ();
    type O = Out<HW, VW>;
    type Kernel = cga_rgbi<HW, VW>;
}

#[kernel]
/// Kernel for [CgaRgbi].
pub fn cga_rgbi<const HW: usize, const VW: usize>(
    _cr: ClockReset,
    _i: (),
    q: Q<HW, VW>,
) -> (Out<HW, VW>, D<HW, VW>)
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    let mut d = D::<HW, VW>::dont_care();
    d.timing = ();

    // Test pattern: 16-color cycle every 64 pixels (4 pixels per bar).
    // Bar `n` (0..16) emits RGBI = `n`, treating the bits as [I, R, G, B].
    // The 4-pixel bar width keeps the pattern visible at the test mini-mode
    // (64-pixel-active scanline) and produces 10 full cycles of the
    // palette across the canonical 640-active-pixel CGA H mode.  A v2
    // widget that handles arbitrary active widths via a divider is
    // tracked as a follow-up.
    let pixel_x = q.timing.pixel_x;
    let bar_idx = (pixel_x >> bits::<HW>(2)) & bits::<HW>(0xF);
    let i_bit = (bar_idx & bits::<HW>(0x8)) != bits::<HW>(0);
    let r_bit = (bar_idx & bits::<HW>(0x4)) != bits::<HW>(0);
    let g_bit = (bar_idx & bits::<HW>(0x2)) != bits::<HW>(0);
    let b_bit = (bar_idx & bits::<HW>(0x1)) != bits::<HW>(0);

    let active = q.timing.active;

    let mut o = Out::<HW, VW>::dont_care();
    o.hsync = q.timing.hsync;
    o.vsync = q.timing.vsync;
    o.active = active;
    // Gate RGBI to 0 outside the active region (CGA monitors expect black during blanking).
    o.r = active && r_bit;
    o.g = active && g_bit;
    o.b = active && b_bit;
    o.i = active && i_bit;
    o.pixel_x = q.timing.pixel_x;
    o.pixel_y = q.timing.pixel_y;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    /// Mini-mode for testing: H total=64, V total=8, with a small active
    /// region and 4-pixel-wide bars (so 16 bars × 4 pixels = 64 pixels).
    fn mini() -> CgaRgbi<7, 4> {
        // 64 cycles per line × 8 lines per frame, with the full line as
        // active region (no blanking) so the color-bar pattern fills the
        // active scan.  HW=7 holds h_total=64; VW=4 holds v_total=8.
        CgaRgbi::new(
            bits(64), // h_total
            bits(63), // h_active_end (one cycle of blanking before sync)
            bits(56), // h_sync_start
            bits(60), // h_sync_end
            bits(8),  // v_total
            bits(6),  // v_active_end
            bits(7),  // v_sync_start
            bits(8),  // v_sync_end
        )
    }

    // Tier 2 — exercise the FSM through one full vertical scan and verify
    // that all 16 RGBI codes appear during the active region.
    #[test]
    fn test_pattern_covers_all_16_colors() -> miette::Result<()> {
        let uut = mini();
        // One full frame = 64 * 8 = 512 cycles.  Run for two frames.
        let stream = std::iter::repeat_n((), 1024)
            .with_reset(1)
            .clock_pos_edge(100);
        let mut seen = [false; 16];
        for s in uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.active)
        {
            let o = s.output;
            let code =
                (o.i as usize) << 3 | (o.r as usize) << 2 | (o.g as usize) << 1 | (o.b as usize);
            seen[code] = true;
        }
        let missing: Vec<_> = (0..16).filter(|&i| !seen[i]).collect();
        assert!(missing.is_empty(), "color codes never seen: {missing:?}");
        Ok(())
    }

    #[test]
    fn test_blanking_zeros_rgbi() -> miette::Result<()> {
        // Outside the active region, RGBI must be 0 (CGA monitors blank).
        let uut = CgaRgbi::<7, 4>::new(
            bits(64),
            bits(40), // h_active_end < h_total to leave a clear blanking region
            bits(48),
            bits(56),
            bits(8),
            bits(4), // v_active_end < v_total
            bits(6),
            bits(7),
        );
        let stream = std::iter::repeat_n((), 1024)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_blank_color = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| {
                let o = s.output;
                !o.active && (o.r || o.g || o.b || o.i)
            });
        assert!(!any_blank_color, "RGBI must be zero outside active region");
        Ok(())
    }

    #[test]
    fn test_hsync_and_vsync_pulse() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n((), 1024)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.hsync), "no hsync pulse");
        assert!(outputs.iter().any(|s| s.output.vsync), "no vsync pulse");
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check.
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = mini();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8884"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip.
    #[test]
    fn test_cga_rgbi_hdl_works() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n((), 1024)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest.
    #[test]
    fn test_cga_rgbi_trace() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n((), 1024)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("cga_rgbi");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0d35289e8cf59b3ffe6a582d746816679ce259549e89f14dd9e40350a8b0a8fe"];
        let digest = vcd.dump_to_file(root.join("cga_rgbi.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
