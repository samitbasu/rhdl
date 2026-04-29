//! Texas Instruments HDQ single-wire master
//!
//! HDQ is TI's proprietary single-wire bus, used to talk to the
//! `bq2018x` / `bq3xxx` series fuel-gauge ICs in laptop and
//! smartphone batteries.  Electrically it looks like 1-Wire — a
//! single open-drain line idling high through an external pull-up —
//! but the framing is different: every transaction starts with a
//! *break* pulse (a long low), followed by an 8-bit address byte
//! (MSB = R/W) and an 8-bit data byte (sent by the host on writes,
//! returned by the slave on reads).
//!
//! **v1 scope:** primitive operations only.  Three ops:
//!
//! - `Break` — drive the line low for `t_break`, then release for
//!   `t_break_recovery`.  The host must issue a Break before each
//!   transaction.
//! - `WriteByte` — transmit one byte LSB-first using the HDQ bit
//!   timing (long-low for 0, short-low for 1).
//! - `ReadByte` — drive the line low for `t_read_low`, sample at
//!   `t_read_sample` past release, repeat for 8 bits.
//!
//! The host sequences these directly: `Break → WriteByte (addr) →
//! WriteByte (data)` for a write; `Break → WriteByte (addr) →
//! ReadByte` for a read.  A higher-level register-mapped wrapper
//! (multi-byte transactions, retry on parity error, periodic
//! refresh) is v2.
//!
//! The output pair `(bus_oe, bus_out)` is identical to 1-Wire's:
//! the host wraps it with [super::super::tristate::simple] (or just
//! gates an open-drain pad).  `bus_oe = true` while we are pulling
//! the line low; otherwise the external pull-up holds it high.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +----------+TiHdqMaster+----------+
     |                                 |
HDQop|                                 | bool
+--->| op                       bus_oe +--->
B<8> |                                 | bool
+--->| data                    bus_out +--->
bool |                                 | B<8>
+--->| start                  data_out +--->
bool |                                 | bool
+--->| bus_in                     busy +--->
     |                            done +--->
     +---------------------------------+
")]
//!
//!# Internals
//!
//! Bus timings — all in *FPGA cycles* — are passed in via a
//! [TiHdqTimings] struct.  Typical TI HDQ standard-mode timings at
//! 100 MHz (multiply microseconds by 100):
//!
//! | Field              | Standard | Fast (HDQ16) |
//! |--------------------|----------|--------------|
//! | `t_break`          | 190 µs   | 5 µs         |
//! | `t_break_recovery` | 40 µs    | 5 µs         |
//! | `t_w0`             | 100 µs   | 6 µs         |
//! | `t_w1`             | 32 µs    | 1 µs         |
//! | `t_read_low`       | 32 µs    | 1 µs         |
//! | `t_read_sample`    | 64 µs    | 4 µs         |
//! | `t_slot`           | 190 µs   | 7 µs         |
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/ti_hdq.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/ti_hdq.md")]
//!
//! And the auto-generated FSM diagram for the slot-walking machine:
#![doc = include_str!("../../doc/ti_hdq_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the HDQ FSM.
///
/// Required by CLAUDE.md §12 rule 14.  Indices match `TiHdqState`
/// declaration order (Idle=0, BreakLow=1, BreakRecover=2,
/// WriteBitLow=3, WriteBitWait=4, ReadBitLow=5, ReadBitSample=6,
/// Stop=7).
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    },
    Transition {
        source_index: 0,
        target_index: 3,
    },
    Transition {
        source_index: 0,
        target_index: 5,
    },
    Transition {
        source_index: 1,
        target_index: 2,
    },
    Transition {
        source_index: 2,
        target_index: 7,
    },
    Transition {
        source_index: 3,
        target_index: 4,
    },
    Transition {
        source_index: 4,
        target_index: 3,
    },
    Transition {
        source_index: 4,
        target_index: 7,
    },
    Transition {
        source_index: 5,
        target_index: 6,
    },
    Transition {
        source_index: 6,
        target_index: 5,
    },
    Transition {
        source_index: 6,
        target_index: 7,
    },
    Transition {
        source_index: 7,
        target_index: 0,
    },
];

/// Operation to perform.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default)]
pub enum TiHdqOp {
    /// Send a break pulse (line low, then released for recovery).
    #[default]
    Break,
    /// Transmit one byte LSB-first.
    WriteByte,
    /// Receive one byte LSB-first.
    ReadByte,
}

/// Internal state machine.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum TiHdqState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    #[fsm_state(label = "Break (low)")]
    BreakLow,
    #[fsm_state(label = "Break (recover)")]
    BreakRecover,
    #[fsm_state(label = "Write (low)")]
    WriteBitLow,
    #[fsm_state(label = "Write (wait)")]
    WriteBitWait,
    #[fsm_state(label = "Read (low)")]
    ReadBitLow,
    #[fsm_state(label = "Read (sample)")]
    ReadBitSample,
    #[fsm_state(label = "stop")]
    Stop,
}

