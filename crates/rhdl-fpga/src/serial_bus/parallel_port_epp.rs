//! IEEE 1284 EPP (Enhanced Parallel Port) master, fully featured
//!
//! EPP is IEEE 1284 mode 4 — bidirectional, fast (up to 2 MB/s),
//! interlocked-handshake parallel I/O.  Compared to the legacy
//! Centronics handshake ([super::parallel_port_centronics]) it has
//! four major upgrades:
//!
//! 1. **Bidirectional data bus.**  The host drives `D[7:0]` for
//!    writes; the device drives it for reads.  Wrapped at the pad
//!    with `tristate::simple` (or directly with the FPGA's
//!    bidirectional-IOB primitive).
//! 2. **Two strobes** — `nDATASTB` and `nADDRSTB` — distinguishing
//!    register-address-port writes from register-data-port writes
//!    in the device's address space.
//! 3. **`nWRITE` direction line** — high during read cycles, low
//!    during writes.  The device watches this to decide who drives
//!    the bus.
//! 4. **`nWAIT` device handshake** — fully interlocked: the host
//!    waits for the device to assert `nWAIT` low before completing
//!    the strobe, then for `nWAIT` high before starting the next
//!    cycle.  No fixed-pulse timing.
//!
//! Four cycle types — `AddrWrite`, `AddrRead`, `DataWrite`,
//! `DataRead` — share the same FSM, parameterised by which strobe
//! goes low and which direction `nWRITE` indicates.
//!
//! **All-features-implemented:** all 4 cycle types, the
//! interlocked `nWAIT` handshake, AND a configurable per-cycle
//! `nWAIT` timeout (a misbehaving device that never asserts
//! `nWAIT` doesn't hang the FSM — `timeout` pulses, the cycle
//! aborts, `done` still pulses so the host can advance).
//!
//! Composes [super::super::core::dff::DFF] and
//! [super::super::core::constant::Constant].  The host wraps the
//! bidirectional `D` bus with `tristate::simple`; the widget
//! exposes `(d_oe, d_out, d_in)`.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+ParallelPortEpp+--------+
     |                              |
EPPop|                              | bool
+--->| op                  data_stb +--->
B<8> |                              | bool
+--->| data                addr_stb +--->
B<8> |                              | bool
+--->| d_in                  wr_n   +--->
bool |                              | bool
+--->| start                  d_oe  +--->
bool |                              | B<8>
+--->| wait_n_in              d_out +--->
     |                     data_out +--->
     |                         busy +--->
     |                         done +--->
     |                      timeout +--->
     +------------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/parallel_port_epp.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/parallel_port_epp.md")]
//!
//! And the auto-generated FSM diagram for the cycle FSM:
#![doc = include_str!("../../doc/parallel_port_epp_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the EPP cycle FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition { source_index: 0, target_index: 1 }, // Idle → AssertStrobe
    Transition { source_index: 1, target_index: 2 }, // AssertStrobe → WaitForLow
    Transition { source_index: 2, target_index: 3 }, // WaitForLow → ReleaseStrobe
    Transition { source_index: 2, target_index: 6 }, // WaitForLow → TimeoutAbort
    Transition { source_index: 3, target_index: 4 }, // ReleaseStrobe → WaitForHigh
    Transition { source_index: 4, target_index: 5 }, // WaitForHigh → Stop
    Transition { source_index: 4, target_index: 6 }, // WaitForHigh → TimeoutAbort
    Transition { source_index: 5, target_index: 0 }, // Stop → Idle
    Transition { source_index: 6, target_index: 0 }, // TimeoutAbort → Idle
];

/// Operation to perform on a single EPP cycle.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default)]
pub enum EppOp {
    /// Write `data` with `nADDRSTB` strobed low (address byte to register port).
    #[default]
    AddrWrite,
    /// Read into `data_out` with `nADDRSTB` strobed low (read register address).
    AddrRead,
    /// Write `data` with `nDATASTB` strobed low (data byte to data port).
    DataWrite,
    /// Read into `data_out` with `nDATASTB` strobed low (data byte from data port).
    DataRead,
}

