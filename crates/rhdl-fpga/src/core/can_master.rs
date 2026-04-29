//! CAN master — Classical CAN 2.0A transmit (v1)
//!
//! A bare-bones Classical CAN 2.0A frame transmitter.  Sends a
//! single standard data frame per `start` strobe, with proper
//! framing (SOF / ID / control / data / CRC-15 / delimiters / EOF /
//! IFS) and bit stuffing per the 5-same-bit rule.  Pairs with an
//! external CAN transceiver (TJA1050 / MCP2551 / SN65HVD230) — only
//! the digital TX line connects to the FPGA from this widget.
//!
//! **v1 scope:**
//! - 11-bit standard ID only (29-bit extended ID deferred).
//! - Data frame only (no remote / error / overload frames).
//! - Up to 8 data bytes (DLC 0..=8).
//! - Bit stuffing: yes (the canonical 5-same-bit rule, applied
//!   from SOF through the end of the CRC field).
//! - CRC-15: yes, polynomial `0x4599`, init `0`.
//! - **No ACK detection.**  The widget always drives the ACK slot
//!   recessive, expecting some other node to dominate it.  No
//!   ACK-error reporting.
//! - **No receiver, no acceptance filtering, no error counters,
//!   no bus-off state, no SJA1000 register interface.**  Those are
//!   tracked as v2 follow-ups.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+CanMaster+-----+
     |                     |
B<11>|                     | bool
+--->| id              tx  +--->
B<4> |                     | bool
+--->| dlc           busy  +--->
B<64>|                     | bool
+--->| data          done  +--->
bool |                     |
+--->| start               |
     +---------------------+
")]
//!
//!# Internals
//!
//! - **Bit-period counter** divides the FPGA clock down to the CAN
//!   bit rate.  At each rollover (one CAN bit time), the frame
//!   producer advances by one bit *unless* a stuff bit is being
//!   inserted.
//! - **Frame producer FSM** walks through the fields:
//!   SOF → ID (11) → RTR/IDE/r0 (3) → DLC (4) → Data (8 × DLC) →
//!   CRC (15) → CrcDelim → AckSlot → AckDelim → EOF (7) → IFS (3)
//!   → Idle.
//! - **Bit stuffer** monitors consecutive identical bits in the
//!   stuffable region (SOF through end of CRC).  After 5 same bits,
//!   the next bit is the inverse — a stuff bit — that does *not*
//!   advance the frame.
//! - **CRC-15 register** accumulates per-bit (excluding stuff
//!   bits) over SOF + ID + control + DLC + data, polynomial
//!   `0x4599`, init `0`.
//!
//!# Bit timing
//!
//! For v1, bit time = `bit_period` FPGA clocks.  Programmable
//! Sync / Prop / Phase1 / Phase2 segments per ISO 11898-1 are
//! deferred — for receive-side resync there's nothing to gain
//! since this v1 is TX only.
//!
//!# Parameters
//!
//! - `DIV_W` — bit width of the bit-period counter
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/can_master.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/can_master.md")]
use rhdl::prelude::*;

use super::{constant::Constant, dff};

/// Aggregate state for the CAN bit-stuffer.
///
/// Bit stuffing is the rule that any 5-in-a-row identical bits in the
/// stuffable region (SOF through CRC) must be followed by a "stuff
/// bit" of the inverse polarity that does NOT advance the frame.
/// Three pieces of state participate:
/// - `last_bit` — the previously committed wire bit (idle = recessive = `true`).
/// - `run` — count of consecutive same-polarity bits at the wire (1..5).
/// - `pending` — `true` when the next wire bit is the stuff bit (i.e.
///   `run` reached 5 on the prior bit).
///
/// Bundled into one struct purely so [CanMaster] stays under the
/// 12-field tuple ceiling that the `Synchronous` derive enforces.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct StuffState {
    /// Last raw (pre-stuff) bit emitted; tracked for stuff detection.
    pub last_bit: bool,
    /// Number of consecutive same bits at the wire (1..5).
    pub run: Bits<3>,
    /// True for one CAN bit period when the next wire bit is a stuff bit.
    pub pending: bool,
}

impl Default for StuffState {
    fn default() -> Self {
        Self {
            last_bit: true, // bus idle is recessive
            run: bits(0),
            pending: false,
        }
    }
}

/// Top-level state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default)]
pub enum CanState {
    #[default]
    Idle,
    Tx,
}

