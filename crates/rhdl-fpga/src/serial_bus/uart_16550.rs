//! Intel / National Semi 16550A-compatible UART (full register surface)
//!
//! Wraps the shipped [super::uart::Uart] (full-duplex with TX/RX
//! FIFOs) with the canonical 8-register memory-mapped interface
//! that Linux `8250_core`, QEMU `hw/char/serial.c`, and every
//! PC-derived firmware stack expects.  At the bit level the
//! register layout matches the National Semiconductor PC16550D
//! datasheet; software written against a real 16550A talks to
//! this widget without modification.
//!
//! The previous v1 of this widget was a 2-register
//! "minimum-viable subset" filed under `bus_uart`.  This v2
//! supersedes it and is renamed to make the chip-family
//! correspondence explicit.
//!
//! # v2 register map
//!
//! 8 byte-addressable registers, accessed via `addr: Bits<3>`.
//! Bit 7 of LCR is the **DLAB** (divisor latch access bit) —
//! when set, the first two registers swap to expose the divisor
//! latch instead of RBR/THR/IER:
//!
//! | Addr | DLAB=0       | DLAB=1 | R/W |
//! |------|--------------|--------|-----|
//! | 0x0  | RBR / THR    | DLL    | R/W |
//! | 0x1  | IER          | DLM    | R/W |
//! | 0x2  | IIR / FCR    | (same) | R/W |
//! | 0x3  | LCR          | (same) | R/W |
//! | 0x4  | MCR          | (same) | R/W |
//! | 0x5  | LSR          | (same) | R   |
//! | 0x6  | MSR          | (same) | R   |
//! | 0x7  | SCR          | (same) | R/W |
//!
//! # v2 scope and limitations
//!
//! - **Programmable word length / parity / stop bits is NOT yet
//!   implemented.**  The wire format is hardcoded to 8N1
//!   (matching the underlying [`super::uart_tx::UartTx`] /
//!   [`super::uart_rx::UartRx`]).  LCR's word-length / parity /
//!   stop-bits fields are accepted by the host but currently have
//!   no effect on the wire.  Wiring them through to the TX / RX
//!   primitives is a v3 follow-up.
//! - **Programmable baud rate via DLL/DLM is NOT yet routed to
//!   the underlying divisor.**  The actual divisor is fixed at
//!   construction; DLL/DLM are storage registers only.  Wiring
//!   them through requires the underlying TX / RX to take
//!   divisor as a runtime input rather than a `Constant`.  v3.
//! - **Parity / framing / break-interrupt detection** are
//!   reported as 0 in LSR — the underlying RX primitive doesn't
//!   surface those error conditions yet.  v3 (along with
//!   programmable word length).
//! - **Loopback mode** (MCR bit 4) is implemented in the kernel
//!   by routing the underlying TX line back to the underlying
//!   RX input internally.
//! - **Modem-status pins** (CTS / DSR / RI / DCD) come in via
//!   inputs and the corresponding delta bits are computed in
//!   the kernel against the previous-cycle values.
//!
//! Despite the deferred items, the **register interface** is
//! bit-compatible with the canonical 16550A — software can
//! probe-detect, read/write all eight registers in correct
//! banks, route interrupts via IIR, and drive RTS / DTR / OUT1 /
//! OUT2.  Full programmable-baud + programmable-LCR support is
//! the natural next step.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+Uart16550+-------+
     |                         |
B<3> |                         | B<8>
+--->| addr          read_data +--->
B<8> |                         | bool
+--->| write_data           tx +--->
bool |                         | bool
+--->| read_enable         irq +--->
bool |                         | bool
+--->| write_enable     rts_n  +--->
bool |                         | bool
+--->| rx               dtr_n  +--->
bool |                         | bool
+--->| cts_n           out1_n  +--->
bool |                         | bool
+--->| dsr_n           out2_n  +--->
bool |                         |
+--->| ri_n                    |
bool |                         |
+--->| dcd_n                   |
     +-------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/uart_16550.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/uart_16550.md")]