/// EPP cycle FSM state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum EppState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Drive D bus (for writes) + nWRITE; assert appropriate strobe low.
    #[fsm_state(label = "assert strobe")]
    AssertStrobe,
    /// Wait for device to assert nWAIT low (interlocked handshake).
    #[fsm_state(label = "wait nWAIT low")]
    WaitForLow,
    /// Release the strobe (back high); sample D for reads on this cycle's exit.
    #[fsm_state(label = "release strobe")]
    ReleaseStrobe,
    /// Wait for device to release nWAIT (back high).
    #[fsm_state(label = "wait nWAIT high")]
    WaitForHigh,
    /// One-cycle done pulse.
    #[fsm_state(label = "stop")]
    Stop,
    /// `nWAIT` timeout — abort cycle, pulse `timeout` + `done`.
    #[fsm_state(label = "timeout abort")]
    TimeoutAbort,
}

/// Per-cycle nWAIT timeout (in FPGA cycles).  Set to 0 to disable timeout
/// (waits forever — matches the original v1 behavior).
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct EppTimings<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Maximum cycles to wait for any `nWAIT` transition.  0 = disabled.
    pub t_wait_max: Bits<T_W>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = EppState)]
/// IEEE 1284 EPP master, fully featured.
pub struct ParallelPortEpp<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<EppState>,
    /// Per-state tick counter (drives the timeout check in WaitForLow / WaitForHigh).
    tick: dff::DFF<Bits<T_W>>,
    op_reg: dff::DFF<EppOp>,
    data_reg: dff::DFF<Bits<8>>,
    /// Latched read byte (refreshed at end of each successful read cycle).
    data_out_reg: dff::DFF<Bits<8>>,
    done_pulse: dff::DFF<bool>,
    timeout_pulse: dff::DFF<bool>,
    timings: Constant<EppTimings<T_W>>,
}

