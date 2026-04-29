//! IEEE 1284 / Centronics parallel-port transmitter (output-only v1)
//!
//! The IBM PC parallel port (DB-25 connector) — drove every printer
//! from the early 1980s through ~2010 and survives today on
//! industrial PCs and lab instrumentation (oscilloscopes, plotters,
//! GPIB-to-parallel bridges).  IEEE 1284 (1994) is the modern
//! standard that specifies the original Centronics handshake plus
//! four bidirectional modes (Nibble, Byte, EPP, ECP).
//!
//! **v1 scope:** Centronics *output-only*.  The host strobes
//! `send` with each byte; the widget asserts `D[7:0]`, pulses
//! `STROBE_n` low for `t_strobe` cycles, then waits for the
//! device's `ACK_n` pulse before going idle.  `BUSY` from the
//! device is exposed as a passthrough output so the host can
//! gate further `send` strobes.  The IEEE 1284 negotiation
//! handshake (mode-select), the Nibble reverse channel, EPP, and
//! ECP modes (with the ECP RLE encoder/decoder pipeline and the
//! bidirectional data-bus tristate management) are deferred to v2,
//! v3, v4.
//!
//! Composes [super::super::core::dff::DFF] for the data register
//! and a small 4-state FSM (`Idle / Setup / StrobeLow / WaitAck`).
//! The host wraps `ack_n_in` and `busy_in` with
//! [super::super::cdc::synchronizer_chain::BitSyncChain] before
//! feeding them in (the parallel port's printer is asynchronous to
//! the FPGA clock).
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +------+ParallelPort+--------+
     |                            |
B<8> |                            | B<8>
+--->| data                  d_o  +--->
bool |                            | bool
+--->| send             strobe_n  +--->
bool |                            | bool
+--->| ack_n_in              busy +--->
bool |                            | bool
+--->| busy_in       byte_taken   +--->
     |                       done +--->
     |               busy_passthru+--->
     +----------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/parallel_port_centronics.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/parallel_port_centronics.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/parallel_port_centronics_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the Centronics handshake FSM.
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
pub enum CentronicsState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Data driven on D[7:0]; STROBE_n still high.
    #[fsm_state(label = "setup")]
    Setup,
    /// STROBE_n pulled low (data-valid pulse).
    #[fsm_state(label = "STROBE low")]
    StrobeLow,
    /// STROBE_n back high; waiting for ACK_n pulse from device.
    #[fsm_state(label = "wait ACK")]
    WaitAck,
}

/// Strobe-timing parameters in *FPGA cycles*.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct CentronicsTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Setup time between data valid and STROBE_n falling.
    pub t_setup: Bits<T_W>,
    /// STROBE_n low-pulse width (must be ≥ 0.5 µs per IEEE 1284).
    pub t_strobe_low: Bits<T_W>,
    /// Maximum cycles to wait for ACK_n before timing out (0 = wait forever).
    pub t_ack_timeout: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = CentronicsState)]
/// Centronics / IEEE 1284 parallel-port transmitter.
pub struct ParallelPortCentronics<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<CentronicsState>,
    tick: dff::DFF<Bits<T_W>>,
    data_reg: dff::DFF<Bits<8>>,
    /// Last-cycle ACK_n value (for falling-edge detection).
    ack_prev: dff::DFF<bool>,
    done_pulse: dff::DFF<bool>,
    timings: Constant<CentronicsTimings<T_W>>,
}

