//! NAND flash controller (ONFI 1.x async parallel)
//!
//! Drives raw NAND flash chips at the chip-level parallel
//! interface defined by the Open NAND Flash Interface (ONFI) 1.0
//! specification.  The wire set:
//!
//! - `D[7:0]` — bidirectional 8-bit data bus.
//! - `CE_n` — chip enable (active low).
//! - `CLE` — command latch enable (high while a *command* byte is
//!   driven on `D`).
//! - `ALE` — address latch enable (high while an *address* byte is
//!   driven on `D`).
//! - `WE_n` — write enable strobe (host → chip).
//! - `RE_n` — read enable strobe (chip → host).
//! - `R_B_n` — ready / busy from the chip (open-drain, active low
//!    while the chip is internally programming or erasing).
//!
//! Standard ONFI 1.0 command set: Reset (0xFF), Read ID (0x90),
//! Read Status (0x70), Page Read (0x00 + addr + 0x30), Page
//! Program (0x80 + addr + data + 0x10), Block Erase (0x60 + addr
//! + 0xD0).
//!
//! **v1 scope:** *primitive cycle* widget.  The host strobes
//! `start` with one of three operations:
//!
//! - `SendCommand` — drives `D = data`, `CLE = 1`, `ALE = 0`,
//!   pulses `WE_n` low.
//! - `SendAddress` — drives `D = data`, `CLE = 0`, `ALE = 1`,
//!   pulses `WE_n` low.
//! - `SendData` — drives `D = data`, `CLE = 0`, `ALE = 0`,
//!   pulses `WE_n` low.
//! - `ReadData` — pulses `RE_n` low, samples `D` on the rising
//!   edge of `RE_n`, exposes the byte as `data_out`.
//!
//! `R_B_n` is exposed as a passthrough output so the host can
//! poll between high-level operations.  The page-level read /
//! program / erase command sequencers (which compose these
//! primitives), the ECC pipeline, and the bidirectional data bus
//! arbitration with the host's tristate pad are all v2.
//!
//! Composes [super::super::core::dff::DFF].  The bidirectional
//! `D` bus is exposed as the `(d_oe, d_out, d_in)` triplet —
//! `d_oe` controls the host's tristate buffer at the pad; the
//! chip drives `d_in` during read cycles.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +----------+NandFlashAsync+----------+
     |                                    |
NFop |                                    | bool
+--->| op                            ce_n +--->
B<8> |                                    | bool
+--->| data                           cle +--->
B<8> |                                    | bool
+--->| d_in                           ale +--->
bool |                                    | bool
+--->| start                         we_n +--->
bool |                                    | bool
+--->| r_b_n_in                      re_n +--->
     |                               d_oe +--->
     |                              d_out +--->
     |                           data_out +--->
     |                                busy+--->
     |                                done+--->
     |                              r_b_n +--->
     +------------------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/nand_flash_async.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/nand_flash_async.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/nand_flash_async_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the NAND-cycle FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → SetupWrite
    Transition {
        source_index: 0,
        target_index: 3,
    }, // Idle → ReadLow
    Transition {
        source_index: 1,
        target_index: 2,
    }, // SetupWrite → WeLow
    Transition {
        source_index: 2,
        target_index: 5,
    }, // WeLow → Stop
    Transition {
        source_index: 3,
        target_index: 4,
    }, // ReadLow → ReadSample
    Transition {
        source_index: 4,
        target_index: 5,
    }, // ReadSample → Stop
    Transition {
        source_index: 5,
        target_index: 0,
    }, // Stop → Idle
];

/// Operation to perform on a single NAND cycle.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default)]
pub enum NandOp {
    /// Write `data` with `CLE=1` (command byte).
    #[default]
    SendCommand,
    /// Write `data` with `ALE=1` (address byte).
    SendAddress,
    /// Write `data` with `CLE=0, ALE=0` (data byte).
    SendData,
    /// Pulse `RE_n` low and sample `D` (host receives `data_out`).
    ReadData,
}

/// Internal state machine.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum NandState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Data + CLE/ALE set up; WE_n still high.
    #[fsm_state(label = "setup write")]
    SetupWrite,
    /// WE_n strobed low.
    #[fsm_state(label = "WE low")]
    WeLow,
    /// RE_n strobed low.
    #[fsm_state(label = "RE low")]
    ReadLow,
    /// RE_n still low; sampling D on this cycle's exit.
    #[fsm_state(label = "RE sample")]
    ReadSample,
    /// One-cycle done pulse and pad-time before idle.
    #[fsm_state(label = "stop")]
    Stop,
}

/// Strobe-timing parameters in *FPGA cycles*.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct NandTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Setup time: cycles between `data`/`CLE`/`ALE` valid and `WE_n` falling.
    pub t_setup: Bits<T_W>,
    /// `WE_n` low-pulse width (must be ≥ tWP per the datasheet).
    pub t_we_low: Bits<T_W>,
    /// `RE_n` low-pulse width before sampling `D` (≥ tRP).
    pub t_re_low: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = NandState)]
