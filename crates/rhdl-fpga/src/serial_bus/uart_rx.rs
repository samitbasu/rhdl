//! UART receiver (8-N-1)
//!
//! Standard 8-data-bit, no-parity, 1-stop-bit UART receiver.
//! Recovers a byte from the serial line by detecting the falling
//! edge of the start bit, sampling each subsequent bit at the
//! center of its baud period, and emitting a one-cycle `valid`
//! pulse once the stop bit completes.
//!
//! The line is sampled directly — for a real metastability-safe
//! receiver, run `rx` through [super::super::cdc::synchronizer::Sync1Bit]
//! (or the N-stage chain) before feeding it to this widget.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+UartRx+-----+
     |                  |
bool |                  | B<8>
+--->| rx       received+--->
     |                  | bool
     |             valid+--->
     |              busy+--->
     +------------------+
")]
//!
//!# Internals
//!
//! - `prev_rx`: registers `rx` from the previous cycle for falling-edge detection.
//! - `receiving`: high while a frame is being read.
//! - `bit_counter`: 4-bit DFF counting `0..=9` (start = 0, data = 1..=8, stop = 9).
//! - `baud_counter`: counts `0..=divisor-1` clock cycles per baud.
//!   Sampled at `divisor / 2` (mid-bit) for noise immunity.
//! - `shift_reg`: 8-bit shift register collecting data bits MSB-in,
//!   so after 8 LSB-first samples the full byte sits with `data[7]`
//!   at MSB and `data[0]` at LSB.
//! - `received_byte`: latched copy of `shift_reg` once the frame
//!   completes.  Held until the next reception.
//! - `received_valid`: pulses high for one cycle after each
//!   successful reception.
//! - `divisor`: constant subcore holding the clocks-per-baud value.
//!
//!# Behavior
//!
//! - Idle: line is high.  When `rx` falls, the receiver enters
//!   `Receiving` with `bit_counter = 0` and `baud_counter = 0`.
//! - Mid-baud sample at `bit_counter = 0`: if the line is back high,
//!   abort (false start).  Otherwise continue.
//! - For `bit_counter = 1..=8`: sample mid-baud and shift into
//!   `shift_reg`.
//! - For `bit_counter = 9`: stop-bit period.  Value is *not* checked
//!   in this implementation (see follow-ups for frame-error reporting).
//! - At the end of the stop-bit period: latch `shift_reg` into
//!   `received_byte` and pulse `valid` for one cycle.
//!
//!# Constraints
//!
//! - `divisor >= 4`.  Mid-baud sample (`divisor / 2`) and
//!   end-of-baud (`divisor - 1`) must be distinct cycles.
//! - For metastability safety, synchronize `rx` to this widget's
//!   clock with [super::super::cdc::synchronizer::Sync1Bit] *before*
//!   it reaches the receiver.
//!
//!# Parameters
//!
//! - `DIV_W` — bit width of the baud counter; must hold values up to `divisor - 1`.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/uart_rx.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/uart_rx.md")]
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// UART receiver (8-N-1) core.
pub struct UartRx<const DIV_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    prev_rx: dff::DFF<bool>,
    receiving: dff::DFF<bool>,
    bit_counter: dff::DFF<Bits<4>>,
    baud_counter: dff::DFF<Bits<DIV_W>>,
    shift_reg: dff::DFF<Bits<8>>,
    received_byte: dff::DFF<Bits<8>>,
    received_valid: dff::DFF<bool>,
    divisor: Constant<Bits<DIV_W>>,
}

