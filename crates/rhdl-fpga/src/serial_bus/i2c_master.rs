//! I2C master (write-only, single-byte v1)
//!
//! Standard 7-bit I2C master that performs single-byte WRITE
//! transactions: `START → addr+W → ACK → data → ACK → STOP`.
//! This v1 hardcodes the write direction (R/W bit = 0), one
//! data byte per transaction, and no clock stretching.  Read
//! transactions and multi-byte bursts are tracked as follow-ups.
//!
//! The master drives **open-drain** outputs: `scl_drive_low` and
//! `sda_drive_low` are high when the master is *actively pulling
//! the line low*, low when the master *releases* the line (at
//! which point an external pull-up resistor takes over and the
//! line goes high).  Wrap each output with [super::super::tristate::simple]
//! to expose true `BitZ` tristate buses to the I/O pads.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+I2cMaster+-------+
     |                         |
B<7> |                         | bool
+--->| addr        scl_drv_low +--->
B<8> |                         | bool
+--->| data        sda_drv_low +--->
bool |                         | bool
+--->| start            ack_ok +--->
bool |                         | bool
+--->| sda_in             busy +--->
     |                    done +--->
     +-------------------------+
")]
//!
//!# Internals
//!
//! The bus rate is set by the `divisor` parameter at construction:
//! each of the four phases per bit (`SCL low setup`, `SCL low hold`,
//! `SCL high sample`, `SCL high hold`) takes `divisor` FPGA cycles.
//! So one I2C bit = `4 * divisor` FPGA cycles, and one byte =
//! `9 * 4 * divisor` cycles (8 data bits + 1 ACK bit).
//!
//! State machine:
//!
//! - `Idle`: lines released (high), waiting for `start`.
//! - `Start`: drive SDA low while SCL is high (the START condition).
//! - `Addr`: shift out the 7-bit address + R/W=0, MSB first.
//! - `AckAddr`: release SDA, sample for slave ACK.
//! - `Data`: shift out the 8 data bits, MSB first.
//! - `AckData`: release SDA, sample for slave ACK.
//! - `Stop`: drive SDA low while raising SCL, then release SDA
//!   (the STOP condition).
//!
//!# Behavior
//!
//! - `scl_drive_low == 1` ⇒ master is pulling SCL low.
//! - `sda_drive_low == 1` ⇒ master is pulling SDA low.
//! - `sda_in` is the sampled value of SDA (after the external
//!   pull-up resolution).  The master uses it during ACK phases.
//! - `ack_ok` is high after a successful transaction (both ACKs
//!   were observed low).  Held until the next transaction begins.
//! - `done` pulses for one cycle at the end of the STOP condition.
//!
//!# Parameters
//!
//! - `DIV_W` — bit width of the per-phase divisor counter
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/i2c_master.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/i2c_master.md")]
//!
//! And the auto-generated FSM diagram for the I2C transaction:
#![doc = include_str!("../../doc/i2c_master_fsm.md")]
use rhdl::prelude::*;

use rhdl::core::fsm::analysis::Transition;

use crate::core::{constant::Constant, dff};

/// State of the I2C transaction.
///
/// Encoded as a 3-bit `Digital` enum so it fits in a tight DFF.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum I2cState {
    /// Bus idle.  Both lines released to the pull-up.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Driving SDA low while SCL is high — the START condition.
    #[fsm_state(label = "START")]
    Start,
    /// Shifting out the 7-bit address + R/W bit, MSB-first.
    Addr,
    /// Released SDA, sampling for slave's address-ACK.
    #[fsm_state(label = "ACK addr")]
    AckAddr,
    /// Shifting out the 8-bit data byte, MSB-first.
    Data,
    /// Released SDA, sampling for slave's data-ACK.
    #[fsm_state(label = "ACK data")]
    AckData,
    /// Driving SDA low while raising SCL, then releasing — the STOP condition.
    #[fsm_state(label = "STOP")]
    Stop,
}

