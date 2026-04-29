//! MIPI DBI Type B (Intel 8080-series parallel) display driver
//!
//! 8-bit (or 16-bit) parallel data bus plus four control strobes —
//! `/CS`, `D/C#`, `/WR`, `/RD` — exactly as Intel's 8080 family
//! defined the protocol in the late 1970s.  Every modern Sitronix /
//! Ilitek / Solomon Systech display controller (ST7735 / ST7789 /
//! ILI9341 / ILI9488 / SSD1351 / SSD1963 / RA8875, etc.) supports
//! both DBI Type C (4-wire SPI) and DBI Type B (8080 parallel).
//! The parallel mode is faster — one byte per `/WR` pulse vs. eight
//! SPI clocks — at the cost of 8 (or 16) extra data pins.
//!
//! **v1 scope:** 8-bit write-only.  The host strobes `send` with
//! each byte and sets `is_command` to drive `D/C#` low (command) or
//! high (data).  The widget pulses `/WR` low for `t_wr_pulse_low`
//! cycles, then high for `t_wr_pulse_high` cycles.  `/CS` is held
//! low while the byte is in flight; `/RD` is held high (unused).
//! Multi-byte autoincrement, 16-bit-bus mode, the `/RD` read path,
//! and FIFO-buffered pixel streaming are deferred to v2.
//!
//! Composes the [super::super::core::dff::DFF] for state and a
//! tiny Mealy FSM.  No SPI master involved — this is the parallel
//! sibling of [super::mipi_dbi_type_c].
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +---------+MipiDbiTypeB+---------+
     |                                |
B<8> |                                | B<8>
+--->| data                       d_o +--->
bool |                                | bool
+--->| is_command                cs_n +--->
bool |                                | bool
+--->| send                      dc_n +--->
     |                           wr_n +--->
     |                           rd_n +--->
     |                           busy +--->
     |                           done +--->
     +--------------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/mipi_dbi_type_b.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/mipi_dbi_type_b.md")]
//!
//! And the auto-generated FSM diagram for the strobe machine:
#![doc = include_str!("../../doc/mipi_dbi_type_b_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the DBI-B strobe FSM.
///
/// Required by CLAUDE.md §12 rule 14.  Indices match `DbiBState`
/// declaration order (Idle=0, Setup=1, WrLow=2, WrHigh=3).
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    },
    Transition {
        source_index: 1,
        target_index: 2,
    },
    Transition {
        source_index: 2,
        target_index: 3,
    },
    Transition {
        source_index: 3,
        target_index: 0,
    },
];

/// Internal state machine.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum DbiBState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Data + D/C# + /CS set up; /WR still high.
    #[fsm_state(label = "setup")]
    Setup,
    /// /WR pulled low (write strobe).
    #[fsm_state(label = "WR low")]
    WrLow,
    /// /WR pulled back high (data is latched on rising edge).
    #[fsm_state(label = "WR high")]
    WrHigh,
}

/// Strobe-timing parameters in *FPGA cycles*.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct DbiBTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Setup time: cycles between `data`/`D/C#` valid and `/WR` falling.
    pub t_setup: Bits<T_W>,
    /// `/WR` low-pulse width.
    pub t_wr_pulse_low: Bits<T_W>,
    /// `/WR` high-time before next byte may begin.
    pub t_wr_pulse_high: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = DbiBState)]
/// MIPI DBI Type B (8080) display driver.
pub struct MipiDbiTypeB<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<DbiBState>,
    tick: dff::DFF<Bits<T_W>>,
    data_reg: dff::DFF<Bits<8>>,
    dc_n_reg: dff::DFF<bool>,
    done_pulse: dff::DFF<bool>,
    timings: Constant<DbiBTimings<T_W>>,
}