impl<const DIV_W: usize> UartRx<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    /// Create a UART receiver with the supplied clocks-per-baud divisor.
    pub fn new(divisor: Bits<DIV_W>) -> Self {
        Self {
            prev_rx: dff::DFF::new(true), // line idle high
            receiving: dff::DFF::default(),
            bit_counter: dff::DFF::default(),
            baud_counter: dff::DFF::default(),
            shift_reg: dff::DFF::default(),
            received_byte: dff::DFF::default(),
            received_valid: dff::DFF::default(),
            divisor: Constant::new(divisor),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [UartRx].
pub struct Out {
    /// Last fully-received byte.  Held until the next reception.
    pub received: Bits<8>,
    /// Pulses high for one cycle after each successful reception.
    pub valid: bool,
    /// High while a frame is being received.
    pub busy: bool,
}

impl<const DIV_W: usize> SynchronousIO for UartRx<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    type I = bool;
    type O = Out;
    type Kernel = uart_rx<DIV_W>;
}

#[kernel]
/// Kernel for [UartRx].
pub fn uart_rx<const DIV_W: usize>(cr: ClockReset, rx: bool, q: Q<DIV_W>) -> (Out, D<DIV_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    let one_div: Bits<DIV_W> = bits::<DIV_W>(1);
    let zero_div: Bits<DIV_W> = bits::<DIV_W>(0);
    let one_b4: Bits<4> = bits::<4>(1);
    let zero_b4: Bits<4> = bits::<4>(0);
    let nine_b4: Bits<4> = bits::<4>(9);
    let eight_b4: Bits<4> = bits::<4>(8);
    let zero_b8: Bits<8> = bits::<8>(0);

    let half_div: Bits<DIV_W> = q.divisor >> 1;
    let baud_tick = q.baud_counter == (q.divisor - one_div);
    let sample_tick = q.baud_counter == half_div;
    let falling_edge = q.prev_rx && !rx;

    let mut d = D::<DIV_W>::dont_care();
    // Default: hold all state.
    d.prev_rx = rx;
    d.receiving = q.receiving;
    d.bit_counter = q.bit_counter;
    d.baud_counter = q.baud_counter;
    d.shift_reg = q.shift_reg;
    d.received_byte = q.received_byte;
    // Valid is a one-cycle pulse: default low, set true only on success.
    d.received_valid = false;

    if !q.receiving {
        if falling_edge {
            d.receiving = true;
            d.bit_counter = zero_b4;
            d.baud_counter = zero_div;
            d.shift_reg = zero_b8;
        }
    } else {
        // Mid-baud sample: shift in data bit, or check start/stop.
        if sample_tick {
            if q.bit_counter == zero_b4 {
                // Start bit re-check.  If line went high, abort (false start).
                if rx {
                    d.receiving = false;
                    d.bit_counter = zero_b4;
                }
            } else if q.bit_counter <= eight_b4 {
                // Data bit.  Shift right, new bit at MSB (LSB-first protocol
                // means the first sampled bit ends up at LSB after 8 shifts).
                let bit_in: Bits<8> = if rx { bits::<8>(0x80) } else { zero_b8 };
                d.shift_reg = (q.shift_reg >> 1) | bit_in;
            }
            // bit_counter == 9 → stop bit; value not checked.
        }
        // End-of-baud: advance bit_counter.
        if baud_tick {
            d.baud_counter = zero_div;
            if q.bit_counter == nine_b4 {
                d.receiving = false;
                d.bit_counter = zero_b4;
                d.received_byte = q.shift_reg;
                d.received_valid = true;
            } else {
                d.bit_counter = q.bit_counter + one_b4;
            }
        } else {
            d.baud_counter = q.baud_counter + one_div;
        }
    }

    if cr.reset.any() {
        d.prev_rx = true;
        d.receiving = false;
        d.bit_counter = zero_b4;
        d.baud_counter = zero_div;
        d.shift_reg = zero_b8;
        d.received_byte = zero_b8;
        d.received_valid = false;
    }

    let mut o = Out::dont_care();
    o.received = q.received_byte;
    o.valid = q.received_valid;
    o.busy = q.receiving;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    /// Encode a byte as a UART 8-N-1 frame at the given divisor.
    /// Returns the per-cycle `rx` line, including idle padding.
    fn encode_frame(
        byte: u128,
        divisor: usize,
        idle_before: usize,
        idle_after: usize,
    ) -> Vec<bool> {
        let mut out = vec![true; idle_before];
        // Start bit (low) for `divisor` cycles.
        for _ in 0..divisor {
            out.push(false);
        }
        // 8 data bits, LSB-first.
        for k in 0..8 {
            let bit = ((byte >> k) & 1) != 0;
            for _ in 0..divisor {
                out.push(bit);
            }
        }
        // Stop bit (high) for `divisor` cycles.
        for _ in 0..divisor {
            out.push(true);
        }
        // Idle.
        for _ in 0..idle_after {
            out.push(true);
        }
        out
    }

    // Tier 2 — receive a single byte and check it against the encoded value.

    #[test]
    fn test_receive_single_byte() -> miette::Result<()> {
        for byte in [0u128, 1, 0x55, 0xAA, 0xFF, 0x42, 0xA5] {
            let divisor = 8;
            let frame = encode_frame(byte, divisor, 8, 8);
            let stream = frame.into_iter().with_reset(1).clock_pos_edge(100);
            let uut = UartRx::<6>::new(bits(divisor as u128));
            let outputs = uut
                .run(stream)
                .synchronous_sample()
                .filter(|s| !s.input.0.reset.any())
                .collect::<Vec<_>>();
            // Find the cycle where `valid` pulses.
            let valid_cycle = outputs.iter().position(|s| s.output.valid);
            assert!(
                valid_cycle.is_some(),
                "no valid pulse for byte {byte:#x}: outputs[..15]={:?}",
                &outputs[..15.min(outputs.len())]
                    .iter()
                    .map(|s| s.output.valid)
                    .collect::<Vec<_>>()
            );
            let received = outputs[valid_cycle.unwrap()].output.received.raw();
            assert_eq!(received, byte, "wrong byte received");
        }
        Ok(())
    }

    #[test]
    fn test_receive_multiple_bytes_back_to_back() -> miette::Result<()> {
        let bytes = [0x11u128, 0x22, 0x33, 0x44];
        let divisor = 8;
        let mut frame = vec![true; 8]; // initial idle
        for &byte in &bytes {
            // Each frame: no extra inter-byte gap needed (next start can come immediately).
            frame.extend(encode_frame(byte, divisor, 0, 2));
        }
        frame.extend(vec![true; 8]); // trailing idle
        let stream = frame.into_iter().with_reset(1).clock_pos_edge(100);
        let uut = UartRx::<6>::new(bits(divisor as u128));
        let received: Vec<u128> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.valid)
            .map(|s| s.output.received.raw())
            .collect();
        assert_eq!(received, bytes.to_vec());
        Ok(())
    }

    #[test]
    fn test_idle_line_no_valid_pulse() -> miette::Result<()> {
        let stream = std::iter::repeat_n(true, 100)
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = UartRx::<6>::new(bits(8));
        let any_valid = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.valid);
        assert!(!any_valid);
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = UartRx::<6>::new(bits(8));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["10169"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_uart_rx_hdl_works() -> miette::Result<()> {
        let uut = UartRx::<6>::new(bits(8));
        let frame = encode_frame(0xA5, 8, 8, 16);
        let stream = frame.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest
    #[test]
    fn test_uart_rx_trace() -> miette::Result<()> {
        let uut = UartRx::<6>::new(bits(8));
        let frame = encode_frame(0xA5, 8, 8, 16);
        let stream = frame.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("uart_rx");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["b2d9315b951654737d2abbfc810711e6a3e0c7864d31cda91c765da276274286"];
        let digest = vcd.dump_to_file(root.join("uart_rx.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
