//! SMBus / SBS host (timeout-enforced I²C wrapper)
//!
//! SMBus 2.0 is electrically I²C with three additional discipline
//! rules:
//!
//! 1. **Bus-time timeout (`t_TIMEOUT,MAX` = 35 ms).** A transaction
//!    that doesn't complete in 35 ms is treated as failed; the host
//!    must release the bus and retry.  The slave is allowed to
//!    detect the same timeout and reset its own state machine.
//! 2. **Clock-low timeout (`t_LOW:SEXT` = 25 ms).** A slave that
//!    holds SCL low longer than this has hung; the host should
//!    abort the transaction.
//! 3. **Frequency cap (10–100 kHz).** SMBus is intentionally slow
//!    so that fault detection has time to act.
//!
//! The Smart Battery System (SBS) Specification — used by every
//! laptop and smartphone smart battery — sits on top of SMBus and
//! defines a register set (manufacturer info, temperature, voltage,
//! state of charge, etc.) accessed via SMBus block reads.
//!
//! **v1 scope:** wrap [super::i2c_master::I2cMaster] and add a
//! transaction-level timeout watchdog.  The host strobes `start`,
//! the inner I²C master runs the protocol, and a counter ticks
//! against `t_xact_max`.  If the counter expires first, the widget
//! pulses `timeout` (and `done` for FSM symmetry) and the host
//! re-issues the transaction.  The clock-low timeout, PEC byte,
//! and the SBS-specific block-read protocol are deferred to v2.
//!
//! Composes [super::i2c_master::I2cMaster] for the actual byte
//! transport.  The widget is intentionally a thin shim — most of
//! the design budget for SMBus is at the *protocol layer* (block
//! transfer, PEC, ARP), not the wire layer.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----------+SmbusHost+----------+
     |                                |
B<7> |                                | bool
+--->| addr             scl_drive_low +--->
B<8> |                                | bool
+--->| data             sda_drive_low +--->
bool |                                | bool
+--->| start                      busy+--->
bool |                                | bool
+--->| sda_in                     done+--->
     |                          ack_ok+--->
     |                         timeout+--->
     +--------------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/smbus_host.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/smbus_host.md")]
use rhdl::prelude::*;

use super::i2c_master::{I2cMaster, i2c_master as i2c_master_kernel};
use crate::core::{constant::Constant, dff};

#[allow(unused_imports)]
use i2c_master_kernel as _;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// SMBus / SBS host (timeout-enforced I²C wrapper).
pub struct SmbusHost<const DIV_W: usize, const T_W: usize>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<T_W>: BitWidth,
{
    i2c: I2cMaster<DIV_W>,
    /// Timeout tick counter.
    tick: dff::DFF<Bits<T_W>>,
    /// True while a transaction is in flight (set on start, cleared on done/timeout).
    in_flight: dff::DFF<bool>,
    /// Latched timeout flag (cleared at next start).
    timed_out: dff::DFF<bool>,
    /// One-cycle done pulse to the host.
    done_pulse: dff::DFF<bool>,
    /// Maximum transaction duration in FPGA cycles.
    t_xact_max: Constant<Bits<T_W>>,
}

