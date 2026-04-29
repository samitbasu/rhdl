//! MIPI DBI Type C display driver (SPI + D/C# tagging)
//!
//! 4-wire SPI plus a `D/C#` (data/command) pin plus optional
//! `RESET#` and `BUSY` inputs.  The single most useful display
//! widget — the same protocol layer drives Sitronix
//! ST7735 / ST7789 / ST7796, Ilitek ILI9341 / ILI9488, Galaxycore
//! GC9A01 (1.28″ round 240×240 smartwatch displays), Solomon
//! SSD1351 / SSD1331 (color OLEDs), and SSD1306 / SH1106 (mono
//! OLEDs in SPI mode).  The SPI bit-level transport is identical
//! across all of them — only the *command set* and pixel format
//! differ, captured as per-controller init-sequence ROM tables
//! the host loads.
//!
//! **v1 scope:** byte-at-a-time (8-bit SPI word) bus driver.  The
//! host strobes `send` with each byte and sets `is_command` to
//! drive D/C# low (command) or high (data) for the duration of
//! the SPI cycle.  Init-sequence ROM, pixel-stream FSM, partial-
//! refresh window commands, and FIFO-buffered pixel streaming
//! are all v2 (each is a self-contained layer that composes on
//! top of this transport).
//!
//! Composes [super::spi_master::SpiMaster].
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+MipiDbiTypeC+------+
     |                           |
B<8> |                           | bool
+--->| data                sclk  +--->
bool |                           | bool
+--->| is_command          mosi  +--->
bool |                           | bool
+--->| send                cs_n  +--->
     |                     dc_n  +--->
     |                     busy  +--->
     |                     done  +--->
     +---------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/mipi_dbi_type_c.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/mipi_dbi_type_c.md")]
use rhdl::prelude::*;

use super::spi_master::{SpiMaster, spi_master as spi_master_kernel};
use crate::core::dff;

#[allow(unused_imports)]
use spi_master_kernel as _;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
/// MIPI DBI Type C display driver.
pub struct MipiDbiTypeC<const CW: usize>
where
    rhdl::bits::W<CW>: BitWidth,
{
    spi: SpiMaster<8, CW>,
    /// Latched D/C# value for the current transaction.  D/C# stays
    /// stable from the cycle `send` is asserted until the SPI
    /// transfer completes.
    dc_n_reg: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [MipiDbiTypeC].
pub struct In {
    /// Byte to transmit.  Latched at `send`.
    pub data: Bits<8>,
    /// Whether the byte is a command (true → D/C# = 0) or data
    /// (false → D/C# = 1).  Latched at `send`.
    pub is_command: bool,
    /// Strobe to begin a byte transfer.  Ignored while `busy`.
    pub send: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [MipiDbiTypeC].
pub struct Out {
    pub sclk: bool,
    pub mosi: bool,
    pub cs_n: bool,
    /// D/C# pin.  Active-low semantic: `false` ⇒ command, `true` ⇒ data.
    pub dc_n: bool,
    pub busy: bool,
    pub done: bool,
}

impl<const CW: usize> SynchronousIO for MipiDbiTypeC<CW>
where
    rhdl::bits::W<CW>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = mipi_dbi_type_c<CW>;
}

#[kernel]
/// Kernel for [MipiDbiTypeC].
pub fn mipi_dbi_type_c<const CW: usize>(
    cr: ClockReset,
    i: In,
    q: Q<CW>,
) -> (Out, D<CW>)
where
    rhdl::bits::W<CW>: BitWidth,
{
    let mut d = D::<CW>::dont_care();
    d.spi = super::spi_master::In::<8> {
        tx_data: i.data,
        start: i.send,
        miso: false, // Display devices are typically write-only
    };
    d.dc_n_reg = q.dc_n_reg;

    // Latch D/C# at the same cycle the SPI starts so it lines up
    // with the SPI master's CS-asserted window.
    if i.send {
        d.dc_n_reg = !i.is_command;
    }

    if cr.reset.any() {
        d.dc_n_reg = true; // idle is data-mode high
    }

    let mut o = Out::dont_care();
    o.sclk = q.spi.sclk;
    o.mosi = q.spi.mosi;
    o.cs_n = q.spi.cs_n;
    o.dc_n = q.dc_n_reg;
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
        }
    }

    #[test]
    fn test_command_drives_dc_n_low() -> miette::Result<()> {
        let uut = MipiDbiTypeC::<4>::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0x29), // ST7789 DISPLAY_ON command
            is_command: true,
            send: true,
        }];
        for _ in 0..40 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // While CS is asserted (cs_n=0), D/C# must be low (command).
        let bad = outputs
            .iter()
            .any(|s| !s.output.cs_n && s.output.dc_n);
        assert!(!bad, "D/C# must be low (command) while CS is asserted");
        Ok(())
    }

    #[test]
    fn test_data_drives_dc_n_high() -> miette::Result<()> {
        let uut = MipiDbiTypeC::<4>::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xFF),
            is_command: false,
            send: true,
        }];
        for _ in 0..40 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let bad = outputs
            .iter()
            .any(|s| !s.output.cs_n && !s.output.dc_n);
        assert!(!bad, "D/C# must be high (data) while CS is asserted");
        Ok(())
    }

    #[test]
    fn test_byte_completes() -> miette::Result<()> {
        let uut = MipiDbiTypeC::<4>::default();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            is_command: false,
            send: true,
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
            "no done pulse"
        );
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = MipiDbiTypeC::<4>::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["12703"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_mipi_dbi_type_c_hdl_works() -> miette::Result<()> {
        let uut = MipiDbiTypeC::<4>::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_mipi_dbi_type_c_trace() -> miette::Result<()> {
        let uut = MipiDbiTypeC::<4>::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("mipi_dbi_type_c");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["bda60a30f6e583ddfe06ccb768503a8092fa7b2bf7e3455fe1f800992922a04e"];
        let digest = vcd
            .dump_to_file(root.join("mipi_dbi_type_c.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