/// Bus-timing parameters, all in *FPGA cycles*.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct TiHdqTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Break-pulse width (line driven low).
    pub t_break: Bits<T_W>,
    /// Recovery time after break (line released, no bit yet).
    pub t_break_recovery: Bits<T_W>,
    /// Low-pulse width for writing a `0` bit.
    pub t_w0: Bits<T_W>,
    /// Low-pulse width for writing a `1` bit.
    pub t_w1: Bits<T_W>,
    /// Low-pulse width that begins a read bit slot.
    pub t_read_low: Bits<T_W>,
    /// Time after the read low pulse at which to sample `bus_in`.
    pub t_read_sample: Bits<T_W>,
    /// Total slot duration (after release) for write or read bit slots.
    pub t_slot: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = TiHdqState)]
/// TI HDQ single-wire master.
pub struct TiHdqMaster<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<TiHdqState>,
    tick: dff::DFF<Bits<T_W>>,
    op_reg: dff::DFF<TiHdqOp>,
    data_reg: dff::DFF<Bits<8>>,
    bit_idx: dff::DFF<Bits<4>>,
    done_pulse: dff::DFF<bool>,
    timings: Constant<TiHdqTimings<T_W>>,
}

impl<const T_W: usize> TiHdqMaster<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Create an HDQ master with the given timings (in FPGA cycles).
    pub fn new(timings: TiHdqTimings<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            op_reg: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            bit_idx: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [TiHdqMaster].
pub struct In {
    pub op: TiHdqOp,
    /// Byte to transmit (used only when `op == WriteByte`).
    pub data: Bits<8>,
    /// One-cycle strobe to begin the transaction.  Ignored while busy.
    pub start: bool,
    /// Sampled level of the bus.
    pub bus_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [TiHdqMaster].
pub struct Out {
    pub bus_oe: bool,
    pub bus_out: bool,
    /// Result byte from the most-recent ReadByte (LSB-first).
    pub data_out: Bits<8>,
    pub busy: bool,
    pub done: bool,
}

impl<const T_W: usize> SynchronousIO for TiHdqMaster<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = ti_hdq<T_W>;
}

#[kernel]
/// Kernel for [TiHdqMaster].
pub fn ti_hdq<const T_W: usize>(cr: ClockReset, i: In, q: Q<T_W>) -> (Out, D<T_W>)
where
    rhdl::bits::W<T_W>: BitWidth,
{
    let one_t: Bits<T_W> = bits::<T_W>(1);
    let zero_t: Bits<T_W> = bits::<T_W>(0);
    let one_b4: Bits<4> = bits::<4>(1);
    let zero_b4: Bits<4> = bits::<4>(0);

    let t = q.timings;

    let mut d = D::<T_W>::dont_care();
    d.state = q.state;
    d.tick = q.tick + one_t;
    d.op_reg = q.op_reg;
    d.data_reg = q.data_reg;
    d.bit_idx = q.bit_idx;
    d.done_pulse = false;

    let shifted = q.data_reg >> bits::<8>(1);

    match q.state {
        TiHdqState::Idle => {
            d.tick = zero_t;
            if i.start {
                d.op_reg = i.op;
                d.data_reg = i.data;
                d.bit_idx = zero_b4;
                d.tick = zero_t;
                match i.op {
                    TiHdqOp::Break => {
                        d.state = TiHdqState::BreakLow;
                    }
                    TiHdqOp::WriteByte => {
                        d.state = TiHdqState::WriteBitLow;
                    }
                    TiHdqOp::ReadByte => {
                        d.state = TiHdqState::ReadBitLow;
                    }
                }
            }
        }
        TiHdqState::BreakLow => {
            if q.tick == t.t_break {
                d.state = TiHdqState::BreakRecover;
                d.tick = zero_t;
            }
        }
        TiHdqState::BreakRecover => {
            if q.tick == t.t_break_recovery {
                d.state = TiHdqState::Stop;
                d.tick = zero_t;
            }
        }
        TiHdqState::WriteBitLow => {
            let bit_is_one = (q.data_reg & bits::<8>(1)) != bits::<8>(0);
            let pulse_width = if bit_is_one { t.t_w1 } else { t.t_w0 };
            if q.tick == pulse_width {
                d.state = TiHdqState::WriteBitWait;
                d.tick = zero_t;
            }
        }
        TiHdqState::WriteBitWait => {
            if q.tick == t.t_slot {
                if q.bit_idx == bits::<4>(7) {
                    d.state = TiHdqState::Stop;
                    d.tick = zero_t;
                } else {
                    d.bit_idx = q.bit_idx + one_b4;
                    d.data_reg = shifted;
                    d.state = TiHdqState::WriteBitLow;
                    d.tick = zero_t;
                }
            }
        }
        TiHdqState::ReadBitLow => {
            if q.tick == t.t_read_low {
                d.state = TiHdqState::ReadBitSample;
                d.tick = zero_t;
            }
        }
        TiHdqState::ReadBitSample => {
            if q.tick == t.t_read_sample {
                let bit_in = if i.bus_in {
                    bits::<8>(0x80)
                } else {
                    bits::<8>(0)
                };
                let cleared = q.data_reg & bits::<8>(0x7F);
                d.data_reg = cleared | bit_in;
            }
            if q.tick == t.t_slot {
                if q.bit_idx == bits::<4>(7) {
                    d.state = TiHdqState::Stop;
                    d.tick = zero_t;
                } else {
                    d.bit_idx = q.bit_idx + one_b4;
                    d.data_reg = q.data_reg >> bits::<8>(1);
                    d.state = TiHdqState::ReadBitLow;
                    d.tick = zero_t;
                }
            }
        }
        TiHdqState::Stop => {
            d.done_pulse = true;
            d.state = TiHdqState::Idle;
            d.tick = zero_t;
        }
    }

    let bus_oe = match q.state {
        TiHdqState::BreakLow | TiHdqState::WriteBitLow | TiHdqState::ReadBitLow => true,
        _ => false,
    };

    if cr.reset.any() {
        d.state = TiHdqState::Idle;
        d.tick = zero_t;
        d.op_reg = TiHdqOp::Break;
        d.data_reg = bits::<8>(0);
        d.bit_idx = zero_b4;
        d.done_pulse = false;
    }

    let mut o = Out::dont_care();
    o.bus_oe = bus_oe;
    o.bus_out = false;
    o.data_out = q.data_reg;
    o.busy = q.state != TiHdqState::Idle;
    o.done = q.done_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn test_timings() -> TiHdqTimings<10> {
        TiHdqTimings {
            t_break: bits(48),
            t_break_recovery: bits(8),
            t_w0: bits(20),
            t_w1: bits(4),
            t_read_low: bits(4),
            t_read_sample: bits(8),
            t_slot: bits(40),
        }
    }

    fn idle_in() -> In {
        In {
            op: TiHdqOp::Break,
            data: bits(0),
            start: false,
            bus_in: true,
        }
    }

    #[test]
    fn test_idle_releases_bus() -> miette::Result<()> {
        let uut = TiHdqMaster::<10>::new(test_timings());
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_drive = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.bus_oe);
        assert!(!any_drive);
        Ok(())
    }

    #[test]
    fn test_break_completes() -> miette::Result<()> {
        let uut = TiHdqMaster::<10>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: TiHdqOp::Break,
            data: bits(0),
            start: true,
            bus_in: true,
        }];
        for _ in 0..200 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        assert!(
            uut.run(stream)
                .synchronous_sample()
                .filter(|s| !s.input.0.reset.any())
                .any(|s| s.output.done),
            "no done pulse from Break"
        );
        Ok(())
    }

    #[test]
    fn test_write_byte_completes() -> miette::Result<()> {
        let uut = TiHdqMaster::<10>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: TiHdqOp::WriteByte,
            data: bits(0xA5),
            start: true,
            bus_in: true,
        }];
        for _ in 0..600 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        assert!(
            uut.run(stream)
                .synchronous_sample()
                .filter(|s| !s.input.0.reset.any())
                .any(|s| s.output.done),
            "no done pulse from WriteByte"
        );
        Ok(())
    }

    #[test]
    fn test_read_byte_captures_low_value() -> miette::Result<()> {
        let uut = TiHdqMaster::<10>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: TiHdqOp::ReadByte,
            data: bits(0),
            start: true,
            bus_in: false,
        }];
        for _ in 0..600 {
            stream_in.push(In {
                op: TiHdqOp::ReadByte,
                data: bits(0),
                start: false,
                bus_in: false,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_idx = outputs.iter().position(|s| s.output.done).unwrap();
        assert_eq!(outputs[done_idx].output.data_out.raw(), 0x00);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = TiHdqMaster::<10>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["14776"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_ti_hdq_hdl_works() -> miette::Result<()> {
        let uut = TiHdqMaster::<10>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: TiHdqOp::WriteByte,
            data: bits(0xA5),
            start: true,
            bus_in: true,
        }];
        for _ in 0..600 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ti_hdq_trace() -> miette::Result<()> {
        let uut = TiHdqMaster::<10>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: TiHdqOp::WriteByte,
            data: bits(0xA5),
            start: true,
            bus_in: true,
        }];
        for _ in 0..600 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ti_hdq");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["4a47a7681f0b1a2fbdf77fec625fede1951531028b889ec26f565acd686d3157"];
        let digest = vcd.dump_to_file(root.join("ti_hdq.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = TiHdqMaster::<10>::fsm_descriptor();
        assert_eq!(desc.widget_name, "TiHdqMaster");
        assert_eq!(desc.variants().len(), 8);
        assert_eq!(desc.initial_index(), 0);
    }
}
