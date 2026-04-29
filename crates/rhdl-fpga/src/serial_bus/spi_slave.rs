//! SPI slave (Mode 0, MSB-first)
//!
//! Standard 4-wire SPI slave: samples `sclk_in`, `mosi_in`,
//! `cs_n_in` from an external master, drives `miso_out` back.
//! This v1 implementation hardcodes **Mode 0** (CPOL=0 / CPHA=0)
//! and **MSB-first** bit order to match
//! [super::spi_master::SpiMaster]; other modes are tracked as
//! follow-ups.
//!
//! The slave samples `sclk_in` on the FPGA clock and edge-detects
//! transitions — this is the standard pattern when the SPI bus is
//! significantly slower than the FPGA clock.  For metastability
//! safety, run `sclk_in`, `mosi_in`, and `cs_n_in` through
//! [super::super::cdc::synchronizer::Sync1Bit] (or the N-stage
//! chain) before they reach this widget.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+SpiSlave+-----+
     |                    |
bool |                    | bool
+--->| sclk_in    miso_out+--->
bool |                    | B<W>
+--->| mosi_in     rx_data+--->
bool |                    | bool
+--->| cs_n_in    rx_valid+--->
B<W> |                    | bool
+--->| tx_data        busy+--->
     +--------------------+
")]
//!
//!# Internals
//!
//! - `prev_sclk`, `prev_cs_n`: previous-cycle samples for edge
//!   detection.
//! - `bit_counter`: counts SPI bits sampled within the current
//!   transfer (`0..W`).
//! - `shift_rx`: collects MOSI bits MSB-first.
//! - `shift_tx`: TX shift register.  Loaded from `tx_data` at the
//!   falling edge of `cs_n_in` (CS assertion).
//! - `received_byte` / `received_valid`: latched output and one-cycle
//!   valid pulse once a full word arrives.
//!
//!# Behavior
//!
//! - Idle (CS deasserted, high): outputs are inactive; `miso_out`
//!   floats (driven low by this widget — wrap with `tristate::simple`
//!   if true high-Z is needed).
//! - On falling edge of `cs_n_in`: latch `tx_data` into `shift_tx`,
//!   reset bit counter and `shift_rx`.
//! - On rising edge of `sclk_in` while CS asserted: sample `mosi_in`
//!   into `shift_rx`.
//! - On falling edge of `sclk_in` while CS asserted: shift
//!   `shift_tx` left so the next bit appears at MSB → `miso_out`.
//! - When the `W`-th bit has been sampled: latch `shift_rx` into
//!   `received_byte`, pulse `rx_valid`.
//!
//!# Parameters
//!
//! - `W` — word width in bits
//! - `CW` — bit width of the bit counter; satisfy `2^CW > W`
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/spi_slave.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/spi_slave.md")]
use rhdl::prelude::*;

use crate::core::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// SPI slave core (Mode 0, MSB-first).
pub struct SpiSlave<const W: usize, const CW: usize>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    prev_sclk: dff::DFF<bool>,
    prev_cs_n: dff::DFF<bool>,
    bit_counter: dff::DFF<Bits<CW>>,
    shift_rx: dff::DFF<Bits<W>>,
    shift_tx: dff::DFF<Bits<W>>,
    received_byte: dff::DFF<Bits<W>>,
    received_valid: dff::DFF<bool>,
}

