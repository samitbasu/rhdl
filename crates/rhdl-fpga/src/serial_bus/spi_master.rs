//! SPI master (Mode 0, MSB-first)
//!
//! Standard 4-wire SPI master: drives `sclk`, `mosi`, and `cs_n`,
//! samples `miso`.  This v1 implementation hardcodes **Mode 0**
//! (CPOL=0 / CPHA=0 — `sclk` idle low, master shifts on falling
//! edge, both ends sample on rising edge) and **MSB-first** bit
//! order.  Other modes / bit orders are tracked as follow-ups.
//!
//! The FPGA clock divides 2:1 into `sclk` — one full SPI bit takes
//! two FPGA cycles.  For slower buses, instantiate this widget on
//! a derived/divided clock or wrap it with a strobe-gating front-end.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+SpiMaster+-----+
     |                     |
B<W> |                     | bool
+--->| tx_data       sclk  +--->
bool |                     | bool
+--->| start         mosi  +--->
bool |                     | bool
+--->| miso          cs_n  +--->
     |                     | B<W>
     |             rx_data +--->
     |                     | bool
     |             busy/done+-->
     +---------------------+
")]
//!
//!# Internals
//!
//! - `transferring`: high while a word is in flight.
//! - `bit_counter`: counts `0..W` (each increment = one full SPI bit).
//! - `phase`: half-bit phase.  `false` = `sclk` low (master presents
//!   MOSI), `true` = `sclk` high (slave samples MOSI, master samples
//!   MISO at the rising edge).
//! - `shift_tx`: TX shift register; MSB is sent first.  Shifts left
//!   one bit per SPI bit.
//! - `shift_rx`: RX shift register; sampled MISO bits enter at the
//!   LSB and shift left.  After `W` bits, the first sampled bit
//!   sits at the MSB.
//! - `rx_done`: one-cycle pulse when a word completes.
//!
//!# Behavior
//!
//! - Idle: `cs_n = 1`, `sclk = 0`, `mosi = 0`.  `start` strobe
//!   latches `tx_data` into `shift_tx` and asserts CS for the next
//!   `2*W + 0` FPGA cycles.
//! - During transfer: `sclk` toggles each FPGA cycle.  At the end
//!   of the last bit, `cs_n` deasserts and `done` pulses high for
//!   one cycle with `rx_data` valid.
//!
//!# Parameters
//!
//! - `W` — word width in bits
//! - `CW` — bit width of the bit counter; satisfy `2^CW > W`
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/spi_master.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/spi_master.md")]
use rhdl::prelude::*;

use crate::core::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
/// SPI master core (Mode 0, MSB-first).
pub struct SpiMaster<const W: usize, const CW: usize>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    transferring: dff::DFF<bool>,
    bit_counter: dff::DFF<Bits<CW>>,
    phase: dff::DFF<bool>,
    shift_tx: dff::DFF<Bits<W>>,
    shift_rx: dff::DFF<Bits<W>>,
    rx_done: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [SpiMaster].
pub struct In<const W: usize>
where
    rhdl::bits::W<W>: BitWidth,
{
    /// Word to transmit (latched at `start`).
    pub tx_data: Bits<W>,
    /// Strobe to begin a transfer.  Ignored while `busy`.
    pub start: bool,
    /// Serial in from slave.
    pub miso: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [SpiMaster].
pub struct Out<const W: usize>
where
    rhdl::bits::W<W>: BitWidth,
{
    /// Serial clock to slave.  Idle low (CPOL=0).
    pub sclk: bool,
    /// Serial out to slave.  MSB-first.
    pub mosi: bool,
    /// Chip select, active low.  Asserted (low) during transfer.
    pub cs_n: bool,
    /// Last fully-received word.  Held until next transfer.
    pub rx_data: Bits<W>,
    /// High while a word is being transferred.
    pub busy: bool,
    /// Pulses high for one cycle at end of transfer with `rx_data` valid.
    pub done: bool,
}

impl<const W: usize, const CW: usize> SynchronousIO for SpiMaster<W, CW>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    type I = In<W>;
    type O = Out<W>;
    type Kernel = spi_master<W, CW>;
}