impl<const T_W: usize> ParallelPortCentronics<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Construct a Centronics transmitter with the given strobe timings.
    pub fn new(timings: CentronicsTimings<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            ack_prev: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [ParallelPortCentronics].
pub struct In {
    /// Byte to transmit.  Latched at `send`.
    pub data: Bits<8>,
    /// Strobe to begin a byte transfer.  Ignored while busy.
    pub send: bool,
    /// Sampled ACK_n from the device (already in FPGA clock domain).
    pub ack_n_in: bool,
    /// Sampled BUSY from the device.
    pub busy_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [ParallelPortCentronics].
pub struct Out {
    /// 8-bit data bus output.
    pub d_o: Bits<8>,
    /// `STROBE_n` (active-low data-valid strobe).
    pub strobe_n: bool,
    /// True while a byte is in flight (host should not strobe `send`).
    pub busy: bool,
    /// Pulses for one cycle when the device's ACK_n falling edge is seen.
    pub byte_taken: bool,
    /// Pulses for one cycle when the byte cycle finishes (after ACK).
    pub done: bool,
    /// Passthrough of `busy_in` so the host can gate at the system level.
    pub busy_passthru: bool,
}

impl<const T_W: usize> SynchronousIO for ParallelPortCentronics<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = parallel_port_centronics<T_W>;
}

#[kernel]
/// Kernel for [ParallelPortCentronics].
pub fn parallel_port_centronics<const T_W: usize>(
    cr: ClockReset,
    i: In,
    q: Q<T_W>,
) -> (Out, D<T_W>)
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
    d.ack_prev = i.ack_n_in;
    d.done_pulse = false;

    // ACK_n falling edge = device acknowledged.
    let ack_falling = q.ack_prev && !i.ack_n_in;

    match q.state {
        CentronicsState::Idle => {
            d.tick = zero_t;
            if i.send {
                d.data_reg = i.data;
                d.state = CentronicsState::Setup;
                d.tick = zero_t;
            }
        }
        CentronicsState::Setup => {
            if q.tick == t.t_setup {
                d.state = CentronicsState::StrobeLow;
                d.tick = zero_t;
            }
        }
        CentronicsState::StrobeLow => {
            if q.tick == t.t_strobe_low {
                d.state = CentronicsState::WaitAck;
                d.tick = zero_t;
            }
        }
        CentronicsState::WaitAck => {
            // Either ACK falling or timeout brings us back to Idle.
            // t_ack_timeout = 0 means "no timeout, wait forever".
            let timed_out = (t.t_ack_timeout != zero_t) && (q.tick >= t.t_ack_timeout);
            if ack_falling || timed_out {
                d.done_pulse = true;
                d.state = CentronicsState::Idle;
                d.tick = zero_t;
            }
        }
    }

    if cr.reset.any() {
        d.state = CentronicsState::Idle;
        d.tick = zero_t;
        d.data_reg = bits::<8>(0);
        d.ack_prev = true;
        d.done_pulse = false;
    }

    let busy = q.state != CentronicsState::Idle;
    let strobe_n = match q.state {
        CentronicsState::StrobeLow => false,
        _ => true,
    };

    let mut o = Out::dont_care();
    o.d_o = q.data_reg;
    o.strobe_n = strobe_n;
    o.busy = busy;
    o.byte_taken = ack_falling && (q.state == CentronicsState::WaitAck);
    o.done = q.done_pulse;
    o.busy_passthru = i.busy_in;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn test_timings() -> CentronicsTimings<8> {
        CentronicsTimings {
            t_setup: bits(2),
            t_strobe_low: bits(4),
            t_ack_timeout: bits(40),
        }
    }

    fn idle_in() -> In {
        In {
            data: bits(0),
            send: false,
            ack_n_in: true,
            busy_in: false,
        }
    }

    #[test]
    fn test_idle_strobe_high() -> miette::Result<()> {
        let uut = ParallelPortCentronics::<8>::new(test_timings());
        let stream = std::iter::repeat_n(idle_in(), 16)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().all(|s| s.output.strobe_n));
        Ok(())
    }

    #[test]
    fn test_byte_with_ack_completes() -> miette::Result<()> {
        // Send one byte; supply an ACK_n falling edge after STROBE_n returns high.
        let uut = ParallelPortCentronics::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
            ack_n_in: true,
            busy_in: false,
        }];
        // 12 cycles of strobe + setup + waiting (no ACK yet).
        for _ in 0..12 {
            stream_in.push(idle_in());
        }
        // Pulse ACK_n low for 2 cycles.
        for _ in 0..2 {
            stream_in.push(In {
                data: bits(0),
                send: false,
                ack_n_in: false,
                busy_in: false,
            });
        }
        for _ in 0..6 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(
            outputs.iter().any(|s| s.output.done),
            "no done pulse from a byte that the device ACKed"
        );
        // Data must hold 0xA5 while strobe_n is low.
        let bad = outputs
            .iter()
            .any(|s| !s.output.strobe_n && s.output.d_o.raw() != 0xA5);
        assert!(!bad);
        Ok(())
    }

    #[test]
    fn test_byte_without_ack_times_out() -> miette::Result<()> {
        // Same byte but never supply ACK_n falling — should still done via timeout.
        let uut = ParallelPortCentronics::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
            ack_n_in: true,
            busy_in: false,
        }];
        for _ in 0..80 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(
            outputs.iter().any(|s| s.output.done),
            "no done pulse — timeout never fired"
        );
        Ok(())
    }

    #[test]
    fn test_busy_passthrough() -> miette::Result<()> {
        let uut = ParallelPortCentronics::<8>::new(test_timings());
        let stream_in = vec![
            In {
                data: bits(0),
                send: false,
                ack_n_in: true,
                busy_in: true,
            };
            16
        ];
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let all_busy = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .all(|s| s.output.busy_passthru);
        assert!(all_busy);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = ParallelPortCentronics::<8>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["8932"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_parallel_port_centronics_hdl_works() -> miette::Result<()> {
        let uut = ParallelPortCentronics::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
            ack_n_in: true,
            busy_in: false,
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
    fn test_parallel_port_centronics_trace() -> miette::Result<()> {
        let uut = ParallelPortCentronics::<8>::new(test_timings());
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            send: true,
            ack_n_in: true,
            busy_in: false,
        }];
        for _ in 0..12 {
            stream_in.push(idle_in());
        }
        for _ in 0..2 {
            stream_in.push(In {
                data: bits(0),
                send: false,
                ack_n_in: false,
                busy_in: false,
            });
        }
        for _ in 0..8 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("parallel_port_centronics");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["8b0dd8f7241a0623e4f2fe4102722c946310b56a542729a494398298a015caaa"];
        let digest = vcd
            .dump_to_file(root.join("parallel_port_centronics.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = ParallelPortCentronics::<8>::fsm_descriptor();
        assert_eq!(desc.widget_name, "ParallelPortCentronics");
        assert_eq!(desc.variants().len(), 4);
        assert_eq!(desc.initial_index(), 0);
    }
}
