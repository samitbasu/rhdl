//! MIPI DPI / RGB parallel display driver
//!
//! For large TFT panels (5″ / 7″ / 10″) without an integrated
//! controller — the host streams pixels in real time exactly like
//! VGA, just at LVTTL voltages and with a different pin shape:
//! VSYNC, HSYNC, DE (data enable), DCLK, R[7:0], G[7:0], B[7:0].
//!
//! This widget wraps the shipped [super::video_timing::VideoTimingCore]
//! with the DPI pin shape — same H/V counters, same sync generator,
//! same active-region detection, only the output bundle differs.
//! The DE pin is just the active-region flag from the timing core.
//!
//! Single biggest user: 800×480 5″ and 1024×600 7″ TFT panels common
//! on FPGA dev boards (Numato, Digilent, Olimex, Terasic).
//!
//! **v1 scope:** the host supplies the per-pixel RGB sample; this
//! widget multiplexes it onto the parallel pixel-bus output during
//! the active region and drives it to all-zero during blanking
//! (the DE signal tells the panel when the bus is meaningful).
//! Framebuffer / pattern generator wiring is left to the host.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+MipiDpi+-------+
     |                       |
B<8> |                       | B<8>
+--->| r_in              r   +--->
B<8> |                       | B<8>
+--->| g_in              g   +--->
B<8> |                       | B<8>
+--->| b_in              b   +--->
     |                  hsync+--->
     |                  vsync+--->
     |                     de+--->
     |               pixel_x +--->
     |               pixel_y +--->
     +-----------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/mipi_dpi.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/mipi_dpi.md")]
use rhdl::prelude::*;

use super::video_timing::{VideoTimingCore, video_timing as video_timing_kernel};

#[allow(unused_imports)]
use video_timing_kernel as _;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// MIPI DPI / RGB parallel display output.
pub struct MipiDpi<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    timing: VideoTimingCore<HW, VW>,
}

impl<const HW: usize, const VW: usize> MipiDpi<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Create a DPI output with the given timing parameters.  Use
    /// the `*_preset` helpers for canonical panel timings.
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
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [MipiDpi].
pub struct In {
    pub r_in: Bits<8>,
    pub g_in: Bits<8>,
    pub b_in: Bits<8>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [MipiDpi].
pub struct Out<const HW: usize, const VW: usize>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    /// Red channel (gated to 0 outside the active region).
    pub r: Bits<8>,
    /// Green channel.
    pub g: Bits<8>,
    /// Blue channel.
    pub b: Bits<8>,
    /// Horizontal sync pulse.
    pub hsync: bool,
    /// Vertical sync pulse.
    pub vsync: bool,
    /// Data enable — `true` while the panel should latch the RGB bus.
    pub de: bool,
    pub pixel_x: Bits<HW>,
    pub pixel_y: Bits<VW>,
}

impl<const HW: usize, const VW: usize> SynchronousIO for MipiDpi<HW, VW>
where
    rhdl::bits::W<HW>: BitWidth,
    rhdl::bits::W<VW>: BitWidth,
{
    type I = In;
    type O = Out<HW, VW>;
    type Kernel = mipi_dpi<HW, VW>;
}

#[kernel]
/// Kernel for [MipiDpi].
pub fn mipi_dpi<const HW: usize, const VW: usize>(
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

    let active = q.timing.active;
    let r = if active { i.r_in } else { bits::<8>(0) };
    let g = if active { i.g_in } else { bits::<8>(0) };
    let b = if active { i.b_in } else { bits::<8>(0) };

    let mut o = Out::<HW, VW>::dont_care();
    o.r = r;
    o.g = g;
    o.b = b;
    o.hsync = q.timing.hsync;
    o.vsync = q.timing.vsync;
    o.de = active;
    o.pixel_x = q.timing.pixel_x;
    o.pixel_y = q.timing.pixel_y;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn mini() -> MipiDpi<7, 4> {
        MipiDpi::new(
            bits(64),
            bits(40),
            bits(48),
            bits(56),
            bits(8),
            bits(4),
            bits(6),
            bits(7),
        )
    }

    fn idle_in() -> In {
        In {
            r_in: bits(0xFF),
            g_in: bits(0x80),
            b_in: bits(0x40),
        }
    }

    #[test]
    fn test_de_matches_active_region() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 1024)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_active_de_mismatch = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| {
                let active = s.output.r.raw() != 0
                    || s.output.g.raw() != 0
                    || s.output.b.raw() != 0;
                // RGB nonzero => DE must be high (since input is constant nonzero).
                active != s.output.de
            });
        assert!(!any_active_de_mismatch, "DE != (RGB!=0) somewhere");
        Ok(())
    }

    #[test]
    fn test_blanking_zeros_rgb() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 1024)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_blank_color = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| {
                !s.output.de
                    && (s.output.r.raw() != 0
                        || s.output.g.raw() != 0
                        || s.output.b.raw() != 0)
            });
        assert!(!any_blank_color, "RGB must be zero outside DE-active region");
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = mini();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8354"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_mipi_dpi_hdl_works() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_mipi_dpi_trace() -> miette::Result<()> {
        let uut = mini();
        let stream = std::iter::repeat_n(idle_in(), 256)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("mipi_dpi");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["b66268c4e63dfb5bede30684752db1b9641a0c1219cb720c9faee46e78bf2144"];
        let digest = vcd.dump_to_file(root.join("mipi_dpi.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
