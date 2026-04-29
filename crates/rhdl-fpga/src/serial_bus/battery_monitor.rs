//! Battery-management single-register poller (TI HDQ fuel gauge)
//!
//! Periodically reads a single 8-bit register from a TI `bq` series
//! fuel-gauge IC over the HDQ single-wire bus.  This is the
//! smallest useful battery-management widget — exactly what a
//! laptop power-monitor or a smart-charger needs to track
//! state-of-charge.  Higher-level multi-register polling (voltage,
//! current, temperature, full-charge capacity, design capacity,
//! manufacturer info), the SMBus / SBS variant, threshold alarms,
//! and a charger-control state machine all build on this primitive.
//!
//! Each polling cycle issues the canonical 3-step HDQ read
//! sequence on the wire:
//!
//! 1. **Break** — drive the line low for the break-pulse window.
//! 2. **Write address** — transmit the 7-bit register address with
//!    the MSB set to `0` (read-direction).  The 8-bit byte is the
//!    address with bit 7 cleared.
//! 3. **Read data** — receive the 8-bit value.
//!
//! Between polls the widget waits `t_poll_interval` FPGA cycles
//! and re-runs the sequence.  After each successful read,
//! `data_out` is updated with the new byte and `valid` pulses
//! high for one cycle.
//!
//! Composes [super::ti_hdq::TiHdqMaster] for the actual wire
//! protocol — the FSM here only sequences the three primitive ops
//! and counts the inter-poll interval.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +---------+BatteryMonitor+--------+
     |                                 |
B<7> |                                 | bool
+--->| reg_addr                bus_oe  +--->
bool |                                 | bool
+--->| bus_in                 bus_out  +--->
     |                       data_out  +--->
     |                          valid  +--->
     |                           busy  +--->
     +---------------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/battery_monitor.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/battery_monitor.md")]
//!
//! And the auto-generated FSM diagram for the polling sequence:
#![doc = include_str!("../../doc/battery_monitor_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use super::ti_hdq::{TiHdqMaster, TiHdqOp, TiHdqTimings, ti_hdq as ti_hdq_kernel};
use crate::core::{constant::Constant, dff};

#[allow(unused_imports)]
use ti_hdq_kernel as _;

/// Author-curated transition graph for the polling FSM.
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
        target_index: 4,
    },
    Transition {
        source_index: 4,
        target_index: 5,
    },
    Transition {
        source_index: 5,
        target_index: 6,
    },
    Transition {
        source_index: 6,
        target_index: 0,
    },
];

/// Polling FSM states.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum PollState {
    /// Waiting for the next poll tick.
    #[default]
    #[fsm_state(label = "wait")]
    Wait,
    /// Issuing the break pulse.
    #[fsm_state(label = "issue Break")]
    IssueBreak,
    /// Waiting for HDQ master to finish the break.
    #[fsm_state(label = "wait Break")]
    WaitBreak,
    /// Issuing the address byte (WriteByte).
    #[fsm_state(label = "issue Addr")]
    IssueAddr,
    /// Waiting for HDQ master to finish the address byte.
    #[fsm_state(label = "wait Addr")]
    WaitAddr,
    /// Issuing the data read.
    #[fsm_state(label = "issue Read")]
    IssueRead,
    /// Waiting for HDQ master to finish the read; capture data on done.
    #[fsm_state(label = "wait Read")]
    WaitRead,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = PollState)]
/// Battery-management single-register poller.
pub struct BatteryMonitor<const T_W: usize, const I_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
    rhdl::bits::W<I_W>: BitWidth,
{
    hdq: TiHdqMaster<T_W>,
    state: dff::DFF<PollState>,
    /// Inter-poll interval counter.
    poll_tick: dff::DFF<Bits<I_W>>,
    /// Latched register read-back value.
    data_reg: dff::DFF<Bits<8>>,
    /// One-cycle valid pulse when a fresh value lands in `data_reg`.
    valid_pulse: dff::DFF<bool>,
    /// One-cycle pulse to strobe the HDQ master's `start` pin.
    start_pulse: dff::DFF<bool>,
    /// FPGA cycles between polls.
    t_poll_interval: Constant<Bits<I_W>>,
}

