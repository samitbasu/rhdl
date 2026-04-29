//! UART transmitter (8-N-1)
//!
//! Standard 8-data-bit, no-parity, 1-stop-bit UART transmitter.
//! Sends one byte at a time, LSB-first, framed with a low start
//! bit and a high stop bit (idle line is high).  The baud-period
//! divisor is supplied at construction time as `divisor` clock
//! cycles per baud — pick `divisor = f_clk / baud` (rounded to
//! the nearest integer).
//!
//! For example, on a 100 MHz clock targeting 115200 baud:
//! `divisor = 100_000_000 / 115200 ≈ 868`.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+UartTx+-----+
     |                  |
B<8> |                  | bool
+--->| data         tx  +--->
bool |                  | bool
+--->| send         busy+--->
     |              ready+---->
     +------------------+
")]
//!
//!# Internals
//!
//! - `transmitting`: bool DFF — high while a frame is in flight.
//! - `bit_counter`: 4-bit DFF counting `0..=9` (start bit, 8 data,
//!   stop bit).  The LSB-first data ordering puts `data[0]` at
//!   `bit_counter == 1` and `data[7]` at `bit_counter == 8`.
//! - `baud_counter`: counts `0..=divisor-1` clock cycles per baud,
//!   advancing `bit_counter` when it hits `divisor-1`.
//! - `data_reg`: latches the input byte at the cycle `send` is
//!   asserted.
//! - `divisor`: constant subcore holding the clocks-per-baud value
//!   supplied at construction.
//!
//!# Behavior
//!
//! - When `transmitting == false`: line is high, `ready == true`,
//!   `busy == false`.  A `send == true` strobe latches `data` and
//!   begins transmission on the next cycle.
//! - During transmission: `tx` walks the frame bits in order
//!   (start = 0, then `data[0..=7]`, then stop = 1), each held for
//!   `divisor` clock cycles.  `busy == true`, `ready == false`,
//!   `send` is ignored.
//! - When the stop bit completes: `transmitting` drops, `ready`
//!   reasserts, line returns to idle high.
//!
//!# Parameters
//!
//! - `DIV_W` — bit width of the baud counter.  Must hold values up
//!   to `divisor - 1`.  For 100 MHz / 115200 baud, `DIV_W = 10` is
//!   sufficient (`868 < 1024`).
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/uart_tx.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/uart_tx.md")]
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// UART transmitter (8-N-1) core.
pub struct UartTx<const DIV_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    transmitting: dff::DFF<bool>,
    bit_counter: dff::DFF<Bits<4>>,
    baud_counter: dff::DFF<Bits<DIV_W>>,
    data_reg: dff::DFF<Bits<8>>,
    divisor: Constant<Bits<DIV_W>>,
}