impl<const W: usize, const CW: usize> Default for SpiSlave<W, CW>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    fn default() -> Self {
        Self {
            prev_sclk: dff::DFF::default(),
            prev_cs_n: dff::DFF::new(true), // CS idle high
            bit_counter: dff::DFF::default(),
            shift_rx: dff::DFF::default(),
            shift_tx: dff::DFF::default(),
            received_byte: dff::DFF::default(),
            received_valid: dff::DFF::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [SpiSlave].
pub struct In<const W: usize>
where
    rhdl::bits::W<W>: BitWidth,
{
    /// Serial clock from the master (sampled, not used as a real clock).
    pub sclk_in: bool,
    /// Serial data in from the master.
    pub mosi_in: bool,
    /// Chip select from the master, active low.
    pub cs_n_in: bool,
    /// Word to transmit on MISO.  Latched at falling edge of `cs_n_in`.
    pub tx_data: Bits<W>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [SpiSlave].
pub struct Out<const W: usize>
where
    rhdl::bits::W<W>: BitWidth,
{
    /// Serial data out to the master.  MSB of the TX shift register.
    pub miso_out: bool,
    /// Last fully-received word.
    pub rx_data: Bits<W>,
    /// One-cycle pulse when `rx_data` is fresh.
    pub rx_valid: bool,
    /// High while CS is asserted.
    pub busy: bool,
}

impl<const W: usize, const CW: usize> SynchronousIO for SpiSlave<W, CW>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    type I = In<W>;
    type O = Out<W>;
    type Kernel = spi_slave<W, CW>;
}

#[kernel]
/// Kernel for [SpiSlave].
pub fn spi_slave<const W: usize, const CW: usize>(
    cr: ClockReset,
    i: In<W>,
    q: Q<W, CW>,
) -> (Out<W>, D<W, CW>)
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    let one_cw: Bits<CW> = bits::<CW>(1);
    let zero_cw: Bits<CW> = bits::<CW>(0);
    let w_cw: Bits<CW> = bits::<CW>(W as u128);
    let zero_w: Bits<W> = bits::<W>(0);
    let one_w: Bits<W> = bits::<W>(1);

    let cs_active = !i.cs_n_in;
    let sclk_rising = !q.prev_sclk && i.sclk_in;
    let sclk_falling = q.prev_sclk && !i.sclk_in;
    let cs_falling = q.prev_cs_n && !i.cs_n_in;

    let mut d = D::<W, CW>::dont_care();
    // Default: hold all state; pulse default low.
    d.prev_sclk = i.sclk_in;
    d.prev_cs_n = i.cs_n_in;
    d.bit_counter = q.bit_counter;
    d.shift_rx = q.shift_rx;
    d.shift_tx = q.shift_tx;
    d.received_byte = q.received_byte;
    d.received_valid = false;

    if cs_falling {
        // CS just asserted: load TX, reset RX state.
        d.shift_tx = i.tx_data;
        d.shift_rx = zero_w;
        d.bit_counter = zero_cw;
    } else if cs_active {
        if sclk_rising {
            // Sample MOSI into shift_rx LSB.
            let mosi_bit: Bits<W> = if i.mosi_in { one_w } else { zero_w };
            let next_rx = (q.shift_rx << 1) | mosi_bit;
            d.shift_rx = next_rx;
            let next_count = q.bit_counter + one_cw;
            d.bit_counter = next_count;
            if next_count == w_cw {
                // Word complete.
                d.received_byte = next_rx;
                d.received_valid = true;
                d.bit_counter = zero_cw;
            }
        }
        if sclk_falling {
            // Shift TX so next MSB is presented on MISO.
            d.shift_tx = q.shift_tx << 1;
        }
    }

    if cr.reset.any() {
        d.prev_sclk = false;
        d.prev_cs_n = true;
        d.bit_counter = zero_cw;
        d.shift_rx = zero_w;
        d.shift_tx = zero_w;
        d.received_byte = zero_w;
        d.received_valid = false;
    }

    let miso_bit = (q.shift_tx >> ((W - 1) as u128)) & one_w;
    let miso_out = (miso_bit != zero_w) && cs_active;

    let mut o = Out::<W>::dont_care();
    o.miso_out = miso_out;
    o.rx_data = q.received_byte;
    o.rx_valid = q.received_valid;
    o.busy = cs_active;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In<8> {
        In {
            sclk_in: false,
            mosi_in: false,
            cs_n_in: true,
            tx_data: bits(0),
        }
    }

    /// Build a stream of inputs that drives one Mode 0 byte at the
    /// slave: assert CS, then 8 (sclk_low, sclk_high) pairs presenting
    /// `byte` MSB-first on MOSI, then deassert CS.
    fn drive_byte(byte: u128, tx_data: u128, idle_after: usize) -> Vec<In<8>> {
        let mut out = Vec::new();
        // Idle: CS high, sclk low for a few cycles.
        for _ in 0..2 {
            let mut inp = idle_in();
            inp.tx_data = bits(tx_data);
            out.push(inp);
        }
        // Assert CS while sclk low for one cycle (latches tx_data).
        out.push(In {
            sclk_in: false,
            mosi_in: false,
            cs_n_in: false,
            tx_data: bits(tx_data),
        });
        // Drive 8 SPI bits.
        for k in 0..8 {
            let bit = ((byte >> (7 - k)) & 1) != 0;
            // Phase 0: sclk low, MOSI presents the bit.
            out.push(In {
                sclk_in: false,
                mosi_in: bit,
                cs_n_in: false,
                tx_data: bits(tx_data),
            });
            // Phase 1: sclk rises, slave samples on the FPGA edge.
            out.push(In {
                sclk_in: true,
                mosi_in: bit,
                cs_n_in: false,
                tx_data: bits(tx_data),
            });
        }
        // Deassert CS.
        for _ in 0..idle_after {
            let mut inp = idle_in();
            inp.tx_data = bits(tx_data);
            out.push(inp);
        }
        out
    }

    // Tier 2 — receive a byte from a simulated master.

    #[test]
    fn test_receive_byte() -> miette::Result<()> {
        for &byte in &[0u128, 0x55, 0xAA, 0xFF, 0xA5, 0x42] {
            let stream = drive_byte(byte, 0, 4)
                .into_iter()
                .with_reset(1)
                .clock_pos_edge(100);
            let uut = SpiSlave::<8, 4>::default();
            let outputs = uut
                .run(stream)
                .synchronous_sample()
                .filter(|s| !s.input.0.reset.any())
                .collect::<Vec<_>>();
            let valid_idx = outputs.iter().position(|s| s.output.rx_valid);
            assert!(valid_idx.is_some(), "no rx_valid pulse for byte {byte:#x}");
            let received = outputs[valid_idx.unwrap()].output.rx_data.raw();
            assert_eq!(received, byte, "mismatch for byte {byte:#x}");
        }
        Ok(())
    }

    #[test]
    fn test_idle_no_valid_pulse() -> miette::Result<()> {
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = SpiSlave::<8, 4>::default();
        let any_valid = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.rx_valid);
        assert!(!any_valid);
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = SpiSlave::<8, 4>::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["9429"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_spi_slave_hdl_works() -> miette::Result<()> {
        let uut = SpiSlave::<8, 4>::default();
        let stream = drive_byte(0xA5, 0x42, 4)
            .into_iter()
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
    fn test_spi_slave_trace() -> miette::Result<()> {
        let uut = SpiSlave::<8, 4>::default();
        let stream = drive_byte(0xA5, 0x42, 8)
            .into_iter()
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("spi_slave");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["08cd20f5126d2ef1fa4622f5a88ca9e20c7295345d308e356d6a0956be9e4d16"];
        let digest = vcd.dump_to_file(root.join("spi_slave.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