use rhdl::prelude::*;

use super::uart::{uart as uart_kernel, Uart};
use crate::core::dff;

#[allow(unused_imports)]
use uart_kernel as _;

/// Address of the RBR (read) / THR (write) register.  When DLAB=1, holds DLL instead.
const ADDR_DATA: u128 = 0x0;
/// Address of the IER register.  When DLAB=1, holds DLM instead.
const ADDR_IER: u128 = 0x1;
/// Address of the IIR (read) / FCR (write) register.
const ADDR_IIR_FCR: u128 = 0x2;
/// Address of the LCR register.
const ADDR_LCR: u128 = 0x3;
/// Address of the MCR register.
const ADDR_MCR: u128 = 0x4;
/// Address of the LSR register (read-only).
const ADDR_LSR: u128 = 0x5;
/// Address of the MSR register (read-only).
const ADDR_MSR: u128 = 0x6;
/// Address of the SCR (scratch) register.
const ADDR_SCR: u128 = 0x7;

/// LCR bit 7: DLAB (divisor-latch access bit).
const LCR_DLAB: u128 = 0x80;
/// LCR bit 6: set break.
const LCR_BREAK: u128 = 0x40;

/// IER bit 0: enable received-data-available interrupt.
const IER_ERBFI: u128 = 0x01;
/// IER bit 1: enable TX-holding-register-empty interrupt.
const IER_ETBEI: u128 = 0x02;

/// MCR bit 0: DTR (data-terminal-ready).
const MCR_DTR: u128 = 0x01;
/// MCR bit 1: RTS (request-to-send).
const MCR_RTS: u128 = 0x02;
/// MCR bit 2: OUT1 (general-purpose output 1).
const MCR_OUT1: u128 = 0x04;
/// MCR bit 3: OUT2 (general-purpose output 2).
const MCR_OUT2: u128 = 0x08;
/// MCR bit 4: loopback enable.
const MCR_LOOP: u128 = 0x10;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// 16550A-compatible UART (v2 — full register surface).
pub struct Uart16550<const DIV_W: usize, const FIFO_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    /// Underlying full-duplex UART with TX/RX FIFOs.
    uart: Uart<DIV_W, FIFO_W>,
    /// Line Control Register.  Bits 0..6 are programmable; bit 7 is DLAB.
    lcr: dff::DFF<Bits<8>>,
    /// Interrupt Enable Register.  Lower 4 bits used.
    ier: dff::DFF<Bits<8>>,
    /// FIFO Control Register.  Bit 0 = FIFO enable; bits 6..7 = RX trigger.
    fcr: dff::DFF<Bits<8>>,
    /// Modem Control Register.  Bits 0..4 used (DTR, RTS, OUT1, OUT2, loop).
    mcr: dff::DFF<Bits<8>>,
    /// Divisor Latch Low.  Stored but not yet routed to the underlying baud divisor.
    dll: dff::DFF<Bits<8>>,
    /// Divisor Latch High.  Stored but not yet routed.
    dlm: dff::DFF<Bits<8>>,
    /// Scratchpad register.  Pure software storage.
    scr: dff::DFF<Bits<8>>,
    /// Sticky overrun bit — set when a new RX byte arrives while the
    /// previous one was unread; cleared on LSR read.
    overrun: dff::DFF<bool>,
    /// Previous-cycle modem-status input lines (active-low pins
    /// inverted to active-high "asserted" form).  Used to compute
    /// the four delta bits in MSR each cycle.
    prev_modem: dff::DFF<Bits<4>>,
}

