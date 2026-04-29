//! Video timing core (CRTC-style H/V counter + sync generator)
//!
//! Generic horizontal / vertical counter pair with sync-pulse and
//! active-region outputs.  Covers MDA, Hercules, CGA, and any
//! VESA DMT VGA mode by changing the timing parameters at
//! construction.  Used as the sync-and-coordinate spine of any
//! video output widget; the host computes pixel data
//! combinationally from the exposed `pixel_x` / `pixel_y` outputs.
//!
//! The widget is **timing-only**.  It does not include a
//! framebuffer, character ROM, palette, or DAC drive — those
//! compose on top.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-+VideoTiming+-+
     |               |
     |        hsync  +--->
     |        vsync  +--->
     |       active  +--->
     |     pixel_x   +--> B<HW>
     |     pixel_y   +--> B<VW>
     +---------------+
")]
//!
//!# Internals
//!
//! Two counters: `h_counter` walks each line `0..h_total-1`, and
//! `v_counter` walks each frame `0..v_total-1` (advanced when
//! `h_counter` wraps).  All four sync-region boundaries (`h_sync_start`,
//! `h_sync_end`, `v_sync_start`, `v_sync_end`) and the two
//! active-region ends (`h_active_end`, `v_active_end`) are runtime
//! parameters.
//!
//! `hsync` and `vsync` are output as **active-high pulses**.  Some
//! modes (MDA, VGA 640×480) want negative-polarity sync — invert
//! externally with a single inverter.
//!
//!# Reference timings
//!
//! | Mode            | h_total | h_act_end | h_sync_start | h_sync_end | v_total | v_act_end | v_sync_start | v_sync_end |
//! |-----------------|--------:|---------:|------------:|----------:|--------:|---------:|------------:|----------:|
//! | MDA 720×350@50  |     882 |      720 |         746 |       844 |     370 |      350 |         353 |       369 |
//! | VGA 640×480@60  |     800 |      640 |         656 |       752 |     525 |      480 |         490 |       492 |
//! | VGA 800×600@60  |    1056 |      800 |         840 |       968 |     628 |      600 |         601 |       605 |
//!
//!# Parameters
//!
//! - `HW` — bit width of `h_counter` (large enough to count `h_total`)
//! - `VW` — bit width of `v_counter` (large enough to count `v_total`)
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/video_timing.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/video_timing.md")]
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// Generic video timing core.
pub struct VideoTimingCore<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    h_counter: dff::DFF<Bits<HW>>,
    v_counter: dff::DFF<Bits<VW>>,
    h_total: Constant<Bits<HW>>,
    h_active_end: Constant<Bits<HW>>,
    h_sync_start: Constant<Bits<HW>>,
    h_sync_end: Constant<Bits<HW>>,
    v_total: Constant<Bits<VW>>,
    v_active_end: Constant<Bits<VW>>,
    v_sync_start: Constant<Bits<VW>>,
    v_sync_end: Constant<Bits<VW>>,
}

impl<const HW: usize, const VW: usize> VideoTimingCore<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Create a video timing core with the given mode timings.
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
            h_counter: dff::DFF::default(),
            v_counter: dff::DFF::default(),
            h_total: Constant::new(h_total),
            h_active_end: Constant::new(h_active_end),
            h_sync_start: Constant::new(h_sync_start),
            h_sync_end: Constant::new(h_sync_end),
            v_total: Constant::new(v_total),
            v_active_end: Constant::new(v_active_end),
            v_sync_start: Constant::new(v_sync_start),
            v_sync_end: Constant::new(v_sync_end),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [VideoTimingCore].
pub struct Out<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Horizontal sync pulse (high during sync region — invert if your mode wants negative polarity).
    pub hsync: bool,
    /// Vertical sync pulse (high during sync region).
    pub vsync: bool,
    /// High during the active display region (both H and V active).
    pub active: bool,
    /// Current horizontal pixel position within the line.
    pub pixel_x: Bits<HW>,
    /// Current vertical line position within the frame.
    pub pixel_y: Bits<VW>,
}

