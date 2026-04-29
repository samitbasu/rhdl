//! E-paper SPI driver (Solomon SSD16xx family)
//!
//! Drives SSD1608 / SSD1619 / SSD1633 / SSD1675 / SSD1677 e-paper
//! controllers, which between them cover most Waveshare and Good
//! Display 1.54″ / 2.13″ / 2.9″ / 4.2″ / 7.5″ / 10.3″ e-paper
//! modules.  Standard signal set: SCK, MOSI, CS#, D/C#, RESET#,
//! BUSY# (driven low while the controller is refreshing).
//!
//! E-paper is fundamentally a "shoot and wait" protocol: the host
//! loads the framebuffer via SPI, kicks off a refresh, and waits
//! for BUSY# to drop (which can take 100 ms to several seconds for
//! full refresh).  This widget composes the
//! [super::spi_master::SpiMaster] with D/C# tagging and BUSY#
//! polling, gating new `send` strobes while the controller is
//! still refreshing.
//!
//! **v1 scope:** byte-at-a-time SPI transport.  Host strobes
//! `send` with each byte, the widget shifts it through SPI with
//! D/C# tagged appropriately, and stalls subsequent strobes until
//! BUSY# is high.  RESET# is exposed as a passthrough output the
//! host pulses at power-on.  Per-controller waveform-mode tables
//! and the framebuffer protocol layer are deferred to v2.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+EpaperSsd16xx+-----+
     |                           |
B<8> |                           | bool
+--->| data                sclk  +--->
bool |                           | bool
+--->| is_command          mosi  +--->
bool |                           | bool
+--->| send                cs_n  +--->
bool |                           | bool
+--->| busy_n              dc_n  +--->
bool |                           | bool
+--->| reset_n_in       reset_n  +--->
     |                ctrl_busy  +--->
     |                       busy+--->
     |                       done+--->
     +---------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/epaper_ssd16xx.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/epaper_ssd16xx.md")]
use rhdl::prelude::*;

use super::spi_master::{SpiMaster, spi_master as spi_master_kernel};
use crate::core::dff;

#[allow(unused_imports)]
use spi_master_kernel as _;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
/// E-paper SPI driver.
pub struct EpaperSsd16xx<const CW: usize>
where
    rhdl::bits::W<CW>: BitWidth,
{
    spi: SpiMaster<8, CW>,
    dc_n_reg: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [EpaperSsd16xx].
pub struct In {
    pub data: Bits<8>,
    pub is_command: bool,
    pub send: bool,
    /// BUSY# pin from the controller.  `false` ⇒ controller is busy
    /// and the host must wait.
    pub busy_n: bool,
    /// RESET# value driven by the host (passed through to the panel).
    pub reset_n_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [EpaperSsd16xx].
pub struct Out {
    pub sclk: bool,
    pub mosi: bool,
    pub cs_n: bool,
    pub dc_n: bool,
    pub reset_n: bool,
    /// `true` while the controller's BUSY# is asserted (low) — i.e.,
    /// it's refreshing.  The host should not strobe `send` while
    /// this is set.
    pub ctrl_busy: bool,
    pub busy: bool,
    pub done: bool,
}

impl<const CW: usize> SynchronousIO for EpaperSsd16xx<CW>
where
    rhdl::bits::W<CW>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = epaper_ssd16xx<CW>;
}

#[kernel]
/// Kernel for [EpaperSsd16xx].
pub fn epaper_ssd16xx<const CW: usize>(
    cr: ClockReset,
    i: In,
    q: Q<CW>,
) -> (Out, D<CW>)
where
    rhdl::bits::W<CW>: BitWidth,
{
    let mut d = D::<CW>::dont_care();
    // Gate `send` while the controller is busy or while we're
    // already mid-byte.  This makes the widget tolerant of host
    // strobes during refresh — they're simply ignored, matching
    // how a host SPI driver should behave around BUSY# anyway.
    let ctrl_busy = !i.busy_n;
    let send_allowed = i.send && !ctrl_busy;
    d.spi = super::spi_master::In::<8> {
        tx_data: i.data,
        start: send_allowed,
        miso: false,
    };
    d.dc_n_reg = q.dc_n_reg;
    if send_allowed {
        d.dc_n_reg = !i.is_command;
    }
    if cr.reset.any() {
        d.dc_n_reg = true;
    }

    let mut o = Out::dont_care();
    o.sclk = q.spi.sclk;
    o.mosi = q.spi.mosi;
    o.cs_n = q.spi.cs_n;
    o.dc_n = q.dc_n_reg;
    o.reset_n = i.reset_n_in;
    o.ctrl_busy = ctrl_busy;
    o.busy = q.spi.busy;
    o.done = q.spi.done;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            data: bits(0),
            is_command: false,
            send: false,
            busy_n: true,
            reset_n_in: true,
        }
    }

    #[test]
    fn test_idle_no_send() -> miette::Result<()> {
        let uut = EpaperSsd16xx::<4>::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_busy = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.busy);
        assert!(!any_busy, "no SPI activity should occur in idle");
        Ok(())
    }

    #[test]
    fn test_busy_signal_passes_through() -> miette::Result<()> {
        let uut = EpaperSsd16xx::<4>::default();
        let mut stream_in: Vec<In> = Vec::new();
        for _ in 0..16 {
            let mut x = idle_in();
            x.busy_n = false; // controller asserts busy
            stream_in.push(x);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let all_ctrl_busy = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .all(|s| s.output.ctrl_busy);
        assert!(
            all_ctrl_busy,
            "ctrl_busy must mirror !busy_n (busy_n=false → ctrl_busy=true)"
        );
        Ok(())
    }

    #[test]
    fn test_send_gated_while_busy() -> miette::Result<()> {
        // Host strobes `send` while the controller's BUSY# is low
        // (controller refreshing).  The widget must NOT issue an
        // SPI transaction.
        let uut = EpaperSsd16xx::<4>::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            is_command: false,
            send: true,
            busy_n: false, // controller busy — should gate
            reset_n_in: true,
        }];
        for _ in 0..40 {
            stream_in.push(In {
                busy_n: false,
                ..idle_in()
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let any_done = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.done);
        assert!(!any_done, "no done pulse — send was gated by ctrl_busy");
        Ok(())
    }

    #[test]
    fn test_byte_completes_when_idle() -> miette::Result<()> {
        let uut = EpaperSsd16xx::<4>::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xAA),
            is_command: true,
            send: true,
            busy_n: true,
            reset_n_in: true,
        }];
        for _ in 0..40 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        assert!(
            uut.run(stream)
                .synchronous_sample()
                .filter(|s| !s.input.0.reset.any())
                .any(|s| s.output.done),
            "byte should complete when busy_n is high"
        );
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = EpaperSsd16xx::<4>::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["13094"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_epaper_ssd16xx_hdl_works() -> miette::Result<()> {
        let uut = EpaperSsd16xx::<4>::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_epaper_ssd16xx_trace() -> miette::Result<()> {
        let uut = EpaperSsd16xx::<4>::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("epaper_ssd16xx");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["a5dfd291ddcdf1fbecc13260fa5f1e9c31b0dda8d261cc0dded1b548df7e6f0d"];
        let digest = vcd
            .dump_to_file(root.join("epaper_ssd16xx.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