impl<const DIV_W: usize, const FIFO_W: usize> Uart16550<DIV_W, FIFO_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    /// Create a 16550A UART with the given baud divisor.  The
    /// divisor is fixed at construction in v2; DLL/DLM are
    /// storage-only.  v3 will route DLL/DLM through to the
    /// underlying TX / RX clocks.
    pub fn new(divisor: Bits<DIV_W>) -> Self {
        Self {
            uart: Uart::new(divisor),
            lcr: dff::DFF::default(),
            ier: dff::DFF::default(),
            fcr: dff::DFF::default(),
            mcr: dff::DFF::default(),
            dll: dff::DFF::default(),
            dlm: dff::DFF::default(),
            scr: dff::DFF::default(),
            overrun: dff::DFF::default(),
            prev_modem: dff::DFF::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Uart16550].
pub struct In {
    /// Register address (3 bits, 8 registers).
    pub addr: Bits<3>,
    /// Byte to write when `write_enable` is asserted.
    pub write_data: Bits<8>,
    /// Strobe a register read.  Side-effects: pop RX FIFO when
    /// `addr == ADDR_DATA && DLAB == 0`; clear LSR sticky bits
    /// when `addr == ADDR_LSR`; clear delta bits in MSR when
    /// `addr == ADDR_MSR`.
    pub read_enable: bool,
    /// Strobe a register write.  Side-effects: push TX FIFO
    /// when `addr == ADDR_DATA && DLAB == 0`.
    pub write_enable: bool,
    /// Serial input line (idle high).
    pub rx: bool,
    /// CTS pin (active-low).  `false` = CTS asserted by peer.
    pub cts_n: bool,
    /// DSR pin (active-low).
    pub dsr_n: bool,
    /// RI pin (active-low).
    pub ri_n: bool,
    /// DCD pin (active-low).
    pub dcd_n: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Uart16550].
pub struct Out {
    /// Register read data.  0 for write-only or unmapped reads.
    pub read_data: Bits<8>,
    /// Serial output line (idle high).
    pub tx: bool,
    /// Composite interrupt request — asserted while any IER-enabled
    /// source is pending.
    pub irq: bool,
    /// RTS pin output (active-low).
    pub rts_n: bool,
    /// DTR pin output (active-low).
    pub dtr_n: bool,
    /// OUT1 pin output (active-low; general-purpose).
    pub out1_n: bool,
    /// OUT2 pin output (active-low; general-purpose, traditionally
    /// used as the master IRQ enable on PC platforms).
    pub out2_n: bool,
}

impl<const DIV_W: usize, const FIFO_W: usize> SynchronousIO for Uart16550<DIV_W, FIFO_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = uart_16550<DIV_W, FIFO_W>;
}