impl<const DIV_W: usize> UartTx<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    /// Create a UART transmitter with the supplied clocks-per-baud divisor.
    pub fn new(divisor: Bits<DIV_W>) -> Self {
        Self {
            transmitting: dff::DFF::default(),
            bit_counter: dff::DFF::default(),
            baud_counter: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            divisor: Constant::new(divisor),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [UartTx].
pub struct In {
    /// Byte to transmit (latched when `send` is asserted).
    pub data: Bits<8>,
    /// Strobe to start transmission.  Ignored while `busy`.
    pub send: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [UartTx].
pub struct Out {
    /// The serial line (idle high).
    pub tx: bool,
    /// High while a frame is being transmitted.
    pub busy: bool,
    /// High when ready to accept a new byte.
    pub ready: bool,
}

impl<const DIV_W: usize> SynchronousIO for UartTx<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = uart_tx<DIV_W>;
}

#[kernel]
/// Kernel for [UartTx].
pub fn uart_tx<const DIV_W: usize>(cr: ClockReset, i: In, q: Q<DIV_W>) -> (Out, D<DIV_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    let one_div: Bits<DIV_W> = bits::<DIV_W>(1);
    let zero_div: Bits<DIV_W> = bits::<DIV_W>(0);
    let one_b4: Bits<4> = bits::<4>(1);
    let zero_b4: Bits<4> = bits::<4>(0);
    let nine_b4: Bits<4> = bits::<4>(9);
    let zero_b8: Bits<8> = bits::<8>(0);

    let baud_tick = q.baud_counter == (q.divisor - one_div);
    let last_bit = q.bit_counter == nine_b4;

    let mut d = D::<DIV_W>::dont_care();
    // Default: hold all state.
    d.transmitting = q.transmitting;
    d.bit_counter = q.bit_counter;
    d.baud_counter = q.baud_counter;
    d.data_reg = q.data_reg;

    if !q.transmitting && i.send {
        // Load and arm.
        d.data_reg = i.data;
        d.transmitting = true;
        d.bit_counter = zero_b4;
        d.baud_counter = zero_div;
    } else if q.transmitting {
        if baud_tick {
            d.baud_counter = zero_div;
            if last_bit {
                d.transmitting = false;
                d.bit_counter = zero_b4;
            } else {
                d.bit_counter = q.bit_counter + one_b4;
            }
        } else {
            d.baud_counter = q.baud_counter + one_div;
        }
    }

    // Compute tx output.  bit_counter encoding:
    //   0 = start bit (low)
    //   1..=8 = data[0..=7] (LSB first)
    //   9 = stop bit (high)
    // bit_idx = bit_counter - 1, used to index data; mask to [0,7]
    // so that the always-evaluated mux input doesn't trip the VM
    // shift-bound check on out-of-range bit_counter values.
    let bit_idx_raw: Bits<4> = q.bit_counter - one_b4;
    let mask_b4: Bits<4> = bits::<4>(0b111);
    let bit_idx_safe: Bits<4> = bit_idx_raw & mask_b4;
    let data_bit: Bits<8> = (q.data_reg >> bit_idx_safe) & bits::<8>(1);
    let data_bit_b = data_bit != zero_b8;
    let is_start = q.bit_counter == zero_b4;
    let is_stop = q.bit_counter == nine_b4;

    let tx = if !q.transmitting {
        true
    } else if is_start {
        false
    } else if is_stop {
        true
    } else {
        data_bit_b
    };

    let busy = q.transmitting;
    let ready = !q.transmitting;

    let mut o = Out::dont_care();
    o.tx = tx;
    o.busy = busy;
    o.ready = ready;

    if cr.reset.any() {
        d.transmitting = false;
        d.bit_counter = zero_b4;
        d.baud_counter = zero_div;
        d.data_reg = zero_b8;
    }
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
            send: false,
        }
    }

    fn q4(
        transmitting: bool,
        bit_counter: u128,
        baud_counter: u128,
        data_reg: u128,
        divisor: u128,
    ) -> Q<6> {
        Q::<6> {
            transmitting,
            bit_counter: bits(bit_counter),
            baud_counter: bits(baud_counter),
            data_reg: bits(data_reg),
            divisor: bits(divisor),
        }
    }

    // Tier 1 — direct kernel unit tests

    #[test]
    fn test_idle_holds_high_and_ready() {
        let cr = ClockReset::dont_care();
        let q = q4(false, 0, 0, 0, 4);
        let (o, _d) = uart_tx::<6>(cr, idle_in(), q);
        assert!(o.tx);
        assert!(!o.busy);
        assert!(o.ready);
    }

    #[test]
    fn test_send_strobe_latches_data() {
        let cr = ClockReset::dont_care();
        let q = q4(false, 0, 0, 0, 4);
        let i = In {
            data: bits(0xA5),
            send: true,
        };
        let (_o, d) = uart_tx::<6>(cr, i, q);
        assert!(d.transmitting);
        assert_eq!(d.data_reg, bits(0xA5));
        assert_eq!(d.bit_counter, bits(0));
        assert_eq!(d.baud_counter, bits(0));
    }

    #[test]
    fn test_send_ignored_while_busy() {
        let cr = ClockReset::dont_care();
        let q = q4(true, 3, 1, 0xA5, 4);
        let i = In {
            data: bits(0xFF),
            send: true,
        };
        let (_o, d) = uart_tx::<6>(cr, i, q);
        // data_reg unchanged
        assert_eq!(d.data_reg, bits(0xA5));
    }

    #[test]
    fn test_baud_counter_increments() {
        let cr = ClockReset::dont_care();
        let q = q4(true, 0, 1, 0xA5, 4);
        let (_o, d) = uart_tx::<6>(cr, idle_in(), q);
        assert_eq!(d.baud_counter, bits(2));
        assert_eq!(d.bit_counter, bits(0));
    }

    #[test]
    fn test_baud_tick_advances_bit_counter() {
        let cr = ClockReset::dont_care();
        // baud_counter = divisor - 1 = 3
        let q = q4(true, 0, 3, 0xA5, 4);
        let (_o, d) = uart_tx::<6>(cr, idle_in(), q);
        assert_eq!(d.baud_counter, bits(0));
        assert_eq!(d.bit_counter, bits(1));
        assert!(d.transmitting);
    }

    #[test]
    fn test_last_bit_returns_to_idle() {
        let cr = ClockReset::dont_care();
        // bit_counter = 9 (stop bit), baud_counter at end
        let q = q4(true, 9, 3, 0xA5, 4);
        let (_o, d) = uart_tx::<6>(cr, idle_in(), q);
        assert!(!d.transmitting);
        assert_eq!(d.bit_counter, bits(0));
    }

    #[test]
    fn test_tx_bit_at_each_position() {
        let cr = ClockReset::dont_care();
        // data = 0b10101010 = 0xAA. Frame (LSB-first): start=0,
        // data[0..=7] = 0,1,0,1,0,1,0,1, stop=1
        for (counter, expected) in [
            (0u128, false), // start
            (1, false),     // data[0] = 0
            (2, true),      // data[1] = 1
            (3, false),     // data[2] = 0
            (4, true),      // data[3] = 1
            (5, false),     // data[4] = 0
            (6, true),      // data[5] = 1
            (7, false),     // data[6] = 0
            (8, true),      // data[7] = 1
            (9, true),      // stop
        ] {
            let q = q4(true, counter, 0, 0xAA, 4);
            let (o, _d) = uart_tx::<6>(cr, idle_in(), q);
            assert_eq!(o.tx, expected, "counter={counter}, data=0xAA");
        }
    }

    #[test]
    fn test_reset_clears_state() {
        let cr = clock_reset(clock(true), reset(true));
        let q = q4(true, 5, 2, 0xA5, 4);
        let (_o, d) = uart_tx::<6>(cr, idle_in(), q);
        assert!(!d.transmitting);
        assert_eq!(d.bit_counter, bits(0));
        assert_eq!(d.baud_counter, bits(0));
        assert_eq!(d.data_reg, bits(0));
    }

    // Tier 2 — iterator simulation: send a byte and decode the line

    /// Sample the `tx` line at the middle of each baud period and
    /// verify the recovered byte equals what was sent.
    fn run_tx_and_decode(byte: u128, divisor: u128) -> (u128, Vec<bool>) {
        let div_w = 6;
        let _ = div_w;
        let uut = UartTx::<6>::new(bits(divisor));
        // 1 cycle of send strobe, then enough idle cycles for the
        // frame to complete: 10 bits × divisor + slack.
        let frame_cycles = (10 * divisor + 5) as usize;
        let mut stream_in: Vec<In> = vec![In {
            data: bits(byte),
            send: true,
        }];
        for _ in 0..frame_cycles {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .map(|s| s.output.tx)
            .collect::<Vec<_>>();
        // The frame begins one cycle after the send strobe (state
        // machine takes one cycle to enter Transmitting).
        // Bit `k` of the frame is at cycles
        //   start_offset + k*divisor .. start_offset + (k+1)*divisor
        // We sample at the *middle* of each baud period for robustness.
        let start_offset = 1usize; // one cycle of send-latency
        let div = divisor as usize;
        let mid = div / 2;
        let mut bits_out: Vec<bool> = Vec::new();
        for k in 0..10 {
            let sample_idx = start_offset + k * div + mid;
            bits_out.push(outputs[sample_idx]);
        }
        // Decode: bits_out[0] = start (should be 0), bits_out[1..=8] = data[0..=7], bits_out[9] = stop (should be 1).
        let mut decoded = 0u128;
        for k in 0..8 {
            if bits_out[1 + k] {
                decoded |= 1 << k;
            }
        }
        (decoded, bits_out)
    }

    #[test]
    fn test_send_byte_round_trip() -> miette::Result<()> {
        for byte in [0u128, 1, 0x55, 0xAA, 0xFF, 0x42, 0xA5] {
            let (decoded, frame) = run_tx_and_decode(byte, 4);
            assert!(!frame[0], "start bit not 0 for byte {byte:#x}: {frame:?}");
            assert!(frame[9], "stop bit not 1 for byte {byte:#x}: {frame:?}");
            assert_eq!(decoded, byte, "decoded byte mismatch for {byte:#x}");
        }
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = UartTx::<6>::new(bits(4));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["7457"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_uart_tx_hdl_works() -> miette::Result<()> {
        let uut = UartTx::<6>::new(bits(4));
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
        }];
        for _ in 0..50 {
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
    fn test_uart_tx_trace() -> miette::Result<()> {
        let uut = UartTx::<6>::new(bits(4));
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
        }];
        for _ in 0..50 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("uart_tx");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["023ead5016a525b257a4b55160d0883b90862b6cf77c7c8eff0144fe8e0a5af2"];
        let digest = vcd.dump_to_file(root.join("uart_tx.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
