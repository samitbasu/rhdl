#![warn(missing_docs)]
//! Video-output widgets.
//!
//! Pixel-clock-driven counters that produce horizontal and
//! vertical sync timings for raster displays, plus the
//! per-format colour / luma encoders that compose on top.  The
//! [`video_timing::VideoTimingCore`] is the shared substrate;
//! everything else (CGA digital RGBI, NTSC composite, future
//! VGA / MDA / EGA / DVI / HDMI flavours) wraps it with
//! format-specific output formatting.
//!
//! Pixel clocks for these widgets are typically faster than the
//! host FPGA's board clock — the canonical setup is a PLL
//! synthesizing the exact pixel rate (14.318 MHz for CGA, 13.5
//! MHz for BT.601 NTSC, 25.175 MHz for VGA 640×480 @ 60 Hz, and
//! so on) and clocking the timing core off that.
pub mod cga_rgbi;
pub mod mipi_dpi;
pub mod ntsc_composite;
pub mod video_timing;