/// Sub-state within a transmit frame.  Each variant corresponds
/// to a CAN field.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum CanField {
    /// Start-of-frame: a single dominant bit that begins every frame.
    #[default]
    #[fsm_state(label = "SOF")]
    Sof,
    /// 11-bit standard identifier, MSB-first.
    Id,
    /// Remote transmission request bit (always dominant for data frames).
    Rtr,
    /// Identifier extension bit (dominant for standard ID, recessive for extended).
    Ide,
    /// Reserved bit r0 (always dominant).
    R0,
    /// 4-bit data length code.
    Dlc,
    /// 0..=64 data bits (8 × DLC), MSB-first.
    Data,
    /// 15-bit CRC, polynomial 0x4599, MSB-first.
    Crc,
    /// CRC delimiter — single recessive bit.
    #[fsm_state(label = "CRCDelim")]
    CrcDelim,
    /// ACK slot — driven recessive by master; receivers respond by going dominant.
    #[fsm_state(label = "ACK")]
    AckSlot,
    /// ACK delimiter — single recessive bit.
    #[fsm_state(label = "ACKDelim")]
    AckDelim,
    /// 7 recessive bits marking end-of-frame.
    Eof,
    /// 3 recessive bits of inter-frame spacing.
    Ifs,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "field", state_enum = CanField)]
/// CAN master core (TX-only v1).
pub struct CanMaster<const DIV_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    state: dff::DFF<CanState>,
    field: dff::DFF<CanField>,
    /// Bit index within the current field.  Wide enough for `Data` (max 64).
    field_bit_idx: dff::DFF<Bits<7>>,
    bit_phase_counter: dff::DFF<Bits<DIV_W>>,
    /// Latched copies of the inputs.
    id_reg: dff::DFF<Bits<11>>,
    dlc_reg: dff::DFF<Bits<4>>,
    data_reg: dff::DFF<Bits<64>>,
    /// CRC-15 register, polynomial 0x4599, init 0.
    crc_reg: dff::DFF<Bits<15>>,
    /// Bit-stuffer state; see [StuffState] for the field rationale.
    stuff: dff::DFF<StuffState>,
    done_pulse: dff::DFF<bool>,
    bit_period: Constant<Bits<DIV_W>>,
}