impl<const DIV_W: usize, const T_W: usize> SmbusHost<DIV_W, T_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Create an SMBus host wrapping an I²C master with the given divisor
    /// and a per-transaction timeout (in FPGA cycles — for SMBus 2.0 the
    /// canonical value is 35 ms × `clock_hz`).
    pub fn new(divisor: Bits<DIV_W>, t_xact_max: Bits<T_W>) -> Self {
        Self {
            i2c: I2cMaster::new(divisor),
            tick: dff::DFF::default(),
            in_flight: dff::DFF::default(),
            timed_out: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            t_xact_max: Constant::new(t_xact_max),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [SmbusHost] (identical shape to [super::i2c_master::In]).
pub struct In {
    pub addr: Bits<7>,
    pub data: Bits<8>,
    pub start: bool,
    pub sda_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [SmbusHost].
pub struct Out {
    pub scl_drive_low: bool,
    pub sda_drive_low: bool,
    pub busy: bool,
    pub done: bool,
    pub ack_ok: bool,
    /// `true` (latched until next `start`) if the most-recent transaction
    /// exceeded `t_xact_max` cycles without producing an I²C `done`.
    pub timeout: bool,
}

impl<const DIV_W: usize, const T_W: usize> SynchronousIO for SmbusHost<DIV_W, T_W>
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = smbus_host<DIV_W, T_W>;
}

#[kernel]
/// Kernel for [SmbusHost].
pub fn smbus_host<const DIV_W: usize, const T_W: usize>(
    cr: ClockReset,
    i: In,
    q: Q<DIV_W, T_W>,
) -> (Out, D<DIV_W, T_W>)
where
    rhdl::bits::W<DIV_W>: BitWidth,
    rhdl::bits::W<T_W>: BitWidth,
{
    let one_t: Bits<T_W> = bits::<T_W>(1);
    let zero_t: Bits<T_W> = bits::<T_W>(0);

    let mut d = D::<DIV_W, T_W>::dont_care();

    // Forward to I²C; suppress new starts if a transaction is already in
    // flight (matches the I²C master's own gating, but explicit here so
    // the timeout counter always agrees with what the inner FSM did).
    d.i2c = super::i2c_master::In {
        addr: i.addr,
        data: i.data,
        start: i.start && !q.in_flight,
        sda_in: i.sda_in,
    };

    let i2c_busy = q.i2c.busy;
    let i2c_done = q.i2c.done;

    d.tick = q.tick;
    d.in_flight = q.in_flight;
    d.timed_out = q.timed_out;
    d.done_pulse = false;

    // Start a new transaction: clear timeout latch, restart counter.
    if i.start && !q.in_flight {
        d.in_flight = true;
        d.tick = zero_t;
        d.timed_out = false;
    } else if q.in_flight {
        d.tick = q.tick + one_t;
    }

    // I²C completed normally: surface the done pulse.
    if i2c_done {
        d.in_flight = false;
        d.done_pulse = true;
    }

    // Timeout: counter reached t_xact_max while still in flight and the
    // I²C master hasn't asserted done this cycle.
    let t_max = q.t_xact_max;
    if q.in_flight && q.tick >= t_max && !i2c_done {
        d.in_flight = false;
        d.timed_out = true;
        d.done_pulse = true;
    }

    if cr.reset.any() {
        d.tick = zero_t;
        d.in_flight = false;
        d.timed_out = false;
        d.done_pulse = false;
    }

    let mut o = Out::dont_care();
    o.scl_drive_low = q.i2c.scl_drive_low;
    o.sda_drive_low = q.i2c.sda_drive_low;
    o.busy = i2c_busy || q.in_flight;
    o.done = q.done_pulse;
    o.ack_ok = q.i2c.ack_ok;
    o.timeout = q.timed_out;
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
            sda_in: true,
        }
    }

    #[test]
    fn test_idle_no_activity() -> miette::Result<()> {
        // divisor=4 → ~12 cycles per I²C bit; t_xact_max=10000 huge enough
        // that idle alone never trips it.
        let uut = SmbusHost::<4, 16>::new(bits(4), bits(10_000));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_busy = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.busy);
        assert!(!any_busy);
        Ok(())
    }

    #[test]
    fn test_normal_transaction_done_no_timeout() -> miette::Result<()> {
        // Generous timeout so the I²C transaction completes well inside it.
        let uut = SmbusHost::<4, 16>::new(bits(4), bits(10_000));
        let mut stream_in: Vec<In> = vec![In {
            addr: bits(0x50),
            data: bits(0xA5),
            start: true,
            sda_in: true,
        }];
        for _ in 0..400 {
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
            "no done pulse from a normally-completing transaction"
        );
        let any_timeout = outputs.iter().any(|s| s.output.timeout);
        assert!(
            !any_timeout,
            "spurious timeout — t_xact_max should have been generous"
        );
        Ok(())
    }

    #[test]
    fn test_timeout_fires_when_slave_hangs() -> miette::Result<()> {
        // Tiny timeout (50 cycles) — well under what an I²C transaction
        // actually takes — so the watchdog fires before done.
        let uut = SmbusHost::<4, 16>::new(bits(4), bits(50));
        let mut stream_in: Vec<In> = vec![In {
            addr: bits(0x50),
            data: bits(0xA5),
            start: true,
            sda_in: true,
        }];
        for _ in 0..400 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let timeout_idx = outputs.iter().position(|s| s.output.timeout);
        assert!(timeout_idx.is_some(), "timeout flag never asserted");
        // Timeout should be sticky: once latched, stays true through the
        // remainder of the trace (until the next start).
        let timeout_idx = timeout_idx.unwrap();
        for s in &outputs[timeout_idx..] {
            assert!(s.output.timeout, "timeout flag dropped before next start");
        }
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = SmbusHost::<4, 16>::new(bits(4), bits(10_000));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["28930"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_smbus_host_hdl_works() -> miette::Result<()> {
        let uut = SmbusHost::<4, 16>::new(bits(4), bits(10_000));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_smbus_host_trace() -> miette::Result<()> {
        let uut = SmbusHost::<4, 16>::new(bits(4), bits(10_000));
        let mut stream_in: Vec<In> = vec![In {
            addr: bits(0x50),
            data: bits(0xA5),
            start: true,
            sda_in: true,
        }];
        for _ in 0..400 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("smbus_host");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["2870db59d6456e375be474ce242f2a16758c465f7134121446c9220463b47210"];
        let digest = vcd.dump_to_file(root.join("smbus_host.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