impl<const T_W: usize> ParallelPortEpp<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Construct an EPP master with the given nWAIT-timeout setting.
    /// `t_wait_max = 0` disables the timeout.
    pub fn new(timings: EppTimings<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            op_reg: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            data_out_reg: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            timeout_pulse: dff::DFF::default(),
            timings: Constant::new(timings),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [ParallelPortEpp].
pub struct In {
    /// Cycle type.
    pub op: EppOp,
    /// Byte to drive (used only for AddrWrite / DataWrite).
    pub data: Bits<8>,
    /// Sampled D[7:0] from the device (used during AddrRead / DataRead).
    pub d_in: Bits<8>,
    /// Strobe to begin the cycle.  Ignored while busy.
    pub start: bool,
    /// Sampled `nWAIT` line (already in the FPGA clock domain).
    pub wait_n_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [ParallelPortEpp].
pub struct Out {
    /// `nDATASTB` (active-low data-strobe).
    pub data_stb: bool,
    /// `nADDRSTB` (active-low address-strobe).
    pub addr_stb: bool,
    /// `nWRITE` (low ⇒ host writes; high ⇒ host reads).
    pub wr_n: bool,
    /// True ⇒ host drives the D bus.
    pub d_oe: bool,
    /// D[7:0] output value (only meaningful while `d_oe == 1`).
    pub d_out: Bits<8>,
    /// Latched byte from the most-recent successful read cycle.
    pub data_out: Bits<8>,
    pub busy: bool,
    /// Pulses when the cycle completes (whether via successful handshake or timeout).
    pub done: bool,
    /// Pulses when `nWAIT` failed to transition within `t_wait_max` cycles.
    pub timeout: bool,
}

impl<const T_W: usize> SynchronousIO for ParallelPortEpp<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = parallel_port_epp<T_W>;
}

#[kernel]
/// Kernel for [ParallelPortEpp].
pub fn parallel_port_epp<const T_W: usize>(
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
    d.op_reg = q.op_reg;
    d.data_reg = q.data_reg;
    d.data_out_reg = q.data_out_reg;
    d.done_pulse = false;
    d.timeout_pulse = false;

    // Timeout check: triggers if t_wait_max != 0 AND tick has reached the limit.
    let timeout_enabled = t.t_wait_max != zero_t;
    let timeout_hit = timeout_enabled && (q.tick >= t.t_wait_max);

    match q.state {
        EppState::Idle => {
            d.tick = zero_t;
            if i.start {
                d.op_reg = i.op;
                d.data_reg = i.data;
                d.state = EppState::AssertStrobe;
                d.tick = zero_t;
            }
        }
        EppState::AssertStrobe => {
            // Single-cycle setup state: data + nWRITE + strobe asserted via
            // combinational outputs below.  Move on to wait for the device.
            d.state = EppState::WaitForLow;
            d.tick = zero_t;
        }
        EppState::WaitForLow => {
            // Wait for nWAIT low (device acknowledges).
            if !i.wait_n_in {
                d.state = EppState::ReleaseStrobe;
                d.tick = zero_t;
            } else if timeout_hit {
                d.state = EppState::TimeoutAbort;
                d.tick = zero_t;
            }
        }
        EppState::ReleaseStrobe => {
            // For read cycles, capture the byte the device drove on D.
            let is_read = (q.op_reg == EppOp::AddrRead) || (q.op_reg == EppOp::DataRead);
            if is_read {
                d.data_out_reg = i.d_in;
            }
            d.state = EppState::WaitForHigh;
            d.tick = zero_t;
        }
        EppState::WaitForHigh => {
            // Wait for nWAIT high (device releases).
            if i.wait_n_in {
                d.state = EppState::Stop;
                d.tick = zero_t;
            } else if timeout_hit {
                d.state = EppState::TimeoutAbort;
                d.tick = zero_t;
            }
        }
        EppState::Stop => {
            d.done_pulse = true;
            d.state = EppState::Idle;
            d.tick = zero_t;
        }
        EppState::TimeoutAbort => {
            d.timeout_pulse = true;
            d.done_pulse = true; // host gets exactly one done per start, regardless
            d.state = EppState::Idle;
            d.tick = zero_t;
        }
    }

    if cr.reset.any() {
        d.state = EppState::Idle;
        d.tick = zero_t;
        d.op_reg = EppOp::AddrWrite;
        d.data_reg = bits::<8>(0);
        d.data_out_reg = bits::<8>(0);
        d.done_pulse = false;
        d.timeout_pulse = false;
    }

    let busy = q.state != EppState::Idle;

    // Determine cycle direction (high-level) from op_reg.
    let is_write = (q.op_reg == EppOp::AddrWrite) || (q.op_reg == EppOp::DataWrite);
    let is_addr = (q.op_reg == EppOp::AddrWrite) || (q.op_reg == EppOp::AddrRead);

    // Strobes are asserted (low) during the AssertStrobe and WaitForLow states.
    let strobe_active = match q.state {
        EppState::AssertStrobe | EppState::WaitForLow => true,
        _ => false,
    };
    let data_stb = !(strobe_active && !is_addr);
    let addr_stb = !(strobe_active && is_addr);
    let wr_n = !is_write;
    // d_oe asserted whenever we're driving the bus during a write cycle.
    let d_oe = is_write && busy;

    let mut o = Out::dont_care();
    o.data_stb = data_stb;
    o.addr_stb = addr_stb;
    o.wr_n = wr_n;
    o.d_oe = d_oe;
    o.d_out = q.data_reg;
    o.data_out = q.data_out_reg;
    o.busy = busy;
    o.done = q.done_pulse;
    o.timeout = q.timeout_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn test_timings() -> EppTimings<8> {
        EppTimings {
            t_wait_max: bits(40),
        }
    }

    fn idle_in() -> In {
        In {
            op: EppOp::AddrWrite,
            data: bits(0),
            d_in: bits(0),
            start: false,
            wait_n_in: true,
        }
    }

    /// Build a stream that issues one EPP cycle with a simulated device:
    /// device asserts nWAIT low after `low_delay` cycles and releases it
    /// after another `low_hold` cycles.
    fn drive_cycle(op: EppOp, data: u8, d_in: u8, low_delay: usize, low_hold: usize) -> Vec<In> {
        let mut out: Vec<In> = vec![In {
            op,
            data: bits(data as u128),
            d_in: bits(d_in as u128),
            start: true,
            wait_n_in: true,
        }];
        for _ in 0..low_delay {
            out.push(In {
                op,
                data: bits(data as u128),
                d_in: bits(d_in as u128),
                start: false,
                wait_n_in: true,
            });
        }
        for _ in 0..low_hold {
            out.push(In {
                op,
                data: bits(data as u128),
                d_in: bits(d_in as u128),
                start: false,
                wait_n_in: false,
            });
        }
        for _ in 0..8 {
            out.push(idle_in());
        }
        out
    }

    #[test]
    fn test_idle_strobes_high() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        let stream = std::iter::repeat_n(idle_in(), 16)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().all(|s| s.output.data_stb));
        assert!(outputs.iter().all(|s| s.output.addr_stb));
        assert!(outputs.iter().all(|s| !s.output.d_oe));
        Ok(())
    }

    #[test]
    fn test_addr_write_completes() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.done));
        assert!(!outputs.iter().any(|s| s.output.timeout));
        let bad = outputs.iter().any(|s| {
            !s.output.addr_stb && (s.output.wr_n || !s.output.d_oe)
        });
        assert!(!bad);
        Ok(())
    }

    #[test]
    fn test_data_read_captures_d_in() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        let stream_in = drive_cycle(EppOp::DataRead, 0, 0xC0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_idx = outputs.iter().position(|s| s.output.done).unwrap();
        assert_eq!(outputs[done_idx].output.data_out.raw(), 0xC0);
        let bad = outputs.iter().any(|s| !s.output.data_stb && s.output.d_oe);
        assert!(!bad);
        Ok(())
    }

    #[test]
    fn test_addr_vs_data_strobe_selection() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| !s.output.addr_stb));
        assert!(outputs.iter().all(|s| s.output.data_stb));
        Ok(())
    }

    /// Timeout corner case: device fails to assert nWAIT low.  EPP must
    /// abort the cycle and pulse `timeout` + `done`.
    #[test]
    fn test_timeout_when_device_never_acks() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        // Device never drives nWAIT low.
        let mut stream_in: Vec<In> = vec![In {
            op: EppOp::AddrWrite,
            data: bits(0x42),
            d_in: bits(0),
            start: true,
            wait_n_in: true,
        }];
        for _ in 0..120 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.timeout));
        assert!(outputs.iter().any(|s| s.output.done));
        Ok(())
    }

    /// Timeout-disabled (t_wait_max = 0): widget waits forever — a stuck
    /// device hangs the FSM (matches the original v1 behavior, opt-in).
    #[test]
    fn test_no_timeout_with_t_wait_max_zero() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(EppTimings { t_wait_max: bits(0) });
        let mut stream_in: Vec<In> = vec![In {
            op: EppOp::AddrWrite,
            data: bits(0x42),
            d_in: bits(0),
            start: true,
            wait_n_in: true,
        }];
        for _ in 0..120 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let any_timeout = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.timeout);
        assert!(!any_timeout, "with t_wait_max=0, timeout must never fire");
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["13455"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_parallel_port_epp_hdl_works() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_parallel_port_epp_trace() -> miette::Result<()> {
        let uut = ParallelPortEpp::<8>::new(test_timings());
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("parallel_port_epp");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["9c703d07f5a176a8de6f86269d5c62ffd75227a2d44f3ecd2ad1aab259e23ad0"];
        let digest = vcd
            .dump_to_file(root.join("parallel_port_epp.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = ParallelPortEpp::<8>::fsm_descriptor();
        assert_eq!(desc.widget_name, "ParallelPortEpp");
        assert_eq!(desc.variants().len(), 7);
        assert_eq!(desc.initial_index(), 0);
    }
}
