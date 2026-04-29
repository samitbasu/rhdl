//! Half-duplex / 3-wire SPI master
//!
//! A SPI master that performs a single CS-asserted transaction
//! consisting of `write_bits` written to the slave followed by
//! `read_bits` read back, with a configurable `turnaround` gap
//! between the two halves.  The shared serial-data wire is
//! tristated by the host between phases — the master drives during
//! `Write`, releases during `Turnaround` and `Read`.
//!
//! This is the form used by Bosch BMP280 / BMI270, ST LIS3DH,
//! TI ADS1015, National Microwire devices, and many other small
//! sensors and converters.  The fundamental shape — write a small
//! command/address word, then turn the line over and read a value
//! back — is the canonical 3-wire pattern, with the configurable
//! turnaround letting the master pause for slave-internal settling.
//!
//! The master exposes the SDIO line as a `(sdio_oe, sdio_out)` pair:
//! the host wraps with [super::super::tristate::simple] (or with
//! the FPGA's pad I/O `IOBUF` primitive) to expose a true
//! bidirectional pin to the outside world.  Mode 0 (CPOL=0,
//! CPHA=0), MSB-first.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+HalfSpiMaster+------+
     |                          |
B<W> |                          | bool
+--->| tx_data           sclk   +--->
B<CW>|                          | bool
+--->| write_bits     sdio_out  +--->
B<CW>|                          | bool
+--->| read_bits      sdio_oe   +--->
B<CW>|                          | bool
+--->| turnaround       cs_n    +--->
bool |                          | B<W>
+--->| sdio_in         rx_data  +--->
bool |                          | bool
+--->| start            busy    +--->
     |                  done    +--->
     +--------------------------+
")]
//!
//!# Internals
//!
//! State machine with four states, all with `cs_n = 0`:
//!
//! - **`Idle`** — `cs_n = 1`, `sclk = 0`, `sdio_oe = 0`.  Asserting
//!   `start` latches the operands and transitions to `Write`.
//! - **`Write`** — Master drives `sdio_out` from the MSB of the
//!   TX shift register; `sdio_oe = 1`.  `sclk` toggles each
//!   FPGA cycle (2 cycles per SPI bit).  After `write_bits` bits,
//!   transition to `Turnaround` (or directly to `Read` if
//!   `turnaround = 0`).
//! - **`Turnaround`** — `sclk` held low; `sdio_oe = 0` (line
//!   released to the slave).  Counts `turnaround_cycles` FPGA
//!   clocks, then transitions to `Read`.
//! - **`Read`** — `sclk` toggles; `sdio_oe = 0`; the master
//!   samples `sdio_in` on each rising sclk edge into the RX shift
//!   register.  After `read_bits` bits, returns to `Idle` and
//!   pulses `done`.
//!
//!# Behavior
//!
//! - `write_bits`, `read_bits`, and `turnaround` are runtime
//!   inputs (latched at `start`).  They must each fit in `Bits<CW>`.
//! - 3-wire mode: external IOBUF with `oe = sdio_oe`,
//!   `out = sdio_out`, and `in → sdio_in`.
//! - 4-wire mode: connect `sdio_out` to MOSI and feed the slave's
//!   MISO into `sdio_in` (ignore `sdio_oe` — MOSI is always
//!   driven).  This widget naturally supports both layouts via the
//!   same I/O.
//!
//!# Parameters
//!
//! - `W` — maximum word width (TX/RX shift registers).  Pick `W >=
//!   max(write_bits, read_bits)` across all transactions.
//! - `CW` — bit width of the bit-counter and turnaround counter.
//!   Satisfy `2^CW >= max_bits + 1`.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/half_spi_master.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/half_spi_master.md")]
//!
//! And the auto-generated FSM diagram for the half-duplex transaction:
#![doc = include_str!("../../doc/half_spi_master_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::dff;

/// Author-curated transition graph for the half-duplex SPI master.
///
/// Required by CLAUDE.md §12 rule 14.  Indices match
/// `HalfSpiState` declaration order (Idle=0, Write=1, Turnaround=2,
/// Read=3).
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → Write (on `start`)
    Transition {
        source_index: 1,
        target_index: 1,
    }, // Write self-loop (per bit)
    Transition {
        source_index: 1,
        target_index: 2,
    }, // Write → Turnaround (after write_bits)
    Transition {
        source_index: 2,
        target_index: 2,
    }, // Turnaround self-loop
    Transition {
        source_index: 2,
        target_index: 3,
    }, // Turnaround → Read (after turnaround)
    Transition {
        source_index: 3,
        target_index: 3,
    }, // Read self-loop (per bit)
    Transition {
        source_index: 3,
        target_index: 0,
    }, // Read → Idle (transaction done)
];

