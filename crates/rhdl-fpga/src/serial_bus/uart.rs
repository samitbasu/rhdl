//! Full-duplex UART (TX FIFO + RX FIFO)
//!
//! Composes [super::uart_tx::UartTx], [super::uart_rx::UartRx], and
//! two [super::super::fifo::synchronous::SyncFIFO] buffers (one per
//! direction) into a complete byte-stream UART.  The host pushes
//! bytes into the TX FIFO and pops bytes from the RX FIFO; the FIFOs
//! decouple the host's clock-domain rate from the wire's baud rate.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+Uart+-------+
     |                    |
B<8> |                    | bool
+--->| tx_data       tx   +--->
bool |                    |
+--->| tx_push    rx_data +-->Option<B<8>>
     |                    |
bool |                    | bool
+--->| rx_pop     tx_full +--->
bool |                    | bool
+--->| rx               rx_empty+--->
     +--------------------+
")]
//!
//!# Internals
//!
//! Four sub-cores: `tx_fifo`, `tx_uart`, `rx_uart`, `rx_fifo`.
//! Wiring is pure dataflow — no state is added at this level.
//! The `FIFO_W` parameter is the address width of the FIFOs (depth
//! is `2^FIFO_W - 1`); `DIV_W` is forwarded to the inner UART halves.
//!
//!# Parameters
//!
//! - `DIV_W` — bit width of the baud divisor (forwarded to TX/RX)
//! - `FIFO_W` — bit width of the FIFO address (depth = `2^FIFO_W - 1`)
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/uart.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/uart.md")]
use rhdl::prelude::*;

use super::uart_rx::UartRx;
use super::uart_tx::UartTx;
use crate::fifo::synchronous::SyncFIFO;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// Full-duplex UART core.
pub struct Uart<const DIV_W: usize, const FIFO_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    tx_fifo: SyncFIFO<Bits<8>, FIFO_W>,
    tx_uart: UartTx<DIV_W>,
    rx_uart: UartRx<DIV_W>,
    rx_fifo: SyncFIFO<Bits<8>, FIFO_W>,
}

impl<const DIV_W: usize, const FIFO_W: usize> Uart<DIV_W, FIFO_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    /// Create a UART with the given baud divisor.
    pub fn new(divisor: Bits<DIV_W>) -> Self {
        Self {
            tx_fifo: SyncFIFO::default(),
            tx_uart: UartTx::new(divisor),
            rx_uart: UartRx::new(divisor),
            rx_fifo: SyncFIFO::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Uart].
pub struct In {
    /// Byte to push into the TX FIFO (when `tx_push`).
    pub tx_data: Bits<8>,
    /// Strobe to push `tx_data` into the TX FIFO.
    pub tx_push: bool,
    /// Strobe to pop a byte from the RX FIFO.
    pub rx_pop: bool,
    /// Serial input line.
    pub rx: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Uart].
pub struct Out {
    /// Serial output line (idle high).
    pub tx: bool,
    /// Head of the RX FIFO, or `None` if empty.
    pub rx_data: Option<Bits<8>>,
    /// True when the TX FIFO has no slots left.
    pub tx_full: bool,
    /// True when the RX FIFO is empty.
    pub rx_empty: bool,
}

impl<const DIV_W: usize, const FIFO_W: usize> SynchronousIO for Uart<DIV_W, FIFO_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = uart<DIV_W, FIFO_W>;
}

#[kernel]
/// Kernel for [Uart] — pure dataflow wiring.
pub fn uart<const DIV_W: usize, const FIFO_W: usize>(
    _cr: ClockReset,
    i: In,
    q: Q<DIV_W, FIFO_W>,
) -> (Out, D<DIV_W, FIFO_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    let mut d = D::<DIV_W, FIFO_W>::dont_care();
    let mut o = Out::dont_care();

    // TX FIFO inputs: host pushes via tx_push; UART consumes when ready.
    d.tx_fifo.data = if i.tx_push { Some(i.tx_data) } else { None };
    d.tx_fifo.next = q.tx_uart.ready;
    // TX UART inputs: take whatever is at the head of the TX FIFO.
    let tx_head = q.tx_fifo.data;
    let mut tx_in = super::uart_tx::In {
        data: bits::<8>(0),
        send: false,
    };
    if let Some(byte) = tx_head {
        tx_in.data = byte;
        tx_in.send = q.tx_uart.ready;
    }
    d.tx_uart = tx_in;
    // TX line out.
    o.tx = q.tx_uart.tx;
    o.tx_full = q.tx_fifo.full;

    // RX UART input: the wire.
    d.rx_uart = i.rx;
    // RX FIFO inputs: push when UART has a fresh byte.
    d.rx_fifo.data = if q.rx_uart.valid {
        Some(q.rx_uart.received)
    } else {
        None
    };
    d.rx_fifo.next = i.rx_pop;
    // RX outputs.
    o.rx_data = q.rx_fifo.data;
    o.rx_empty = q.rx_fifo.almost_empty;

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

    /// Loopback-ish: push a byte through TX (the wire goes nowhere),
    /// and *separately* drive the RX line with an externally-encoded
    /// frame.  Then pop the RX FIFO and verify the byte appears.
    #[test]
    fn test_rx_path_pushes_into_fifo() -> miette::Result<()> {
        let divisor = 6;
        let frame = encode_frame(0xA5, divisor);
        // Build inputs: drive RX line per the frame, periodically pop the RX FIFO.
        let mut stream_in: Vec<In> = Vec::new();
        for &rx_bit in &frame {
            let mut inp = idle_in();
            inp.rx = rx_bit;
            stream_in.push(inp);
        }
        // Then a few cycles popping the RX FIFO.
        for _ in 0..10 {
            let mut inp = idle_in();
            inp.rx_pop = true;
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let uut = Uart::<6, 4>::new(bits(divisor as u128));
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Look for the first cycle where rx_data is Some.
        let popped = outputs.iter().find_map(|s| s.output.rx_data);
        assert_eq!(popped, Some(bits(0xA5)));
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Uart::<6, 4>::new(bits(8));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["54780"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_uart_hdl_works() -> miette::Result<()> {
        let uut = Uart::<6, 4>::new(bits(6));
        let frame = encode_frame(0xA5, 6);
        let mut stream_in: Vec<In> = Vec::new();
        for &rx_bit in &frame {
            let mut inp = idle_in();
            inp.rx = rx_bit;
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
    fn test_uart_trace() -> miette::Result<()> {
        let uut = Uart::<6, 4>::new(bits(6));
        let frame = encode_frame(0xA5, 6);
        let mut stream_in: Vec<In> = Vec::new();
        for &rx_bit in &frame {
            let mut inp = idle_in();
            inp.rx = rx_bit;
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
            .join("uart");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["7697b11c121480143278e7be3cf3ef818c93fd31571e0227b35f7a8f4130a979"];
        let digest = vcd.dump_to_file(root.join("uart.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