#[kernel]
/// Kernel for [SpiMaster].
pub fn spi_master<const W: usize, const CW: usize>(
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

    let mut d = D::<W, CW>::dont_care();
    // Default: hold all state; pulse default low.
    d.transferring = q.transferring;
    d.bit_counter = q.bit_counter;
    d.phase = q.phase;
    d.shift_tx = q.shift_tx;
    d.shift_rx = q.shift_rx;
    d.rx_done = false;

    if !q.transferring {
        if i.start {
            d.transferring = true;
            d.bit_counter = zero_cw;
            d.phase = false;
            d.shift_tx = i.tx_data;
            d.shift_rx = zero_w;
        }
    } else {
        d.phase = !q.phase;
        if !q.phase {
            // sclk transitioning low → high (rising edge): sample MISO.
            // The latched value lands in shift_rx for the next cycle,
            // which is exactly the rising-edge capture semantics.
            let miso_bit: Bits<W> = if i.miso { one_w } else { zero_w };
            d.shift_rx = (q.shift_rx << 1) | miso_bit;
        } else {
            // sclk transitioning high → low (falling edge): master
            // shifts to next bit and advances counter.
            d.shift_tx = q.shift_tx << 1;
            let next_count = q.bit_counter + one_cw;
            d.bit_counter = next_count;
            if next_count == w_cw {
                d.transferring = false;
                d.bit_counter = zero_cw;
                d.phase = false;
                d.rx_done = true;
            }
        }
    }

    if cr.reset.any() {
        d.transferring = false;
        d.bit_counter = zero_cw;
        d.phase = false;
        d.shift_tx = zero_w;
        d.shift_rx = zero_w;
        d.rx_done = false;
    }

    // Outputs are derived from current state q.
    let cs_n = !q.transferring;
    let sclk = q.phase;
    let mosi_bit = (q.shift_tx >> ((W - 1) as u128)) & one_w;
    let mosi = mosi_bit != zero_w;

    let mut o = Out::<W>::dont_care();
    o.sclk = sclk;
    o.mosi = mosi;
    o.cs_n = cs_n;
    o.rx_data = q.shift_rx;
    o.busy = q.transferring;
    o.done = q.rx_done;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In<8> {
        In {
            tx_data: bits(0),
            start: false,
            miso: false,
        }
    }

    /// Run a single 8-bit transfer, with a slave that echoes
    /// `slave_response` MSB-first onto MISO.  Returns the master's
    /// view of the transfer: (rx_data, mosi_sequence, sclk_cycles).
    fn run_one_transfer(tx: u128, slave_response: u128) -> (u128, Vec<bool>, usize) {
        // We need to drive MISO based on the SPI bit position the
        // slave is "currently presenting".  In Mode 0, the slave
        // presents bit 0 (MSB) starting at /CS assertion, and changes
        // bits on each falling sclk.  Since this is a simulated test,
        // we model the slave as a function of sclk transitions.
        //
        // Simpler: run the master with a fixed MISO pattern that
        // matches what the slave would present, by pre-computing the
        // MISO sequence aligned to the cycle stream.
        //
        // The master starts at cycle 0 (idle, start=true).
        // - Cycle 0: idle, start=true
        // - Cycle 1: transferring, phase=0, MOSI=tx[7] (bit 0 sent),
        //            MISO sampled = slave bit 0 (slave's MSB)
        // - Cycle 2: phase=1
        // - Cycle 3: phase=0, MISO sampled = slave bit 1
        // ...
        // So MISO at cycle (2k+1) for k in 0..8 = slave_response bit (7-k).
        let n_cycles = 1 + 2 * 8 + 4; // start + 16 transfer + slack
        let mut stream_in: Vec<In<8>> = Vec::with_capacity(n_cycles);
        for cycle in 0..n_cycles {
            let mut inp = idle_in();
            // slave bits: slave_response[7] first (MSB-first response).
            let bit_idx = if cycle >= 1 && cycle % 2 == 1 {
                let k = (cycle - 1) / 2;
                if k < 8 {
                    Some(7 - k)
                } else {
                    None
                }
            } else {
                None
            };
            inp.miso = bit_idx
                .map(|b| ((slave_response >> b) & 1) != 0)
                .unwrap_or(false);
            if cycle == 0 {
                inp.tx_data = bits(tx);
                inp.start = true;
            }
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let uut = SpiMaster::<8, 4>::default();
        let outputs = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect::<Vec<_>>();
        let done_cycle = outputs.iter().position(|s| s.output.done).unwrap();
        let rx = outputs[done_cycle].output.rx_data.raw();
        // Capture mosi sequence at cycles where SPI bit is being presented (q.phase=0 cycles).
        let mosi_seq: Vec<bool> = outputs
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.output.busy && !s.output.sclk {
                    Some(s.output.mosi)
                } else {
                    None
                }
            })
            .collect();
        // Count sclk rising edges by looking at sclk transitions.
        let mut rises = 0;
        for w in outputs.windows(2) {
            if !w[0].output.sclk && w[1].output.sclk && w[0].output.busy && w[1].output.busy {
                rises += 1;
            }
        }
        let _ = rises;
        let _ = mosi_seq.clone();
        (rx, mosi_seq, done_cycle)
    }

    // Tier 2 — round-trip tests

    #[test]
    fn test_single_transfer_round_trip() -> miette::Result<()> {
        for &(tx, slave) in &[
            (0x00u128, 0x00u128),
            (0xFF, 0xFF),
            (0xA5, 0x5A),
            (0x42, 0x18),
            (0x01, 0x80),
            (0x80, 0x01),
        ] {
            let (rx, mosi, _) = run_one_transfer(tx, slave);
            assert_eq!(rx, slave, "rx mismatch tx=0x{tx:x} slave=0x{slave:x}");
            // mosi bits should equal tx in MSB-first order.
            assert!(mosi.len() >= 8, "mosi too short: {mosi:?}");
            // The first 8 mosi bits captured (during transferring && sclk=0) are tx[7], tx[6], ..., tx[0].
            for k in 0..8 {
                let expected_bit = ((tx >> (7 - k)) & 1) != 0;
                assert_eq!(mosi[k], expected_bit, "mosi[{k}] mismatch for tx=0x{tx:x}");
            }
        }
        Ok(())
    }

    #[test]
    fn test_idle_state_outputs() -> miette::Result<()> {
        // No start strobe ever — outputs should stay in idle.
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = SpiMaster::<8, 4>::default();
        let any_busy = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.busy || !s.output.cs_n || s.output.sclk);
        assert!(!any_busy, "outputs should stay idle");
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = SpiMaster::<8, 4>::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["9081"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_spi_master_hdl_works() -> miette::Result<()> {
        let uut = SpiMaster::<8, 4>::default();
        let mut stream_in: Vec<In<8>> = vec![In {
            tx_data: bits(0xA5),
            start: true,
            miso: false,
        }];
        for _ in 0..30 {
            stream_in.push(idle_in());
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
    fn test_spi_master_trace() -> miette::Result<()> {
        let uut = SpiMaster::<8, 4>::default();
        let mut stream_in: Vec<In<8>> = vec![In {
            tx_data: bits(0xA5),
            start: true,
            miso: false,
        }];
        for _ in 0..30 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("spi_master");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["ec5010ed9e8056e0fc77415071213fb8e3ce4eb18dbf63082a992048e9d2ec23"];
        let digest = vcd.dump_to_file(root.join("spi_master.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