/// State of the half-duplex SPI master.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum HalfSpiState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    #[fsm_state(label = "write")]
    Write,
    #[fsm_state(label = "turnaround")]
    Turnaround,
    #[fsm_state(label = "read")]
    Read,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = HalfSpiState)]
/// Half-duplex / 3-wire SPI master core.
pub struct HalfSpiMaster<const W: usize, const CW: usize>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    state: dff::DFF<HalfSpiState>,
    bit_counter: dff::DFF<Bits<CW>>,
    turn_counter: dff::DFF<Bits<CW>>,
    phase: dff::DFF<bool>,
    shift_tx: dff::DFF<Bits<W>>,
    shift_rx: dff::DFF<Bits<W>>,
    write_bits_reg: dff::DFF<Bits<CW>>,
    read_bits_reg: dff::DFF<Bits<CW>>,
    turn_reg: dff::DFF<Bits<CW>>,
    done_pulse: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [HalfSpiMaster].
pub struct In<const W: usize, const CW: usize>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    /// Data to write (latched at `start`).  MSB sent first.
    pub tx_data: Bits<W>,
    /// Number of bits to write (latched at `start`).  Must satisfy `1 <= write_bits <= W`.
    pub write_bits: Bits<CW>,
    /// Number of bits to read (latched at `start`).  Must satisfy `1 <= read_bits <= W`.
    pub read_bits: Bits<CW>,
    /// FPGA cycles to hold between write and read phases (latched at `start`).
    /// Use `0` to skip the turnaround.
    pub turnaround: Bits<CW>,
    /// Sampled SDIO line (used during `Read` phase).
    pub sdio_in: bool,
    /// Strobe to begin a transaction.  Ignored while `busy`.
    pub start: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [HalfSpiMaster].
pub struct Out<const W: usize>
where
    rhdl::bits::W<W>: BitWidth,
{
    /// Serial clock (idle low, CPOL=0).  Held low during `Idle` and `Turnaround`.
    pub sclk: bool,
    /// Drive value for the SDIO line (only meaningful when `sdio_oe == true`).
    pub sdio_out: bool,
    /// Output enable for the SDIO line.  `true` ⇒ master drives;
    /// `false` ⇒ master releases (the slave or pull-up takes over).
    pub sdio_oe: bool,
    /// Chip select, active low.  Asserted (low) during all non-Idle phases.
    pub cs_n: bool,
    /// Last fully-received word.  MSB-first (the first bit sampled
    /// ends up at bit position `read_bits - 1` after all shifts).
    pub rx_data: Bits<W>,
    /// High during a transaction.
    pub busy: bool,
    /// Pulses for one cycle when the transaction completes.
    pub done: bool,
}

impl<const W: usize, const CW: usize> SynchronousIO for HalfSpiMaster<W, CW>
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    type I = In<W, CW>;
    type O = Out<W>;
    type Kernel = half_spi_master<W, CW>;
}