/// Author-curated transition graph for the I2C transaction FSM.
///
/// Required by CLAUDE.md §12 rule 14.  Indices match `I2cState`
/// declaration order (Idle=0, Start=1, Addr=2, AckAddr=3, Data=4,
/// AckData=5, Stop=6).
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → Start (on `start` strobe)
    Transition {
        source_index: 1,
        target_index: 2,
    }, // Start → Addr
    Transition {
        source_index: 2,
        target_index: 2,
    }, // Addr self-loop (per bit)
    Transition {
        source_index: 2,
        target_index: 3,
    }, // Addr → AckAddr (after 8 bits)
    Transition {
        source_index: 3,
        target_index: 4,
    }, // AckAddr → Data
    Transition {
        source_index: 4,
        target_index: 4,
    }, // Data self-loop (per bit)
    Transition {
        source_index: 4,
        target_index: 5,
    }, // Data → AckData
    Transition {
        source_index: 5,
        target_index: 6,
    }, // AckData → Stop
    Transition {
        source_index: 6,
        target_index: 0,
    }, // Stop → Idle
];

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = I2cState)]
/// I2C master (write-only, single-byte v1).
pub struct I2cMaster<const DIV_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    state: dff::DFF<I2cState>,
    /// Sub-counter within the current phase (0..divisor-1).
    phase_sub: dff::DFF<Bits<DIV_W>>,
    /// Phase within the current bit: 0..3 (low setup, low hold, high sample, high hold).
    phase: dff::DFF<Bits<2>>,
    /// Bit index within the current byte: 0..8 (8 data + 1 ACK).
    bit_idx: dff::DFF<Bits<4>>,
    /// Address shift register (latched at start).
    addr_reg: dff::DFF<Bits<8>>,
    /// Data shift register (latched at start).
    data_reg: dff::DFF<Bits<8>>,
    /// ACK results from address and data bytes.
    ack_addr_ok: dff::DFF<bool>,
    ack_data_ok: dff::DFF<bool>,
    /// One-cycle done pulse.
    done_pulse: dff::DFF<bool>,
    divisor: Constant<Bits<DIV_W>>,
}