impl<const T_W: usize, const I_W: usize> BatteryMonitor<T_W, I_W>
where
    rhdl::bits::W<T_W>: BitWidth,
    rhdl::bits::W<I_W>: BitWidth,
{
    /// Construct a battery monitor.  `hdq_timings` configure the inner
    /// HDQ master; `t_poll_interval` is the inter-poll wait in FPGA cycles.
    pub fn new(hdq_timings: TiHdqTimings<T_W>, t_poll_interval: Bits<I_W>) -> Self {
        Self {
            hdq: TiHdqMaster::new(hdq_timings),
            state: dff::DFF::default(),
            poll_tick: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            valid_pulse: dff::DFF::default(),
            start_pulse: dff::DFF::default(),
            t_poll_interval: Constant::new(t_poll_interval),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [BatteryMonitor].
pub struct In {
    /// 7-bit register address to poll (MSB is forced to 0 for read).
    pub reg_addr: Bits<7>,
    /// Sampled HDQ bus level.
    pub bus_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [BatteryMonitor].
pub struct Out {
    pub bus_oe: bool,
    pub bus_out: bool,
    /// Latched 8-bit register value (refreshed every poll cycle).
    pub data_out: Bits<8>,
    /// Pulses for one cycle when `data_out` is freshly captured.
    pub valid: bool,
    pub busy: bool,
}

impl<const T_W: usize, const I_W: usize> SynchronousIO for BatteryMonitor<T_W, I_W>
where
    rhdl::bits::W<T_W>: BitWidth,
    rhdl::bits::W<I_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = battery_monitor<T_W, I_W>;
}

#[kernel]
/// Kernel for [BatteryMonitor].
pub fn battery_monitor<const T_W: usize, const I_W: usize>(
    cr: ClockReset,
    i: In,
    q: Q<T_W, I_W>,
) -> (Out, D<T_W, I_W>)
where
    rhdl::bits::W<T_W>: BitWidth,
    rhdl::bits::W<I_W>: BitWidth,
{
    let one_i: Bits<I_W> = bits::<I_W>(1);
    let zero_i: Bits<I_W> = bits::<I_W>(0);

    let mut d = D::<T_W, I_W>::dont_care();
    d.state = q.state;
    d.poll_tick = q.poll_tick;
    d.data_reg = q.data_reg;
    d.valid_pulse = false;
    d.start_pulse = false;

    // Default: forward the read-direction address byte (bit 7 = 0) and Break op.
    // The actual op + data are overridden per-state below; HDQ ignores them
    // unless `start` is high.  reg_addr is Bits<7>; zero-extending into Bits<8>
    // automatically gives us bit 7 = 0 (the read-direction flag).
    let addr_byte: Bits<8> = i.reg_addr.resize();
    let mut hdq_op = TiHdqOp::Break;
    let mut hdq_data: Bits<8> = bits::<8>(0);

    match q.state {
        PollState::Wait => {
            d.poll_tick = q.poll_tick + one_i;
            if q.poll_tick >= q.t_poll_interval {
                d.poll_tick = zero_i;
                d.state = PollState::IssueBreak;
            }
        }
        PollState::IssueBreak => {
            hdq_op = TiHdqOp::Break;
            d.start_pulse = true;
            d.state = PollState::WaitBreak;
        }
        PollState::WaitBreak => {
            if q.hdq.done {
                d.state = PollState::IssueAddr;
            }
        }
        PollState::IssueAddr => {
            hdq_op = TiHdqOp::WriteByte;
            hdq_data = addr_byte;
            d.start_pulse = true;
            d.state = PollState::WaitAddr;
        }
        PollState::WaitAddr => {
            if q.hdq.done {
                d.state = PollState::IssueRead;
            }
        }
        PollState::IssueRead => {
            hdq_op = TiHdqOp::ReadByte;
            d.start_pulse = true;
            d.state = PollState::WaitRead;
        }
        PollState::WaitRead => {
            if q.hdq.done {
                d.data_reg = q.hdq.data_out;
                d.valid_pulse = true;
                d.state = PollState::Wait;
            }
        }
    }

    // Drive the HDQ master.  `start` is the 1-cycle pulse register from
    // last cycle (one-cycle delay between Issue* state and HDQ start).
    d.hdq = super::ti_hdq::In {
        op: hdq_op,
        data: hdq_data,
        start: q.start_pulse,
        bus_in: i.bus_in,
    };

    if cr.reset.any() {
        d.state = PollState::Wait;
        d.poll_tick = zero_i;
        d.data_reg = bits::<8>(0);
        d.valid_pulse = false;
        d.start_pulse = false;
    }

    let mut o = Out::dont_care();
    o.bus_oe = q.hdq.bus_oe;
    o.bus_out = q.hdq.bus_out;
    o.data_out = q.data_reg;
    o.valid = q.valid_pulse;
    o.busy = q.state != PollState::Wait;
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
            reg_addr: bits(0x06), // bq fuel-gauge "state of charge" register
            bus_in: true,
        }
    }

    #[test]
    fn test_polling_eventually_completes() -> miette::Result<()> {
        // poll_interval=10 (very short) so multiple cycles fit in the trace.
        let uut = BatteryMonitor::<10, 8>::new(test_timings(), bits(10));
        let stream = std::iter::repeat_n(idle_in(), 4000)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let valids = outputs.iter().filter(|s| s.output.valid).count();
        assert!(valids >= 1, "no valid pulse — polling never completed");
        Ok(())
    }

    #[test]
    fn test_idle_busy_low() -> miette::Result<()> {
        // Long poll interval — first 5 cycles must show busy=false (Wait state).
        let uut = BatteryMonitor::<10, 16>::new(test_timings(), bits(50_000));
        let stream = std::iter::repeat_n(idle_in(), 5)
            .with_reset(1)
            .clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().all(|s| !s.output.busy));
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = BatteryMonitor::<10, 8>::new(test_timings(), bits(10));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["25223"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_battery_monitor_hdl_works() -> miette::Result<()> {
        let uut = BatteryMonitor::<10, 8>::new(test_timings(), bits(10));
        let stream = std::iter::repeat_n(idle_in(), 200)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_battery_monitor_trace() -> miette::Result<()> {
        let uut = BatteryMonitor::<10, 8>::new(test_timings(), bits(10));
        let stream = std::iter::repeat_n(idle_in(), 600)
            .with_reset(1)
            .clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("battery_monitor");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["b8cd307f5bd264a4fa4430aad836fe313e49233024a784626b176b37851662bf"];
        let digest = vcd
            .dump_to_file(root.join("battery_monitor.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = BatteryMonitor::<10, 8>::fsm_descriptor();
        assert_eq!(desc.widget_name, "BatteryMonitor");
        assert_eq!(desc.variants().len(), 7);
        assert_eq!(desc.initial_index(), 0);
    }
}