#[kernel]
/// Kernel for [HalfSpiMaster].
pub fn half_spi_master<const W: usize, const CW: usize>(
    cr: ClockReset,
    i: In<W, CW>,
    q: Q<W, CW>,
) -> (Out<W>, D<W, CW>)
where
    rhdl::bits::W<W>: BitWidth,
    rhdl::bits::W<CW>: BitWidth,
{
    let one_cw: Bits<CW> = bits::<CW>(1);
    let zero_cw: Bits<CW> = bits::<CW>(0);
    let zero_w: Bits<W> = bits::<W>(0);
    let one_w: Bits<W> = bits::<W>(1);

    let mut d = D::<W, CW>::dont_care();
    d.state = q.state;
    d.bit_counter = q.bit_counter;
    d.turn_counter = q.turn_counter;
    d.phase = q.phase;
    d.shift_tx = q.shift_tx;
    d.shift_rx = q.shift_rx;
    d.write_bits_reg = q.write_bits_reg;
    d.read_bits_reg = q.read_bits_reg;
    d.turn_reg = q.turn_reg;
    d.done_pulse = false;

    match q.state {
        HalfSpiState::Idle => {
            if i.start {
                d.state = HalfSpiState::Write;
                d.shift_tx = i.tx_data;
                d.shift_rx = zero_w;
                d.bit_counter = zero_cw;
                d.turn_counter = zero_cw;
                d.phase = false;
                d.write_bits_reg = i.write_bits;
                d.read_bits_reg = i.read_bits;
                d.turn_reg = i.turnaround;
            }
        }
        HalfSpiState::Write => {
            d.phase = !q.phase;
            // On falling sclk edge (q.phase==1, becoming 0): advance bit.
            if q.phase {
                d.shift_tx = q.shift_tx << 1;
                let next_count = q.bit_counter + one_cw;
                d.bit_counter = next_count;
                if next_count == q.write_bits_reg {
                    // Last write bit done.  Choose next state based on turnaround.
                    if q.turn_reg == zero_cw {
                        d.state = HalfSpiState::Read;
                        d.bit_counter = zero_cw;
                        d.phase = false;
                    } else {
                        d.state = HalfSpiState::Turnaround;
                        d.turn_counter = zero_cw;
                        d.phase = false;
                    }
                }
            }
        }
        HalfSpiState::Turnaround => {
            let next_turn = q.turn_counter + one_cw;
            d.turn_counter = next_turn;
            if next_turn == q.turn_reg {
                d.state = HalfSpiState::Read;
                d.turn_counter = zero_cw;
                d.bit_counter = zero_cw;
                d.phase = false;
            }
        }
        HalfSpiState::Read => {
            d.phase = !q.phase;
            // On rising sclk edge (q.phase==0, becoming 1): sample SDIO.
            if !q.phase {
                let bit_in: Bits<W> = if i.sdio_in { one_w } else { zero_w };
                d.shift_rx = (q.shift_rx << 1) | bit_in;
            } else {
                // Falling edge: advance bit counter.
                let next_count = q.bit_counter + one_cw;
                d.bit_counter = next_count;
                if next_count == q.read_bits_reg {
                    d.state = HalfSpiState::Idle;
                    d.bit_counter = zero_cw;
                    d.phase = false;
                    d.done_pulse = true;
                }
            }
        }
    }

    if cr.reset.any() {
        d.state = HalfSpiState::Idle;
        d.bit_counter = zero_cw;
        d.turn_counter = zero_cw;
        d.phase = false;
        d.shift_tx = zero_w;
        d.shift_rx = zero_w;
        d.write_bits_reg = zero_cw;
        d.read_bits_reg = zero_cw;
        d.turn_reg = zero_cw;
        d.done_pulse = false;
    }

    // Outputs derived from current state.  Collapse the per-state
    // arms into or-patterns: the chip-select / busy logic groups by
    // "is this an active transaction state", and the sclk / sdio_oe
    // logic groups by "is this state actually clocking bits".
    let cs_n = match q.state {
        HalfSpiState::Idle => true,
        HalfSpiState::Write | HalfSpiState::Turnaround | HalfSpiState::Read => false,
    };
    let sclk = match q.state {
        HalfSpiState::Idle | HalfSpiState::Turnaround => false,
        HalfSpiState::Write | HalfSpiState::Read => q.phase,
    };
    let sdio_oe = match q.state {
        HalfSpiState::Write => true,
        HalfSpiState::Idle | HalfSpiState::Turnaround | HalfSpiState::Read => false,
    };
    // sdio_out: MSB of shift_tx during Write, otherwise don't-care (set 0).
    let mosi_bit = (q.shift_tx >> ((W - 1) as u128)) & one_w;
    let sdio_out = mosi_bit != zero_w;

    let busy = match q.state {
        HalfSpiState::Idle => false,
        HalfSpiState::Write | HalfSpiState::Turnaround | HalfSpiState::Read => true,
    };

    let mut o = Out::<W>::dont_care();
    o.sclk = sclk;
    o.sdio_out = sdio_out;
    o.sdio_oe = sdio_oe;
    o.cs_n = cs_n;
    o.rx_data = q.shift_rx;
    o.busy = busy;
    o.done = q.done_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In<8, 4> {
        In {
            tx_data: bits(0),
            write_bits: bits(0),
            read_bits: bits(0),
            turnaround: bits(0),
            sdio_in: false,
            start: false,
        }
    }

    /// Build a stream that starts a transaction and feeds back a
    /// simulated slave response on `sdio_in` during the read phase.
    /// Returns the recovered rx_data.
    fn run_transaction(
        tx: u128,
        write_bits: u128,
        read_bits: u128,
        turnaround: u128,
        slave_response: u128,
    ) -> u128 {
        // Cycle layout for a single transaction:
        //   cycle 0: idle, start strobe
        //   cycles 1..(2*write_bits + turnaround + 2*read_bits + 2): transaction
        // We'll build the input sequence and pre-compute MISO bits aligned to read-phase rising edges.
        //
        // The Read phase reads MSB-first. The k-th sampled bit (k=0..read_bits-1)
        // is the (read_bits-1-k)-th bit of slave_response (MSB first).
        let n_cycles =
            1 + 2 * (write_bits as usize) + (turnaround as usize) + 2 * (read_bits as usize) + 4;
        let mut stream_in: Vec<In<8, 4>> = Vec::with_capacity(n_cycles);
        for cycle in 0..n_cycles {
            let mut inp = idle_in();
            // Start strobe on cycle 0.
            if cycle == 0 {
                inp.tx_data = bits(tx);
                inp.write_bits = bits(write_bits);
                inp.read_bits = bits(read_bits);
                inp.turnaround = bits(turnaround);
                inp.start = true;
            }
            // Compute SDIO_in for the read phase.
            // Read starts at cycle = 1 (start) + 1 (state transition delay)
            //                       + 2*write_bits (write phase, 2 cycles per bit)
            //                       + turnaround.
            // Within read, each bit is 2 cycles; the master samples on the cycle
            // where q.phase=0 (rising edge from 0 to 1), so the slave should be
            // presenting the bit during the q.phase=0 cycle.
            //
            // Since the read starts with phase=false on its first cycle, the
            // master samples during cycles 0, 2, 4, ... of the read phase
            // (relative to the first read-phase cycle).
            //
            // Simpler: present each bit for 2 consecutive cycles aligned with
            // the read phase start.
            let read_start = 1 + 2 * (write_bits as usize) + (turnaround as usize);
            if cycle >= read_start {
                let read_offset = cycle - read_start;
                let bit_idx_in_phase = read_offset / 2;
                if bit_idx_in_phase < read_bits as usize {
                    let bit_pos = read_bits as usize - 1 - bit_idx_in_phase;
                    inp.sdio_in = ((slave_response >> bit_pos) & 1) != 0;
                }
            }
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let uut = HalfSpiMaster::<8, 4>::default();
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_idx = outputs.iter().position(|s| s.output.done).unwrap();
        outputs[done_idx].output.rx_data.raw()
    }

    // Tier 2 — write-then-read round trips with various widths.
    #[test]
    fn test_round_trip_8w_8r() -> miette::Result<()> {
        // Write 8 bits, read 8 bits, no turnaround.  Slave returns 0x55.
        let rx = run_transaction(0xA5, 8, 8, 0, 0x55);
        assert_eq!(rx, 0x55, "rx mismatch (no turnaround)");
        Ok(())
    }

    #[test]
    fn test_round_trip_with_turnaround() -> miette::Result<()> {
        // 8w 8r with 4-cycle turnaround.
        let rx = run_transaction(0xA5, 8, 8, 4, 0x42);
        assert_eq!(rx, 0x42);
        Ok(())
    }

    #[test]
    fn test_short_write_short_read() -> miette::Result<()> {
        // 4w 4r with no turnaround.  Slave returns 0xC (bottom nibble).
        let rx = run_transaction(0xC, 4, 4, 0, 0xC);
        // After 4 read shifts, rx_data has the 4 bits in the LSB nibble.
        // So `rx & 0xF` should be 0xC.
        assert_eq!(rx & 0xF, 0xC);
        Ok(())
    }

    #[test]
    fn test_idle_outputs_inactive() -> miette::Result<()> {
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = HalfSpiMaster::<8, 4>::default();
        let any_active = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.busy || !s.output.cs_n || s.output.sdio_oe || s.output.sclk);
        assert!(!any_active);
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = HalfSpiMaster::<8, 4>::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["17155"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_half_spi_master_hdl_works() -> miette::Result<()> {
        let uut = HalfSpiMaster::<8, 4>::default();
        let mut stream_in: Vec<In<8, 4>> = vec![In {
            tx_data: bits(0xA5),
            write_bits: bits(8),
            read_bits: bits(8),
            turnaround: bits(2),
            sdio_in: false,
            start: true,
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
    fn test_half_spi_master_trace() -> miette::Result<()> {
        let uut = HalfSpiMaster::<8, 4>::default();
        let mut stream_in: Vec<In<8, 4>> = vec![In {
            tx_data: bits(0xA5),
            write_bits: bits(8),
            read_bits: bits(8),
            turnaround: bits(2),
            sdio_in: false,
            start: true,
        }];
        for _ in 0..50 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("half_spi_master");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0ad99654f1eb89e1999fe912667163ec6d5dcadb63014a746e2d333657bc2f12"];
        let digest = vcd.dump_to_file(root.join("half_spi_master.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