impl<const DIV_W: usize> I2cMaster<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    /// Create an I2C master with the given per-phase divisor.
    /// Total bit time = `4 * divisor` FPGA cycles.
    pub fn new(divisor: Bits<DIV_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            phase_sub: dff::DFF::default(),
            phase: dff::DFF::default(),
            bit_idx: dff::DFF::default(),
            addr_reg: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            ack_addr_ok: dff::DFF::default(),
            ack_data_ok: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            divisor: Constant::new(divisor),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [I2cMaster].
pub struct In {
    /// 7-bit slave address (latched at `start`).
    pub addr: Bits<7>,
    /// 8-bit data byte to write (latched at `start`).
    pub data: Bits<8>,
    /// Strobe to begin a transaction.  Ignored while `busy`.
    pub start: bool,
    /// Sampled SDA value (after external pull-up resolution).
    pub sda_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [I2cMaster].
pub struct Out {
    /// `1` ⇒ master pulls SCL low; `0` ⇒ master releases SCL.
    pub scl_drive_low: bool,
    /// `1` ⇒ master pulls SDA low; `0` ⇒ master releases SDA.
    pub sda_drive_low: bool,
    /// High while a transaction is in progress.
    pub busy: bool,
    /// Pulses for one cycle at end of transaction.
    pub done: bool,
    /// `true` if the last transaction was successfully ACKed by the slave
    /// (both address and data ACKs observed low).
    pub ack_ok: bool,
}

impl<const DIV_W: usize> SynchronousIO for I2cMaster<DIV_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = i2c_master<DIV_W>;
}

#[kernel]
/// Kernel for [I2cMaster].
pub fn i2c_master<const DIV_W: usize>(cr: ClockReset, i: In, q: Q<DIV_W>) -> (Out, D<DIV_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
{
    let one_div: Bits<DIV_W> = bits::<DIV_W>(1);
    let zero_div: Bits<DIV_W> = bits::<DIV_W>(0);
    let one_b2: Bits<2> = bits::<2>(1);
    let zero_b2: Bits<2> = bits::<2>(0);
    let three_b2: Bits<2> = bits::<2>(3);
    let one_b4: Bits<4> = bits::<4>(1);
    let zero_b4: Bits<4> = bits::<4>(0);
    let eight_b4: Bits<4> = bits::<4>(8);
    let zero_b8: Bits<8> = bits::<8>(0);

    let mut d = D::<DIV_W>::dont_care();
    // Default: hold; clear pulse.
    d.state = q.state;
    d.phase_sub = q.phase_sub;
    d.phase = q.phase;
    d.bit_idx = q.bit_idx;
    d.addr_reg = q.addr_reg;
    d.data_reg = q.data_reg;
    d.ack_addr_ok = q.ack_addr_ok;
    d.ack_data_ok = q.ack_data_ok;
    d.done_pulse = false;

    let phase_done = q.phase_sub == (q.divisor - one_div);

    // Determine if the current state is one that takes "1 bit time" (4 phases)
    // — Idle takes 0; everything else does.
    let in_byte_phase = match q.state {
        I2cState::Idle => false,
        // Every non-Idle state takes one bit time (4 phases).
        I2cState::Start
        | I2cState::Addr
        | I2cState::AckAddr
        | I2cState::Data
        | I2cState::AckData
        | I2cState::Stop => true,
    };

    if !in_byte_phase {
        // Idle: latch on start.
        if i.start {
            // Latch operands and prepare START.
            d.state = I2cState::Start;
            d.phase = zero_b2;
            d.phase_sub = zero_div;
            d.bit_idx = zero_b4;
            // addr_reg layout: addr in bits 7:1, R/W (= 0 for write) in bit 0,
            // sent MSB-first by the Addr state.
            d.addr_reg = (i.addr.resize::<8>()) << 1;
            d.data_reg = i.data;
            d.ack_addr_ok = false;
            d.ack_data_ok = false;
        }
    } else {
        // Sample slave ACK at the start of phase==2 of an Ack* state.
        let sample_ack = q.phase == one_b2 && q.phase_sub == zero_div;
        if sample_ack {
            match q.state {
                I2cState::AckAddr => {
                    if !i.sda_in {
                        d.ack_addr_ok = true;
                    }
                }
                I2cState::AckData => {
                    if !i.sda_in {
                        d.ack_data_ok = true;
                    }
                }
                _ => {}
            }
        }
        // Advance the phase / sub-phase.
        if phase_done {
            d.phase_sub = zero_div;
            if q.phase == three_b2 {
                d.phase = zero_b2;
                // End of one bit time — advance state machine.
                match q.state {
                    I2cState::Start => {
                        d.state = I2cState::Addr;
                        d.bit_idx = zero_b4;
                    }
                    I2cState::Addr => {
                        let next_bit = q.bit_idx + one_b4;
                        // Shift addr_reg left for next bit.
                        d.addr_reg = q.addr_reg << 1;
                        if next_bit == eight_b4 {
                            d.state = I2cState::AckAddr;
                            d.bit_idx = zero_b4;
                        } else {
                            d.bit_idx = next_bit;
                        }
                    }
                    I2cState::AckAddr => {
                        d.state = I2cState::Data;
                        d.bit_idx = zero_b4;
                    }
                    I2cState::Data => {
                        let next_bit = q.bit_idx + one_b4;
                        d.data_reg = q.data_reg << 1;
                        if next_bit == eight_b4 {
                            d.state = I2cState::AckData;
                            d.bit_idx = zero_b4;
                        } else {
                            d.bit_idx = next_bit;
                        }
                    }
                    I2cState::AckData => {
                        d.state = I2cState::Stop;
                        d.bit_idx = zero_b4;
                    }
                    I2cState::Stop => {
                        d.state = I2cState::Idle;
                        d.done_pulse = true;
                    }
                    I2cState::Idle => {
                        d.state = I2cState::Idle;
                    }
                }
            } else {
                d.phase = q.phase + one_b2;
            }
        } else {
            d.phase_sub = q.phase_sub + one_div;
        }
    }

    if cr.reset.any() {
        d.state = I2cState::Idle;
        d.phase_sub = zero_div;
        d.phase = zero_b2;
        d.bit_idx = zero_b4;
        d.addr_reg = zero_b8;
        d.data_reg = zero_b8;
        d.ack_addr_ok = false;
        d.ack_data_ok = false;
        d.done_pulse = false;
    }

    // === Outputs (combinational from current state) ===
    // SCL: low during phase 0 and phase 3, high during phase 1 and phase 2.
    // (For Start: SCL high throughout; for Stop: low at start, high at end.)
    let mut scl_drive_low = false;
    let mut sda_drive_low = false;
    match q.state {
        I2cState::Idle => {
            // Both released.
        }
        I2cState::Start => {
            // SCL stays high throughout START.
            // SDA: high in phase 0, low in phases 1,2,3 (falls while SCL high).
            if q.phase != zero_b2 {
                sda_drive_low = true;
            }
        }
        I2cState::Addr => {
            // SCL: low in phase 0,3 (drive low); high in phase 1,2 (release).
            if q.phase == zero_b2 || q.phase == three_b2 {
                scl_drive_low = true;
            }
            // SDA: drive based on current MSB of the addr shift register.
            let bit_val = (q.addr_reg >> bits::<8>(7)) & bits::<8>(1);
            if bit_val == zero_b8 {
                sda_drive_low = true;
            }
        }
        I2cState::Data => {
            if q.phase == zero_b2 || q.phase == three_b2 {
                scl_drive_low = true;
            }
            let bit_val = (q.data_reg >> bits::<8>(7)) & bits::<8>(1);
            if bit_val == zero_b8 {
                sda_drive_low = true;
            }
        }
        // ACK slots: SCL toggles like the data phases; SDA stays
        // released so the addressed slave can drive it low.
        I2cState::AckAddr | I2cState::AckData => {
            if q.phase == zero_b2 || q.phase == three_b2 {
                scl_drive_low = true;
            }
        }
        I2cState::Stop => {
            // SCL: low in phase 0, high in phases 1,2,3 (rises during STOP).
            // SDA: low in phases 0,1, high in phases 2,3 (rises while SCL high).
            if q.phase == zero_b2 {
                scl_drive_low = true;
            }
            if q.phase == zero_b2 || q.phase == one_b2 {
                sda_drive_low = true;
            }
        }
    }

    let busy = q.state != I2cState::Idle;
    let ack_ok = q.ack_addr_ok && q.ack_data_ok;

    let mut o = Out::dont_care();
    o.scl_drive_low = scl_drive_low;
    o.sda_drive_low = sda_drive_low;
    o.busy = busy;
    o.done = q.done_pulse;
    o.ack_ok = ack_ok;
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
            data: bits(0),
            start: false,
            sda_in: true, // pull-up
        }
    }

    // Tier 2 — drive a write transaction with a "perfect slave" (always ACKs).
    #[test]
    fn test_write_transaction_completes() -> miette::Result<()> {
        let divisor = 2;
        let uut = I2cMaster::<4>::new(bits(divisor));
        // Build the input sequence.  The "slave" model: SDA pulled low
        // by master if sda_drive_low.  Otherwise pulled low by slave
        // during ACK phases (we just always say sda_in=false during Ack
        // phase 1 / phase_sub 0 sample point).
        // Easier: just compute SDA based on the master's outputs.
        // Drive start=true for the first cycle.
        // Total cycle count: idle 2 + 1+8+1+8+1 = 19 bit times × 4 phases × divisor = 152 cycles + slack.
        let n_cycles = 200;
        let mut stream_in: Vec<In> = Vec::with_capacity(n_cycles);
        for k in 0..n_cycles {
            let mut inp = idle_in();
            if k == 0 {
                inp.addr = bits(0x42);
                inp.data = bits(0x55);
                inp.start = true;
            }
            // For sda_in: simulated slave ACKs by pulling low.
            // We'll just set it low always except during the data-out phases
            // where the master is driving.  The master only samples SDA during
            // ACK phases; setting it low there means "slave ACKed".
            inp.sda_in = false;
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect::<Vec<_>>();
        let done_idx = outputs.iter().position(|s| s.output.done);
        assert!(done_idx.is_some(), "no done pulse seen");
        let final_ack = outputs[done_idx.unwrap()].output.ack_ok;
        assert!(final_ack, "expected ack_ok at done");
        Ok(())
    }

    #[test]
    fn test_idle_releases_lines() -> miette::Result<()> {
        let uut = I2cMaster::<4>::new(bits(2));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_drive = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.scl_drive_low || s.output.sda_drive_low);
        assert!(!any_drive);
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = I2cMaster::<4>::new(bits(2));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["20628"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_i2c_master_hdl_works() -> miette::Result<()> {
        let uut = I2cMaster::<4>::new(bits(2));
        let mut stream_in: Vec<In> = vec![In {
            addr: bits(0x42),
            data: bits(0x55),
            start: true,
            sda_in: false,
        }];
        for _ in 0..200 {
            let mut inp = idle_in();
            inp.sda_in = false;
            stream_in.push(inp);
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
    fn test_i2c_master_trace() -> miette::Result<()> {
        let uut = I2cMaster::<4>::new(bits(2));
        let mut stream_in: Vec<In> = vec![In {
            addr: bits(0x42),
            data: bits(0x55),
            start: true,
            sda_in: false,
        }];
        for _ in 0..160 {
            let mut inp = idle_in();
            inp.sda_in = false;
            stream_in.push(inp);
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("i2c_master");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["9b859e31177c024e941a4de60489ef1da30497cd8f59b8f59754f72520155abb"];
        let digest = vcd.dump_to_file(root.join("i2c_master.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