impl<const HW: usize, const VW: usize> SynchronousIO for VideoTimingCore<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    type I = ();
    type O = Out<HW, VW>;
    type Kernel = video_timing<HW, VW>;
}

#[kernel]
/// Kernel for [VideoTimingCore].
pub fn video_timing<const HW: usize, const VW: usize>(
    cr: ClockReset,
    _i: (),
    q: Q<HW, VW>,
) -> (Out<HW, VW>, D<HW, VW>)
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    let one_h: Bits<HW> = bits::<HW>(1);
    let zero_h: Bits<HW> = bits::<HW>(0);
    let one_v: Bits<VW> = bits::<VW>(1);
    let zero_v: Bits<VW> = bits::<VW>(0);

    let h_done = q.h_counter == (q.h_total - one_h);
    let v_done = h_done && q.v_counter == (q.v_total - one_v);

    let next_h = if h_done { zero_h } else { q.h_counter + one_h };
    let next_v = if v_done {
        zero_v
    } else if h_done {
        q.v_counter + one_v
    } else {
        q.v_counter
    };

    let mut d = D::<HW, VW>::dont_care();
    d.h_counter = next_h;
    d.v_counter = next_v;
    if cr.reset.any() {
        d.h_counter = zero_h;
        d.v_counter = zero_v;
    }

    let hsync = q.h_counter >= q.h_sync_start && q.h_counter < q.h_sync_end;
    let vsync = q.v_counter >= q.v_sync_start && q.v_counter < q.v_sync_end;
    let h_active = q.h_counter < q.h_active_end;
    let v_active = q.v_counter < q.v_active_end;
    let active = h_active && v_active;

    let mut o = Out::<HW, VW>::dont_care();
    o.hsync = hsync;
    o.vsync = vsync;
    o.active = active;
    o.pixel_x = q.h_counter;
    o.pixel_y = q.v_counter;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    /// Mini-mode for testing: H total=10, H active_end=6, H sync 7..9.
    /// V total=4, V active_end=3, V sync 3..4.
    fn mini() -> VideoTimingCore<4, 4> {
        VideoTimingCore::new(
            bits(10), // h_total
            bits(6),  // h_active_end
            bits(7),  // h_sync_start
            bits(9),  // h_sync_end
            bits(4),  // v_total
            bits(3),  // v_active_end
            bits(3),  // v_sync_start
            bits(4),  // v_sync_end
        )
    }

    // Tier 2 — counter sweep verifies the timing pulse alignment.
    #[test]
    fn test_counter_walk_and_pulses() -> miette::Result<()> {
        let uut = mini();
        // Run for 2 full frames = 80 cycles.
        let stream = std::iter::repeat_n((), 80)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Verify hsync occurs only when pixel_x in [7, 9).
        for s in &outputs {
            let x = s.output.pixel_x.raw();
            let expected_hsync = x >= 7 && x < 9;
            assert_eq!(
                s.output.hsync, expected_hsync,
                "hsync at x={x} should be {expected_hsync}"
            );
        }
        // Verify vsync only when pixel_y == 3.
        for s in &outputs {
            let y = s.output.pixel_y.raw();
            let expected_vsync = y == 3;
            assert_eq!(s.output.vsync, expected_vsync, "vsync at y={y}");
        }
        // Verify active only when both x < 6 AND y < 3.
        for s in &outputs {
            let x = s.output.pixel_x.raw();
            let y = s.output.pixel_y.raw();
            let expected_active = x < 6 && y < 3;
            assert_eq!(s.output.active, expected_active, "active at ({x}, {y})");
        }
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = mini();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["6087"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_video_timing_hdl_works() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n((), 80)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest
    #[test]
    fn test_video_timing_trace() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n((), 80)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("video_timing");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["066c741e4539c3b1d1e86d8d5ac4acd16f4ea685e4cc8fb2c862b0e7ab4a8b91"];
        let digest = vcd.dump_to_file(root.join("video_timing.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