impl<const T_W: usize> MipiDbiTypeB<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Create a DBI-B master with the given strobe timings (FPGA cycles).
    pub fn new(timings: DbiBTimings<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            dc_n_reg: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [MipiDbiTypeB].
pub struct In {
    /// Byte to transmit.  Latched at `send`.
    pub data: Bits<8>,
    /// Whether the byte is a command (true → D/C# low) or data (false → high).
    pub is_command: bool,
    /// Strobe to begin a byte transfer.  Ignored while busy.
    pub send: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [MipiDbiTypeB].
pub struct Out {
    /// 8-bit parallel data bus.
    pub d_o: Bits<8>,
    /// `/CS` (active-low chip select).
    pub cs_n: bool,
    /// D/C# (low = command, high = data).
    pub dc_n: bool,
    /// `/WR` (active-low write strobe).
    pub wr_n: bool,
    /// `/RD` (active-low read strobe — held high in v1).
    pub rd_n: bool,
    pub busy: bool,
    pub done: bool,
}

impl<const T_W: usize> SynchronousIO for MipiDbiTypeB<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = mipi_dbi_type_b<T_W>;
}

#[kernel]
/// Kernel for [MipiDbiTypeB].
pub fn mipi_dbi_type_b<const T_W: usize>(cr: ClockReset, i: In, q: Q<T_W>) -> (Out, D<T_W>)
where
    rhdl::bits::W<T_W>: BitWidth,
{
    let one_t: Bits<T_W> = bits::<T_W>(1);
    let zero_t: Bits<T_W> = bits::<T_W>(0);

    let t = q.timings;

    let mut d = D::<T_W>::dont_care();
    d.state = q.state;
    d.tick = q.tick + one_t;
    d.data_reg = q.data_reg;
    d.dc_n_reg = q.dc_n_reg;
    d.done_pulse = false;

    match q.state {
        DbiBState::Idle => {
            d.tick = zero_t;
            if i.send {
                d.data_reg = i.data;
                d.dc_n_reg = !i.is_command;
                d.state = DbiBState::Setup;
                d.tick = zero_t;
            }
        }
        DbiBState::Setup => {
            if q.tick == t.t_setup {
                d.state = DbiBState::WrLow;
                d.tick = zero_t;
            }
        }
        DbiBState::WrLow => {
            if q.tick == t.t_wr_pulse_low {
                d.state = DbiBState::WrHigh;
                d.tick = zero_t;
            }
        }
        DbiBState::WrHigh => {
            if q.tick == t.t_wr_pulse_high {
                d.done_pulse = true;
                d.state = DbiBState::Idle;
                d.tick = zero_t;
            }
        }
    }

    if cr.reset.any() {
        d.state = DbiBState::Idle;
        d.tick = zero_t;
        d.data_reg = bits::<8>(0);
        d.dc_n_reg = true;
        d.done_pulse = false;
    }

    let busy = q.state != DbiBState::Idle;
    let cs_n = !busy;
    let wr_n = match q.state {
        DbiBState::WrLow => false,
        _ => true,
    };

    let mut o = Out::dont_care();
    o.d_o = q.data_reg;
    o.cs_n = cs_n;
    o.dc_n = q.dc_n_reg;
    o.wr_n = wr_n;
    o.rd_n = true;
    o.busy = busy;
    o.done = q.done_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn test_timings() -> DbiBTimings<8> {
        DbiBTimings {
            t_setup: bits(2),
            t_wr_pulse_low: bits(4),
            t_wr_pulse_high: bits(2),
        }
    }

    fn idle_in() -> In {
        In {
            data: bits(0),
            is_command: false,
            send: false,
        }
    }

    #[test]
    fn test_idle_releases_strobes() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().all(|s| s.output.cs_n));
        assert!(outputs.iter().all(|s| s.output.wr_n));
        assert!(outputs.iter().all(|s| s.output.rd_n));
        Ok(())
    }

    #[test]
    fn test_byte_completes() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            is_command: false,
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        assert!(
            uut.run(stream)
                .synchronous_sample()
                .filter(|s| !s.input.0.reset.any())
                .any(|s| s.output.done)
        );
        Ok(())
    }

    #[test]
    fn test_data_appears_on_bus() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            is_command: false,
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        // While busy, d_o must equal the latched byte (0xA5).
        let bad = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| !s.output.cs_n && s.output.d_o.raw() != 0xA5);
        assert!(!bad, "d_o must hold 0xA5 throughout the strobe");
        Ok(())
    }

    #[test]
    fn test_command_drives_dc_low() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0x29),
            is_command: true,
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let bad = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| !s.output.cs_n && s.output.dc_n);
        assert!(!bad, "D/C# must be low during the command strobe");
        Ok(())
    }

    #[test]
    fn test_wr_pulse_low_then_high() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0x55),
            is_command: false,
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Find a sample where wr_n went low — that's the strobe.
        let any_low = outputs.iter().any(|s| !s.output.wr_n);
        assert!(any_low, "/WR was never asserted");
        // After the busy window ends, /WR must be high again.
        let last = outputs.last().unwrap();
        assert!(last.output.wr_n, "/WR not deasserted at end of transfer");
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8663"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_mipi_dbi_type_b_hdl_works() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            is_command: false,
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_mipi_dbi_type_b_trace() -> miette::Result<()> {
        let uut = MipiDbiTypeB::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            is_command: false,
            send: true,
        }];
        for _ in 0..32 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("mipi_dbi_type_b");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["d3254acec926ef88b93e12e8b0a1c88914cf9ba376ec6872bc85a1ca14c7c314"];
        let digest = vcd
            .dump_to_file(root.join("mipi_dbi_type_b.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = MipiDbiTypeB::<8>::fsm_descriptor();
        assert_eq!(desc.widget_name, "MipiDbiTypeB");
        assert_eq!(desc.variants().len(), 4);
        assert_eq!(desc.initial_index(), 0);
    }
}