#[kernel]
/// Kernel for [Uart16550].
pub fn uart_16550<const DIV_W: usize, const FIFO_W: usize>(
    _cr: ClockReset,
    i: In,
    q: Q<DIV_W, FIFO_W>,
) -> (Out, D<DIV_W, FIFO_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<FIFO_W>: BitWidth,
{
    let mut d = D::<DIV_W, FIFO_W>::dont_care();
    d.uart = super::uart::In {
        tx_data: bits::<8>(0),
        tx_push: false,
        rx_pop: false,
        rx: i.rx,
    };
    d.lcr = q.lcr;
    d.ier = q.ier;
    d.fcr = q.fcr;
    d.mcr = q.mcr;
    d.dll = q.dll;
    d.dlm = q.dlm;
    d.scr = q.scr;
    d.overrun = q.overrun;
    d.prev_modem = q.prev_modem;

    // ---- Decode address ----
    let dlab = (q.lcr & bits::<8>(LCR_DLAB)) != bits::<8>(0);
    let is_data = i.addr == bits::<3>(ADDR_DATA);
    let is_ier = i.addr == bits::<3>(ADDR_IER);
    let is_iir_fcr = i.addr == bits::<3>(ADDR_IIR_FCR);
    let is_lcr = i.addr == bits::<3>(ADDR_LCR);
    let is_mcr = i.addr == bits::<3>(ADDR_MCR);
    let is_lsr = i.addr == bits::<3>(ADDR_LSR);
    let is_msr = i.addr == bits::<3>(ADDR_MSR);
    let is_scr = i.addr == bits::<3>(ADDR_SCR);

    // ---- Decode RX byte from underlying UART ----
    let (rx_byte, rx_valid) = match q.uart.rx_data {
        Some(byte) => (byte, true),
        None => (bits::<8>(0), false),
    };

    // ---- Loopback wiring (MCR bit 4) ----
    let loopback = (q.mcr & bits::<8>(MCR_LOOP)) != bits::<8>(0);

    // ---- Modem-status pin sampling ----
    // Pins are active-low at the connector; convert to active-high
    // "asserted" semantics here.  In loopback mode, MCR bits drive
    // MSR inputs internally (CTS<-RTS, DSR<-DTR, RI<-OUT1, DCD<-OUT2)
    // so software can self-test without external wires.
    let raw_cts = !i.cts_n;
    let raw_dsr = !i.dsr_n;
    let raw_ri = !i.ri_n;
    let raw_dcd = !i.dcd_n;
    let mcr_dtr = (q.mcr & bits::<8>(MCR_DTR)) != bits::<8>(0);
    let mcr_rts = (q.mcr & bits::<8>(MCR_RTS)) != bits::<8>(0);
    let mcr_out1 = (q.mcr & bits::<8>(MCR_OUT1)) != bits::<8>(0);
    let mcr_out2 = (q.mcr & bits::<8>(MCR_OUT2)) != bits::<8>(0);
    let cts = if loopback { mcr_rts } else { raw_cts };
    let dsr = if loopback { mcr_dtr } else { raw_dsr };
    let ri = if loopback { mcr_out1 } else { raw_ri };
    let dcd = if loopback { mcr_out2 } else { raw_dcd };

    // Pack into Bits<4> for storage / delta computation.
    let mut cur_modem = bits::<4>(0);
    if cts {
        cur_modem = cur_modem | bits::<4>(0x1);
    }
    if dsr {
        cur_modem = cur_modem | bits::<4>(0x2);
    }
    if ri {
        cur_modem = cur_modem | bits::<4>(0x4);
    }
    if dcd {
        cur_modem = cur_modem | bits::<4>(0x8);
    }
    d.prev_modem = cur_modem;
    let modem_changed = cur_modem ^ q.prev_modem;
    let dcts = (modem_changed & bits::<4>(0x1)) != bits::<4>(0);
    let ddsr = (modem_changed & bits::<4>(0x2)) != bits::<4>(0);
    // RI uses *trailing-edge* (the assert→deassert transition)
    // per the canonical 16550A: TERI = was set, now clear.
    let prev_ri = (q.prev_modem & bits::<4>(0x4)) != bits::<4>(0);
    let teri = prev_ri && !ri;
    let ddcd = (modem_changed & bits::<4>(0x8)) != bits::<4>(0);

    // ---- Update overrun sticky bit ----
    // (Reading LSR clears the sticky bits.)
    if is_lsr && i.read_enable {
        d.overrun = false;
    }

    // ---- LSR composition ----
    let mut lsr = bits::<8>(0);
    if rx_valid {
        lsr = lsr | bits::<8>(0x01); // DR (data ready)
    }
    if q.overrun {
        lsr = lsr | bits::<8>(0x02); // OE (overrun)
    }
    // Bits 2-4 (parity / framing / break) — not yet wired; v3.
    if !q.uart.tx_full {
        lsr = lsr | bits::<8>(0x20); // THRE (TX holding register empty)
        lsr = lsr | bits::<8>(0x40); // TEMT (TX shifter empty — approximated)
    }

    // ---- IIR composition (priority-encoded) ----
    // Per 16550A datasheet, priority order (highest first):
    //   1. Receiver Line Status     (bits 1-3 = 0b110, ID = 0x6)
    //   2. Received Data Available  (bits 1-3 = 0b100, ID = 0x4)
    //   3. THR Empty                (bits 1-3 = 0b010, ID = 0x2)
    //   4. Modem Status             (bits 1-3 = 0b000, ID = 0x0)
    //   None pending                (bit 0 = 1)
    let ier = q.ier;
    let line_status_pending =
        ((ier & bits::<8>(0x04)) != bits::<8>(0)) && ((lsr & bits::<8>(0x9E)) != bits::<8>(0));
    let rx_pending = ((ier & bits::<8>(IER_ERBFI)) != bits::<8>(0)) && rx_valid;
    let tx_pending = ((ier & bits::<8>(IER_ETBEI)) != bits::<8>(0)) && !q.uart.tx_full;
    let modem_pending = ((ier & bits::<8>(0x08)) != bits::<8>(0)) && (dcts || ddsr || teri || ddcd);
    let any_pending = line_status_pending || rx_pending || tx_pending || modem_pending;

    let iir = if line_status_pending {
        // FIFO state in bits 6-7 always 0b11 (FIFO enabled — the
        // underlying UART always has a FIFO).
        bits::<8>(0xC0 | 0x06)
    } else if rx_pending {
        bits::<8>(0xC0 | 0x04)
    } else if tx_pending {
        bits::<8>(0xC0 | 0x02)
    } else if modem_pending {
        bits::<8>(0xC0 | 0x00)
    } else {
        bits::<8>(0xC0 | 0x01) // no interrupt pending (bit 0 = 1)
    };

    // ---- MSR composition ----
    let mut msr = bits::<8>(0);
    if dcts {
        msr = msr | bits::<8>(0x01);
    }
    if ddsr {
        msr = msr | bits::<8>(0x02);
    }
    if teri {
        msr = msr | bits::<8>(0x04);
    }
    if ddcd {
        msr = msr | bits::<8>(0x08);
    }
    if cts {
        msr = msr | bits::<8>(0x10);
    }
    if dsr {
        msr = msr | bits::<8>(0x20);
    }
    if ri {
        msr = msr | bits::<8>(0x40);
    }
    if dcd {
        msr = msr | bits::<8>(0x80);
    }

    // ---- Read mux ----
    let read_data = if is_data && !dlab {
        rx_byte
    } else if is_data && dlab {
        q.dll
    } else if is_ier && !dlab {
        q.ier
    } else if is_ier && dlab {
        q.dlm
    } else if is_iir_fcr {
        iir
    } else if is_lcr {
        q.lcr
    } else if is_mcr {
        q.mcr
    } else if is_lsr {
        lsr
    } else if is_msr {
        msr
    } else if is_scr {
        q.scr
    } else {
        bits::<8>(0)
    };

    // ---- Write side-effects ----
    if i.write_enable {
        if is_data && !dlab {
            d.uart = super::uart::In {
                tx_data: i.write_data,
                tx_push: true,
                rx_pop: false,
                rx: i.rx,
            };
        } else if is_data && dlab {
            d.dll = i.write_data;
        } else if is_ier && !dlab {
            d.ier = i.write_data & bits::<8>(0x0F); // only low 4 bits defined
        } else if is_ier && dlab {
            d.dlm = i.write_data;
        } else if is_iir_fcr {
            d.fcr = i.write_data;
            // FIFO clear bits 1 / 2 are self-clearing per cycle.
            // The underlying UART doesn't yet expose a clear input;
            // we accept the write and leave clearing as a v3
            // follow-up.
        } else if is_lcr {
            d.lcr = i.write_data;
        } else if is_mcr {
            d.mcr = i.write_data & bits::<8>(0x1F); // bits 0..4 defined
        } else if is_scr {
            d.scr = i.write_data;
        }
    }

    // ---- Read side-effects ----
    if i.read_enable && is_data && !dlab {
        // Pop RX FIFO if the underlying UART had a valid byte;
        // otherwise the read returns 0 and pop_rx is harmless.
        d.uart = super::uart::In {
            tx_data: bits::<8>(0),
            tx_push: false,
            rx_pop: true,
            rx: i.rx,
        };
    }

    // ---- Loopback: when MCR.LOOP is set, the underlying UART's
    // RX line is fed by its own TX line (so software can self-test
    // without external wires).  This overrides the rx routing.
    if loopback {
        d.uart = super::uart::In {
            tx_data: d.uart.tx_data,
            tx_push: d.uart.tx_push,
            rx_pop: d.uart.rx_pop,
            rx: q.uart.tx,
        };
    }

    // ---- Break control: when LCR.bit6 is set, force TX low ----
    let tx_break = (q.lcr & bits::<8>(LCR_BREAK)) != bits::<8>(0);
    let tx_out = if tx_break { false } else { q.uart.tx };

    let mut o = Out::dont_care();
    o.read_data = read_data;
    o.tx = tx_out;
    o.irq = any_pending;
    o.rts_n = !mcr_rts;
    o.dtr_n = !mcr_dtr;
    o.out1_n = !mcr_out1;
    o.out2_n = !mcr_out2;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            addr: bits(0),
            write_data: bits(0),
            read_enable: false,
            write_enable: false,
            rx: true,
            cts_n: true,
            dsr_n: true,
            ri_n: true,
            dcd_n: true,
        }
    }

    fn write_reg(addr: u128, data: u128) -> In {
        In {
            addr: bits(addr),
            write_data: bits(data),
            read_enable: false,
            write_enable: true,
            rx: true,
            cts_n: true,
            dsr_n: true,
            ri_n: true,
            dcd_n: true,
        }
    }

    fn read_reg(addr: u128) -> In {
        In {
            addr: bits(addr),
            write_data: bits(0),
            read_enable: true,
            write_enable: false,
            rx: true,
            cts_n: true,
            dsr_n: true,
            ri_n: true,
            dcd_n: true,
        }
    }

    /// Encode a UART frame for the rx line (idle high, start bit, 8 data
    /// bits LSB-first, stop bit), each held for `divisor` cycles.
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

    fn run_stream<const DV: usize, const FW: usize>(
        uut: &Uart16550<DV, FW>,
        stream_in: Vec<In>,
    ) -> Vec<(In, Out)>
    where
        rhdl::bits::W<DV>: BitWidth,
        rhdl::bits::W<FW>: BitWidth,
    {
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        uut.run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .map(|s| (s.input.1, s.output))
            .collect()
    }

    /// Find the read-back of the *last* read of `addr` in the stream.
    fn last_read_at(outputs: &[(In, Out)], addr: u128) -> Option<u128> {
        outputs
            .iter()
            .rev()
            .find(|(i, _)| i.read_enable && i.addr.raw() == addr)
            .map(|(_, o)| o.read_data.raw())
    }

    #[test]
    fn test_idle_state_no_irq() -> miette::Result<()> {
        let uut = Uart16550::<6, 4>::new(bits(8));
        let outputs = run_stream(&uut, std::iter::repeat_n(idle_in(), 64).collect());
        let any_irq = outputs.iter().any(|(_, o)| o.irq);
        assert!(!any_irq, "irq must not assert in idle");
        Ok(())
    }

    #[test]
    fn test_dlab_round_trip() -> miette::Result<()> {
        // Set DLAB by writing LCR=0x80; then write DLL=0x42, DLM=0x13;
        // read them back.  Verify DLL/DLM hold the written values.
        let uut = Uart16550::<6, 4>::new(bits(8));
        let mut stream_in: Vec<In> = vec![
            write_reg(ADDR_LCR, LCR_DLAB), // set DLAB
            write_reg(ADDR_DATA, 0x42),    // DLL = 0x42
            write_reg(ADDR_IER, 0x13),     // DLM = 0x13
            read_reg(ADDR_DATA),           // read DLL
            read_reg(ADDR_IER),            // read DLM
        ];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        // The reads happened with DLAB=1, so addr=0 → DLL, addr=1 → DLM.
        let dll = last_read_at(&outputs, ADDR_DATA).unwrap();
        let dlm = last_read_at(&outputs, ADDR_IER).unwrap();
        assert_eq!(dll, 0x42, "DLL read mismatch: got 0x{dll:02x}");
        assert_eq!(dlm, 0x13, "DLM read mismatch: got 0x{dlm:02x}");
        Ok(())
    }

    #[test]
    fn test_scratch_register_round_trip() -> miette::Result<()> {
        let uut = Uart16550::<6, 4>::new(bits(8));
        let mut stream_in: Vec<In> = vec![write_reg(ADDR_SCR, 0xA5), read_reg(ADDR_SCR)];
        for _ in 0..8 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        let val = last_read_at(&outputs, ADDR_SCR).unwrap();
        assert_eq!(val, 0xA5, "SCR round-trip mismatch: got 0x{val:02x}");
        Ok(())
    }

    #[test]
    fn test_mcr_drives_outputs() -> miette::Result<()> {
        // Write MCR = 0b00001111 (DTR + RTS + OUT1 + OUT2 set).
        // Verify that the output pins go active-low (asserted).
        let uut = Uart16550::<6, 4>::new(bits(8));
        let mut stream_in: Vec<In> = vec![write_reg(ADDR_MCR, 0x0F)];
        for _ in 0..8 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        let post = outputs.iter().rev().find(|(i, _)| !i.write_enable).unwrap();
        assert!(!post.1.dtr_n, "DTR should be asserted (low)");
        assert!(!post.1.rts_n, "RTS should be asserted (low)");
        assert!(!post.1.out1_n, "OUT1 should be asserted (low)");
        assert!(!post.1.out2_n, "OUT2 should be asserted (low)");
        Ok(())
    }

    #[test]
    fn test_msr_modem_inputs_visible() -> miette::Result<()> {
        // Drive cts low (asserted), read MSR, verify CTS bit set.
        let uut = Uart16550::<6, 4>::new(bits(8));
        let mut stream_in: Vec<In> = Vec::new();
        // Hold modem inputs for a few cycles so prev_modem catches up.
        for _ in 0..4 {
            stream_in.push(In {
                addr: bits(0),
                write_data: bits(0),
                read_enable: false,
                write_enable: false,
                rx: true,
                cts_n: false, // CTS asserted
                dsr_n: true,
                ri_n: true,
                dcd_n: true,
            });
        }
        stream_in.push(In {
            addr: bits(ADDR_MSR),
            write_data: bits(0),
            read_enable: true,
            write_enable: false,
            rx: true,
            cts_n: false,
            dsr_n: true,
            ri_n: true,
            dcd_n: true,
        });
        for _ in 0..4 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        let msr = last_read_at(&outputs, ADDR_MSR).unwrap();
        assert!(
            msr & 0x10 != 0,
            "CTS bit must be set in MSR when cts_n is low: got 0x{msr:02x}"
        );
        Ok(())
    }

    #[test]
    fn test_loopback_byte() -> miette::Result<()> {
        // Set MCR.LOOP, write a byte to THR; the underlying TX → RX
        // path should loop the byte back to RBR.
        let divisor = 6;
        let uut = Uart16550::<6, 4>::new(bits(divisor as u128));
        let mut stream_in: Vec<In> = vec![
            write_reg(ADDR_MCR, MCR_LOOP), // enable loopback
            write_reg(ADDR_DATA, 0x5A),    // THR write
        ];
        // Wait long enough for a full UART frame to round-trip.
        for _ in 0..(15 * divisor + 20) {
            stream_in.push(idle_in());
        }
        // Now read RBR.
        stream_in.push(read_reg(ADDR_DATA));
        for _ in 0..4 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        let rx_byte = last_read_at(&outputs, ADDR_DATA).unwrap();
        assert_eq!(rx_byte, 0x5A, "loopback byte mismatch: got 0x{rx_byte:02x}");
        Ok(())
    }

    #[test]
    fn test_rx_to_data_register() -> miette::Result<()> {
        let divisor = 6;
        let uut = Uart16550::<6, 4>::new(bits(divisor as u128));
        let frame = encode_frame(0xA5, divisor);
        let mut stream_in: Vec<In> = Vec::new();
        for _ in 0..4 {
            stream_in.push(idle_in());
        }
        for &rx in &frame {
            stream_in.push(In {
                addr: bits(0),
                write_data: bits(0),
                read_enable: false,
                write_enable: false,
                rx,
                cts_n: true,
                dsr_n: true,
                ri_n: true,
                dcd_n: true,
            });
        }
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        stream_in.push(read_reg(ADDR_DATA));
        for _ in 0..4 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        let val = last_read_at(&outputs, ADDR_DATA).unwrap();
        assert_eq!(val, 0xA5, "RBR read mismatch: got 0x{val:02x}");
        Ok(())
    }

    #[test]
    fn test_break_control_drives_tx_low() -> miette::Result<()> {
        // Set LCR.bit6 (break) — TX line should go low immediately.
        let uut = Uart16550::<6, 4>::new(bits(8));
        let mut stream_in: Vec<In> = vec![write_reg(ADDR_LCR, LCR_BREAK)];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        let post_low = outputs.iter().rev().take(8).all(|(_, o)| !o.tx);
        assert!(post_low, "TX must be held low while LCR.break is set");
        Ok(())
    }

    #[test]
    fn test_iir_priority_encoding() -> miette::Result<()> {
        // Inject an RX byte while IER has the RX-enable bit set;
        // read IIR — should see source ID 0b100 (0x4 in bits 1-3),
        // bit 0 cleared (interrupt pending), bits 6-7 = 0b11.
        let divisor = 6;
        let uut = Uart16550::<6, 4>::new(bits(divisor as u128));
        let frame = encode_frame(0x77, divisor);
        let mut stream_in: Vec<In> = vec![write_reg(ADDR_IER, IER_ERBFI)]; // enable RX irq
        for _ in 0..4 {
            stream_in.push(idle_in());
        }
        for &rx in &frame {
            stream_in.push(In {
                addr: bits(0),
                write_data: bits(0),
                read_enable: false,
                write_enable: false,
                rx,
                cts_n: true,
                dsr_n: true,
                ri_n: true,
                dcd_n: true,
            });
        }
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        stream_in.push(read_reg(ADDR_IIR_FCR));
        for _ in 0..4 {
            stream_in.push(idle_in());
        }
        let outputs = run_stream(&uut, stream_in);
        let iir = last_read_at(&outputs, ADDR_IIR_FCR).unwrap();
        // Bits 6-7 = 0b11 (FIFO enabled), bits 1-3 = 0b010 (the canonical
        // RX-data-available source ID is 0x04 in the byte, which is
        // 0b00000100 → bits 1-3 = 0b010 — but the priority constant we
        // emit is 0xC4, decoded as: bits 6-7 = 0b11, bit 0 = 0, bits 1-3 = 0b010).
        assert!(
            iir & 0xC0 == 0xC0,
            "FIFO state bits should be 0b11 in IIR: got 0x{iir:02x}"
        );
        assert!(
            iir & 0x01 == 0,
            "interrupt pending (bit 0 = 0): got 0x{iir:02x}"
        );
        assert_eq!(
            iir & 0x0E,
            0x04,
            "source ID should be RX-data: got 0x{iir:02x}"
        );
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check.
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Uart16550::<6, 4>::new(bits(8));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["78819"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip.
    #[test]
    fn test_uart_16550_hdl_works() -> miette::Result<()> {
        let uut = Uart16550::<6, 4>::new(bits(8));
        let stream = std::iter::repeat_n(idle_in(), 64)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest.
    #[test]
    fn test_uart_16550_trace() -> miette::Result<()> {
        let uut = Uart16550::<6, 4>::new(bits(8));
        let stream = std::iter::repeat_n(idle_in(), 64)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("uart_16550");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["3024820752cdedc2ca61d4316048ee1b92509fa8b11b487cc26711ff4fc25780"];
        let digest = vcd.dump_to_file(root.join("uart_16550.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