impl<const DIV_W: usize> CanMaster<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    /// Create a CAN master with the given FPGA-cycles-per-CAN-bit period.
    /// E.g. for 100 MHz clock and 1 Mbps CAN, `bit_period = 100`.
    pub fn new(bit_period: Bits<DIV_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            field: dff::DFF::default(),
            field_bit_idx: dff::DFF::default(),
            bit_phase_counter: dff::DFF::default(),
            id_reg: dff::DFF::default(),
            dlc_reg: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            crc_reg: dff::DFF::default(),
            stuff: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            bit_period: Constant::new(bit_period),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [CanMaster].
pub struct In {
    /// 11-bit standard CAN identifier.
    pub id: Bits<11>,
    /// Data length code (0..=8).  Specifies the number of data bytes.
    pub dlc: Bits<4>,
    /// Up to 8 bytes of data.  MSB-first transmission order: the
    /// most-significant byte (`bits 63..56`) goes out first.
    pub data: Bits<64>,
    /// Strobe to begin transmitting a frame.  Ignored while `busy`.
    pub start: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [CanMaster].
pub struct Out {
    /// CAN bus line.  `false` (low) = dominant, `true` (high) = recessive.
    /// Idle is recessive.
    pub tx: bool,
    /// High while a frame is being transmitted.
    pub busy: bool,
    /// Pulses for one cycle when a frame finishes.
    pub done: bool,
}

impl<const DIV_W: usize> SynchronousIO for CanMaster<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = can_master<DIV_W>;
}

#[kernel]
/// Kernel for [CanMaster].
pub fn can_master<const DIV_W: usize>(
    cr: ClockReset,
    i: In,
    q: Q<DIV_W>,
) -> (Out, D<DIV_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    let one_div: Bits<DIV_W> = bits::<DIV_W>(1);
    let zero_div: Bits<DIV_W> = bits::<DIV_W>(0);
    let one_b7: Bits<7> = bits::<7>(1);
    let zero_b7: Bits<7> = bits::<7>(0);
    let zero_b3: Bits<3> = bits::<3>(0);
    let one_b3: Bits<3> = bits::<3>(1);
    let four_b3: Bits<3> = bits::<3>(4);

    let mut d = D::<DIV_W>::dont_care();
    d.state = q.state;
    d.field = q.field;
    d.field_bit_idx = q.field_bit_idx;
    d.bit_phase_counter = q.bit_phase_counter;
    d.id_reg = q.id_reg;
    d.dlc_reg = q.dlc_reg;
    d.data_reg = q.data_reg;
    d.crc_reg = q.crc_reg;
    d.stuff = q.stuff;
    d.done_pulse = false;

    // The "raw" frame bit at the current field/index — what would go
    // out if there were no stuffing.  Computed combinationally from q.
    // All positions computed in Bits<7> (the width of field_bit_idx),
    // and we rely on the generic `Shr<Bits<M>> for Bits<N>` impl rather
    // than converting the index to the register's width.
    let raw_bit = match q.field {
        // SOF + RTR (data frame) + IDE (standard ID) + r0 (reserved) all
        // emit a dominant bit.
        CanField::Sof | CanField::Rtr | CanField::Ide | CanField::R0 => false,
        CanField::Id => {
            // id MSB-first: bit_idx 0 → id[10], bit_idx 10 → id[0].
            let pos = bits::<7>(10) - q.field_bit_idx;
            ((q.id_reg >> pos) & bits::<11>(1)) != bits::<11>(0)
        }
        CanField::Dlc => {
            // dlc MSB-first: bit_idx 0 → dlc[3], bit_idx 3 → dlc[0].
            let pos = bits::<7>(3) - q.field_bit_idx;
            ((q.dlc_reg >> pos) & bits::<4>(1)) != bits::<4>(0)
        }
        CanField::Data => {
            // data MSB-first: bit_idx 0 → data[63].
            let pos = bits::<7>(63) - q.field_bit_idx;
            ((q.data_reg >> pos) & bits::<64>(1)) != bits::<64>(0)
        }
        CanField::Crc => {
            // crc MSB-first.
            let pos = bits::<7>(14) - q.field_bit_idx;
            ((q.crc_reg >> pos) & bits::<15>(1)) != bits::<15>(0)
        }
        // CRC delim, ACK slot (we drive recessive; receivers ack by going
        // dominant), ACK delim, EOF, and IFS are all recessive.
        CanField::CrcDelim
        | CanField::AckSlot
        | CanField::AckDelim
        | CanField::Eof
        | CanField::Ifs => true,
    };

    // The wire bit: stuff bit if pending, else raw_bit.
    let wire_bit = if q.stuff.pending {
        !q.stuff.last_bit
    } else {
        raw_bit
    };

    // Stuffing applies in fields SOF through CRC inclusive.
    let in_stuff_zone = match q.field {
        CanField::Sof
        | CanField::Id
        | CanField::Rtr
        | CanField::Ide
        | CanField::R0
        | CanField::Dlc
        | CanField::Data
        | CanField::Crc => true,
        _ => false,
    };

    // CRC update: per-bit, on non-stuff bits, in fields SOF through end of data.
    // Polynomial 0x4599, MSB-first shift.  Update happens when we *commit* a wire
    // bit (per CAN bit period), so we need to know if this commit is happening
    // *now* — we update at the same edge as the bit advance.
    let crc_input_active = match q.field {
        CanField::Sof
        | CanField::Id
        | CanField::Rtr
        | CanField::Ide
        | CanField::R0
        | CanField::Dlc
        | CanField::Data => true,
        _ => false,
    };

    // Top of CRC register.
    let crc_top = (q.crc_reg >> bits::<15>(14)) & bits::<15>(1);
    let crc_top_set = crc_top != bits::<15>(0);
    // CRC step: shift left, XOR with poly if (top XOR new_bit) != 0.
    let crc_feedback = crc_top_set != raw_bit;
    let crc_shifted = q.crc_reg << 1;
    let crc_stepped = if crc_feedback {
        (crc_shifted ^ bits::<15>(0x4599)) & bits::<15>(0x7FFF)
    } else {
        crc_shifted & bits::<15>(0x7FFF)
    };

    // Bit-period counter advances.
    let bit_done = q.bit_phase_counter == (q.bit_period - one_div);
    if q.state == CanState::Idle {
        if i.start {
            d.state = CanState::Tx;
            d.field = CanField::Sof;
            d.field_bit_idx = zero_b7;
            d.bit_phase_counter = zero_div;
            d.id_reg = i.id;
            d.dlc_reg = i.dlc;
            d.data_reg = i.data;
            d.crc_reg = bits::<15>(0);
            d.stuff = StuffState {
                last_bit: true, // idle was recessive
                run: zero_b3,
                pending: false,
            };
        }
    } else {
        // Tx state: count bit_phase_counter; advance frame on bit boundary.
        if bit_done {
            d.bit_phase_counter = zero_div;
            // Update stuff tracker based on the wire bit we just committed.
            let new_run = if wire_bit == q.stuff.last_bit {
                q.stuff.run + one_b3
            } else {
                one_b3
            };
            // Decide if next bit is a stuff bit.
            let next_will_stuff = in_stuff_zone && new_run == bits::<3>(5);
            d.stuff = StuffState {
                last_bit: wire_bit,
                run: new_run,
                pending: next_will_stuff,
            };
            // Advance the field only if this bit was NOT a stuff bit.
            if q.stuff.pending {
                // We just sent a stuff bit; do not advance frame.
            } else {
                // Update CRC if applicable.
                if crc_input_active {
                    d.crc_reg = crc_stepped;
                }
                // Advance field/bit_idx.
                let next_idx = q.field_bit_idx + one_b7;
                match q.field {
                    CanField::Sof => {
                        d.field = CanField::Id;
                        d.field_bit_idx = zero_b7;
                    }
                    CanField::Id => {
                        if q.field_bit_idx == bits::<7>(10) {
                            d.field = CanField::Rtr;
                            d.field_bit_idx = zero_b7;
                        } else {
                            d.field_bit_idx = next_idx;
                        }
                    }
                    CanField::Rtr => {
                        d.field = CanField::Ide;
                        d.field_bit_idx = zero_b7;
                    }
                    CanField::Ide => {
                        d.field = CanField::R0;
                        d.field_bit_idx = zero_b7;
                    }
                    CanField::R0 => {
                        d.field = CanField::Dlc;
                        d.field_bit_idx = zero_b7;
                    }
                    CanField::Dlc => {
                        if q.field_bit_idx == bits::<7>(3) {
                            // Done with DLC.  If dlc==0, skip Data and go straight to CRC.
                            if q.dlc_reg == bits::<4>(0) {
                                d.field = CanField::Crc;
                                d.field_bit_idx = zero_b7;
                            } else {
                                d.field = CanField::Data;
                                d.field_bit_idx = zero_b7;
                            }
                        } else {
                            d.field_bit_idx = next_idx;
                        }
                    }
                    CanField::Data => {
                        // total data bits = dlc * 8.  DLC is at most 8 (CAN spec)
                        // so the result fits in Bits<7>.  Decoded as a const-bound
                        // match on q.dlc_reg, since we cannot turbofish `as_bits`
                        // inside a kernel.
                        let total_data_bits: Bits<7> = match q.dlc_reg {
                            Bits::<4>(0) => bits::<7>(0),
                            Bits::<4>(1) => bits::<7>(8),
                            Bits::<4>(2) => bits::<7>(16),
                            Bits::<4>(3) => bits::<7>(24),
                            Bits::<4>(4) => bits::<7>(32),
                            Bits::<4>(5) => bits::<7>(40),
                            Bits::<4>(6) => bits::<7>(48),
                            Bits::<4>(7) => bits::<7>(56),
                            Bits::<4>(8) => bits::<7>(64),
                            _ => bits::<7>(64), // DLC > 8: clamp to 8 bytes per spec
                        };
                        if next_idx == total_data_bits {
                            d.field = CanField::Crc;
                            d.field_bit_idx = zero_b7;
                        } else {
                            d.field_bit_idx = next_idx;
                        }
                    }
                    CanField::Crc => {
                        if q.field_bit_idx == bits::<7>(14) {
                            d.field = CanField::CrcDelim;
                            d.field_bit_idx = zero_b7;
                        } else {
                            d.field_bit_idx = next_idx;
                        }
                    }
                    CanField::CrcDelim => {
                        d.field = CanField::AckSlot;
                        d.field_bit_idx = zero_b7;
                    }
                    CanField::AckSlot => {
                        d.field = CanField::AckDelim;
                        d.field_bit_idx = zero_b7;
                    }
                    CanField::AckDelim => {
                        d.field = CanField::Eof;
                        d.field_bit_idx = zero_b7;
                    }
                    CanField::Eof => {
                        if q.field_bit_idx == bits::<7>(6) {
                            d.field = CanField::Ifs;
                            d.field_bit_idx = zero_b7;
                        } else {
                            d.field_bit_idx = next_idx;
                        }
                    }
                    CanField::Ifs => {
                        if q.field_bit_idx == bits::<7>(2) {
                            d.state = CanState::Idle;
                            d.field = CanField::Sof;
                            d.field_bit_idx = zero_b7;
                            d.done_pulse = true;
                        } else {
                            d.field_bit_idx = next_idx;
                        }
                    }
                }
            }
        } else {
            d.bit_phase_counter = q.bit_phase_counter + one_div;
        }
    }

    if cr.reset.any() {
        d.state = CanState::Idle;
        d.field = CanField::Sof;
        d.field_bit_idx = zero_b7;
        d.bit_phase_counter = zero_div;
        d.id_reg = bits::<11>(0);
        d.dlc_reg = bits::<4>(0);
        d.data_reg = bits::<64>(0);
        d.crc_reg = bits::<15>(0);
        d.stuff = StuffState {
            last_bit: true,
            run: zero_b3,
            pending: false,
        };
        d.done_pulse = false;
    }

    let _ = four_b3;
    let tx = if q.state == CanState::Idle {
        true // recessive idle
    } else {
        wire_bit
    };
    let busy = q.state != CanState::Idle;

    let mut o = Out::dont_care();
    o.tx = tx;
    o.busy = busy;
    o.done = q.done_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            id: bits(0),
            dlc: bits(0),
            data: bits(0),
            start: false,
        }
    }

    // Tier 2 — drive a frame and verify done eventually pulses.
    #[test]
    fn test_frame_completes() -> miette::Result<()> {
        let bit_period = 4u128;
        let uut = CanMaster::<5>::new(bits(bit_period));
        // Frame size estimate: SOF(1) + ID(11) + Ctrl(3) + DLC(4) + Data(8*1=8) +
        // CRC(15) + Delims(3) + EOF(7) + IFS(3) = 55 bits, plus up to ~10 stuff
        // bits ≈ 65 bits × 4 cycles + slack.
        let n_cycles = 65 * (bit_period as usize) + 50;
        let mut stream_in: Vec<In> = vec![In {
            id: bits(0x123),
            dlc: bits(1),
            data: bits(0xA5_00_00_00_00_00_00_00),
            start: true,
        }];
        for _ in 0..n_cycles {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_idx = outputs.iter().position(|s| s.output.done);
        assert!(done_idx.is_some(), "no done pulse");
        Ok(())
    }

    #[test]
    fn test_idle_line_recessive() -> miette::Result<()> {
        let uut = CanMaster::<5>::new(bits(4));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_dominant = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| !s.output.tx || s.output.busy);
        assert!(!any_dominant, "idle line should be recessive and not busy");
        Ok(())
    }

    #[test]
    fn test_frame_starts_with_sof_dominant() -> miette::Result<()> {
        let uut = CanMaster::<5>::new(bits(4));
        let mut stream_in: Vec<In> = vec![In {
            id: bits(0x555),
            dlc: bits(0),
            data: bits(0),
            start: true,
        }];
        for _ in 0..200 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Look for the first cycle where tx goes low; that's the start of SOF.
        let sof_idx = outputs.iter().position(|s| !s.output.tx);
        assert!(sof_idx.is_some(), "tx never went dominant — no SOF detected");
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = CanMaster::<5>::new(bits(4));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["26937"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_can_master_hdl_works() -> miette::Result<()> {
        let uut = CanMaster::<5>::new(bits(4));
        let mut stream_in: Vec<In> = vec![In {
            id: bits(0x123),
            dlc: bits(1),
            data: bits(0xA5_00_00_00_00_00_00_00),
            start: true,
        }];
        for _ in 0..400 {
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
    fn test_can_master_trace() -> miette::Result<()> {
        let uut = CanMaster::<5>::new(bits(4));
        let mut stream_in: Vec<In> = vec![In {
            id: bits(0x123),
            dlc: bits(1),
            data: bits(0xA5_00_00_00_00_00_00_00),
            start: true,
        }];
        for _ in 0..400 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("can_master");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["d74611f07753c780ce7d803268c31f3701b395e96e6ce09575634f3f40c01ef1"];
        let digest = vcd.dump_to_file(root.join("can_master.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    // -----------------------------------------------------------
    // FSM-tooling validation: the #[derive(Fsm)] + #[derive(FsmWidget)]
    // metadata is well-formed and lines up with the source enum.
    // -----------------------------------------------------------

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = CanMaster::<5>::fsm_descriptor();
        assert_eq!(desc.widget_name, "CanMaster");
        assert_eq!(desc.widget.state_field, "field");
        assert_eq!(desc.kernel.state_var, "q.field");
        let variants = desc.variants();
        assert_eq!(variants.len(), 13);
        assert_eq!(variants[0].name, "Sof");
        assert_eq!(variants[0].label, Some("SOF"));
        assert_eq!(variants[12].name, "Ifs");
        // Sof is the #[default] — initial index is 0.
        assert_eq!(desc.initial_index(), 0);
    }
}