/// NAND flash controller (ONFI 1.x async, primitive-cycle).
pub struct NandFlashAsync<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<NandState>,
    tick: dff::DFF<Bits<T_W>>,
    op_reg: dff::DFF<NandOp>,
    data_reg: dff::DFF<Bits<8>>,
    /// Latched read byte (refreshed at the end of every ReadData cycle).
    data_out_reg: dff::DFF<Bits<8>>,
    done_pulse: dff::DFF<bool>,
    timings: Constant<NandTimings<T_W>>,
}

impl<const T_W: usize> NandFlashAsync<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Create a NAND controller with the given strobe timings (FPGA cycles).
    pub fn new(timings: NandTimings<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            op_reg: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            data_out_reg: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [NandFlashAsync].
pub struct In {
    /// Operation for the next cycle.
    pub op: NandOp,
    /// Byte to drive (write ops) — ignored for ReadData.
    pub data: Bits<8>,
    /// Sampled value of the chip's `D[7:0]` bus (used during ReadData).
    pub d_in: Bits<8>,
    /// Strobe to begin the cycle.  Ignored while busy.
    pub start: bool,
    /// Sampled R/B# input from the chip (passed through to outputs).
    pub r_b_n_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [NandFlashAsync].
pub struct Out {
    /// `CE_n` (chip enable, active low).  Held low while busy or idle —
    /// the host wraps with their own external gating if multiple chips
    /// share the bus.
    pub ce_n: bool,
    /// `CLE` (command latch enable).
    pub cle: bool,
    /// `ALE` (address latch enable).
    pub ale: bool,
    /// `WE_n` (write enable, active low).
    pub we_n: bool,
    /// `RE_n` (read enable, active low).
    pub re_n: bool,
    /// `D[7:0]` output enable.  `1` ⇒ host drives the bus.
    pub d_oe: bool,
    /// `D[7:0]` output value (only meaningful while `d_oe == 1`).
    pub d_out: Bits<8>,
    /// Latched byte from the most-recent ReadData cycle.
    pub data_out: Bits<8>,
    pub busy: bool,
    pub done: bool,
    /// Passthrough of `r_b_n_in` to the host.
    pub r_b_n: bool,
}

impl<const T_W: usize> SynchronousIO for NandFlashAsync<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = nand_flash_async<T_W>;
}

#[kernel]
/// Kernel for [NandFlashAsync].
pub fn nand_flash_async<const T_W: usize>(cr: ClockReset, i: In, q: Q<T_W>) -> (Out, D<T_W>)
where
    rhdl::bits::W<T_W>: BitWidth,
{
    let one_t: Bits<T_W> = bits::<T_W>(1);
    let zero_t: Bits<T_W> = bits::<T_W>(0);

    let t = q.timings;

    let mut d = D::<T_W>::dont_care();
    d.state = q.state;
    d.tick = q.tick + one_t;
    d.op_reg = q.op_reg;
    d.data_reg = q.data_reg;
    d.data_out_reg = q.data_out_reg;
    d.done_pulse = false;

    match q.state {
        NandState::Idle => {
            d.tick = zero_t;
            if i.start {
                d.op_reg = i.op;
                d.data_reg = i.data;
                d.tick = zero_t;
                match i.op {
                    NandOp::ReadData => {
                        d.state = NandState::ReadLow;
                    }
                    NandOp::SendCommand | NandOp::SendAddress | NandOp::SendData => {
                        d.state = NandState::SetupWrite;
                    }
                }
            }
        }
        NandState::SetupWrite => {
            if q.tick == t.t_setup {
                d.state = NandState::WeLow;
                d.tick = zero_t;
            }
        }
        NandState::WeLow => {
            if q.tick == t.t_we_low {
                d.state = NandState::Stop;
                d.tick = zero_t;
            }
        }
        NandState::ReadLow => {
            if q.tick == t.t_re_low {
                d.state = NandState::ReadSample;
                d.tick = zero_t;
            }
        }
        NandState::ReadSample => {
            // Sample on the very next cycle (t == 0 inside ReadSample),
            // matching tDH after RE_n rising.
            d.data_out_reg = i.d_in;
            d.state = NandState::Stop;
            d.tick = zero_t;
        }
        NandState::Stop => {
            d.done_pulse = true;
            d.state = NandState::Idle;
            d.tick = zero_t;
        }
    }

    if cr.reset.any() {
        d.state = NandState::Idle;
        d.tick = zero_t;
        d.op_reg = NandOp::SendCommand;
        d.data_reg = bits::<8>(0);
        d.data_out_reg = bits::<8>(0);
        d.done_pulse = false;
    }

    let busy = q.state != NandState::Idle;

    // CLE / ALE assertions during write cycles.
    let in_write = match q.state {
        NandState::SetupWrite | NandState::WeLow => true,
        _ => false,
    };
    let cle = in_write && (q.op_reg == NandOp::SendCommand);
    let ale = in_write && (q.op_reg == NandOp::SendAddress);

    let we_n = match q.state {
        NandState::WeLow => false,
        _ => true,
    };
    let re_n = match q.state {
        NandState::ReadLow | NandState::ReadSample => false,
        _ => true,
    };
    // We drive the bus during write cycles; the chip drives it during read.
    let d_oe = in_write;
    let ce_n = false; // always selected in v1; host can gate externally

    let mut o = Out::dont_care();
    o.ce_n = ce_n;
    o.cle = cle;
    o.ale = ale;
    o.we_n = we_n;
    o.re_n = re_n;
    o.d_oe = d_oe;
    o.d_out = q.data_reg;
    o.data_out = q.data_out_reg;
    o.busy = busy;
    o.done = q.done_pulse;
    o.r_b_n = i.r_b_n_in;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn test_timings() -> NandTimings<8> {
        NandTimings {
            t_setup: bits(2),
            t_we_low: bits(4),
            t_re_low: bits(4),
        }
    }

    fn idle_in() -> In {
        In {
            op: NandOp::SendCommand,
            data: bits(0),
            d_in: bits(0),
            start: false,
            r_b_n_in: true,
        }
    }

    #[test]
    fn test_idle_strobes_high() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let stream = std::iter::repeat_n(idle_in(), 16)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().all(|s| s.output.we_n));
        assert!(outputs.iter().all(|s| s.output.re_n));
        assert!(outputs.iter().all(|s| !s.output.cle));
        assert!(outputs.iter().all(|s| !s.output.ale));
        Ok(())
    }

    #[test]
    fn test_send_command_asserts_cle() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: NandOp::SendCommand,
            data: bits(0xFF), // ONFI Reset
            d_in: bits(0),
            start: true,
            r_b_n_in: true,
        }];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // CLE must be high at some point during the cycle, ALE must NEVER be.
        assert!(outputs.iter().any(|s| s.output.cle));
        assert!(outputs.iter().all(|s| !s.output.ale));
        // d_out holds 0xFF when bus is being driven.
        let bad = outputs
            .iter()
            .any(|s| s.output.d_oe && s.output.d_out.raw() != 0xFF);
        assert!(!bad, "d_out must hold 0xFF during the write");
        Ok(())
    }

    #[test]
    fn test_send_address_asserts_ale() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: NandOp::SendAddress,
            data: bits(0x55),
            d_in: bits(0),
            start: true,
            r_b_n_in: true,
        }];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.ale));
        assert!(outputs.iter().all(|s| !s.output.cle));
        Ok(())
    }

    #[test]
    fn test_send_data_neither_cle_nor_ale() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: NandOp::SendData,
            data: bits(0x42),
            d_in: bits(0),
            start: true,
            r_b_n_in: true,
        }];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // While we_n is low, both CLE and ALE must be low (data byte).
        let bad = outputs
            .iter()
            .any(|s| !s.output.we_n && (s.output.cle || s.output.ale));
        assert!(!bad);
        Ok(())
    }

    #[test]
    fn test_read_data_captures_d_in() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: NandOp::ReadData,
            data: bits(0),
            d_in: bits(0xA5),
            start: true,
            r_b_n_in: true,
        }];
        for _ in 0..16 {
            stream_in.push(In {
                op: NandOp::ReadData,
                data: bits(0),
                d_in: bits(0xA5),
                start: false,
                r_b_n_in: true,
            });
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_idx = outputs.iter().position(|s| s.output.done).unwrap();
        assert_eq!(outputs[done_idx].output.data_out.raw(), 0xA5);
        // During read, d_oe must be low (chip drives bus).
        let bad = outputs.iter().any(|s| !s.output.re_n && s.output.d_oe);
        assert!(!bad, "d_oe must be low during read");
        Ok(())
    }

    #[test]
    fn test_r_b_n_passthrough() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let stream_in = vec![
            In {
                op: NandOp::SendCommand,
                data: bits(0),
                d_in: bits(0),
                start: false,
                r_b_n_in: false,
            };
            16
        ];
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let any_high = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.r_b_n);
        assert!(!any_high);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["12312"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_nand_flash_async_hdl_works() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: NandOp::SendCommand,
            data: bits(0xFF),
            d_in: bits(0),
            start: true,
            r_b_n_in: true,
        }];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_nand_flash_async_trace() -> miette::Result<()> {
        let uut = NandFlashAsync::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            op: NandOp::SendCommand,
            data: bits(0xFF),
            d_in: bits(0),
            start: true,
            r_b_n_in: true,
        }];
        for _ in 0..16 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("nand_flash_async");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["d19091b4f03f86aa7aeb3c1d9276826e979f5dcbb5afe7b4327e7941c3e52fd1"];
        let digest = vcd
            .dump_to_file(root.join("nand_flash_async.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = NandFlashAsync::<8>::fsm_descriptor();
        assert_eq!(desc.widget_name, "NandFlashAsync");
        assert_eq!(desc.variants().len(), 6);
        assert_eq!(desc.initial_index(), 0);
    }
}
