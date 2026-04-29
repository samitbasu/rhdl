//! MIDI TX/RX (wire layer + status-byte tagging)
//!
//! Standard MIDI is a 31.25 kbit/s 8-N-1 byte stream with a
//! message-level FSM on top.  This v1 ships the **wire layer**:
//! a full-duplex UART parameterized for any baud divisor, plus
//! a small piece of state that tags incoming bytes as
//! status (MSB=1) or data (MSB=0) and remembers the most recent
//! status byte to enable running-status-aware downstream parsing.
//!
//! The full message-level decoder (Note On / Note Off / CC / Pitch
//! Bend / SysEx etc.) is deferred — it's a state machine that
//! consumes the byte stream this widget exposes.
//!
//! Composes [super::uart::Uart].
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+Midi+-------+
     |                    |
B<8> |                    | bool
+--->| tx_data        tx  +--->
bool |                    |
+--->| tx_push  rx_byte   +-->Option<B<8>>
     |                    |
bool |                    | bool
+--->| rx_pop   is_status +--->
bool |                    | B<8>
+--->| rx       last_status+-->
     +--------------------+
")]
//!
//!# Internals
//!
//! Wraps [super::uart::Uart] verbatim, adds a `last_status` DFF
//! that latches every received status byte (MSB=1).  The
//! `is_status` output is purely combinational: `rx_byte.is_some()
//! && (byte & 0x80) != 0`.
//!
//!# Parameters
//!
//! - `DIV_W` — bit width of the baud divisor.  For
//!   `f_clk = 100 MHz` and 31250 baud, `divisor = 3200` so
//!   `DIV_W = 12`.
//! - `FIFO_W` — bit width of the inner UART FIFO addresses.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/midi.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/midi.md")]
use rhdl::prelude::*;

use crate::core::dff;
use super::uart::Uart;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// MIDI interface (wire layer).
pub struct MidiInterface<const DIV_W: usize, const FIFO_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    uart: Uart<DIV_W, FIFO_W>,
    last_status: dff::DFF<Bits<8>>,
}

impl<const DIV_W: usize, const FIFO_W: usize> MidiInterface<DIV_W, FIFO_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    /// Create a MIDI interface with the given baud divisor.
    pub fn new(divisor: Bits<DIV_W>) -> Self {
        Self {
            uart: Uart::new(divisor),
            last_status: dff::DFF::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [MidiInterface].
pub struct In {
    /// Byte to push to the TX FIFO.
    pub tx_data: Bits<8>,
    /// Push strobe for the TX FIFO.
    pub tx_push: bool,
    /// Pop strobe for the RX FIFO.
    pub rx_pop: bool,
    /// Serial input line.
    pub rx: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [MidiInterface].
pub struct Out {
    /// Serial output line.
    pub tx: bool,
    /// Head of the RX FIFO (the next received byte, or None).
    pub rx_byte: Option<Bits<8>>,
    /// True if `rx_byte` is a status byte (MSB=1).
    pub is_status: bool,
    /// Most recently observed status byte (held).
    pub last_status: Bits<8>,
    /// True when the TX FIFO is full.
    pub tx_full: bool,
    /// True when the RX FIFO is empty.
    pub rx_empty: bool,
}

impl<const DIV_W: usize, const FIFO_W: usize> SynchronousIO for MidiInterface<DIV_W, FIFO_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = midi<DIV_W, FIFO_W>;
}

#[kernel]
/// Kernel for [MidiInterface].
pub fn midi<const DIV_W: usize, const FIFO_W: usize>(
    cr: ClockReset,
    i: In,
    q: Q<DIV_W, FIFO_W>,
) -> (Out, D<DIV_W, FIFO_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    let mut d = D::<DIV_W, FIFO_W>::dont_care();
    // Forward host I/O to the inner UART.
    d.uart = super::uart::In {
        tx_data: i.tx_data,
        tx_push: i.tx_push,
        rx_pop: i.rx_pop,
        rx: i.rx,
    };
    d.last_status = q.last_status;
    // Status byte detection: RX FIFO head is Some(byte) and bit 7 set.
    let rx_byte = q.uart.rx_data;
    let mut is_status = false;
    let mut new_last_status: Bits<8> = q.last_status;
    if let Some(byte) = rx_byte {
        let msb_set = (byte & bits::<8>(0x80)) != bits::<8>(0);
        if msb_set {
            is_status = true;
            new_last_status = byte;
        }
    }
    d.last_status = new_last_status;

    if cr.reset.any() {
        d.last_status = bits::<8>(0);
    }

    let mut o = Out::dont_care();
    o.tx = q.uart.tx;
    o.rx_byte = rx_byte;
    o.is_status = is_status;
    o.last_status = q.last_status;
    o.tx_full = q.uart.tx_full;
    o.rx_empty = q.uart.rx_empty;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            tx_data: bits(0),
            tx_push: false,
            rx_pop: false,
            rx: true,
        }
    }

    fn encode_frame(byte: u128, divisor: usize) -> Vec<bool> {
        let mut out = vec![true; 4];
        for _ in 0..divisor {
            out.push(false);
        }
        for k in 0..8 {
            let b = ((byte >> k) & 1) != 0;
            for _ in 0..divisor {
                out.push(b);
            }
        }
        for _ in 0..divisor {
            out.push(true);
        }
        out
    }

    // Tier 2 — verify status-byte detection.
    #[test]
    fn test_status_byte_detected() -> miette::Result<()> {
        let divisor = 6;
        let frame = encode_frame(0x90, divisor); // 0x90 = Note On channel 0, status byte.
        let mut stream_in: Vec<In> = Vec::new();
        for &rx in &frame {
            let mut inp = idle_in();
            inp.rx = rx;
            stream_in.push(inp);
        }
        for _ in 0..10 {
            let mut inp = idle_in();
            inp.rx_pop = true;
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let uut = MidiInterface::<6, 4>::new(bits(divisor as u128));
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Look for a cycle where the status byte appears at the FIFO head.
        let status_cycle = outputs
            .iter()
            .find(|s| s.output.rx_byte == Some(bits(0x90)) && s.output.is_status);
        assert!(status_cycle.is_some(), "0x90 status byte not detected");
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = MidiInterface::<6, 4>::new(bits(6));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["59455"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_midi_hdl_works() -> miette::Result<()> {
        let uut = MidiInterface::<6, 4>::new(bits(6));
        let frame = encode_frame(0x90, 6);
        let mut stream_in: Vec<In> = Vec::new();
        for &rx in &frame {
            let mut inp = idle_in();
            inp.rx = rx;
            stream_in.push(inp);
        }
        for _ in 0..10 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest
    #[test]
    fn test_midi_trace() -> miette::Result<()> {
        let uut = MidiInterface::<6, 4>::new(bits(6));
        let frame = encode_frame(0x90, 6);
        let mut stream_in: Vec<In> = Vec::new();
        for &rx in &frame {
            let mut inp = idle_in();
            inp.rx = rx;
            stream_in.push(inp);
        }
        for _ in 0..10 {
            let mut inp = idle_in();
            inp.rx_pop = true;
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("midi");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["d1ac4697b8305b85126c580830aa85ae2c9a76f1f30a381c584a8a4116a6e92a"];
        let digest = vcd.dump_to_file(root.join("midi.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
