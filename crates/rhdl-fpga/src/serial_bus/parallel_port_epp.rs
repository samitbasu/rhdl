//! IEEE 1284 EPP (Enhanced Parallel Port) master
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
//! **v1 scope:** the 4-cycle EPP transaction set.  IEEE 1284
//! mode-select negotiation, the Compatibility / Nibble / Byte
//! / ECP modes, and the bidirectional `nWAIT` timeout (a misbehaving
//! device that never asserts `nWAIT` will hang the FSM in v1; v2
//! adds a configurable timeout) are deferred.
//!
//! Composes [super::super::core::dff::DFF] for state and a
//! 6-state FSM.  The host wraps the bidirectional `D` bus with
//! `tristate::simple`; the widget exposes `(d_oe, d_out, d_in)`.
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

use crate::core::dff;

/// Author-curated transition graph for the EPP cycle FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → AssertStrobe
    Transition {
        source_index: 1,
        target_index: 2,
    }, // AssertStrobe → WaitForLow
    Transition {
        source_index: 2,
        target_index: 3,
    }, // WaitForLow → ReleaseStrobe
    Transition {
        source_index: 3,
        target_index: 4,
    }, // ReleaseStrobe → WaitForHigh
    Transition {
        source_index: 4,
        target_index: 5,
    }, // WaitForHigh → Stop
    Transition {
        source_index: 5,
        target_index: 0,
    }, // Stop → Idle
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
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = EppState)]
/// IEEE 1284 EPP master.
pub struct ParallelPortEpp {
    state: dff::DFF<EppState>,
    op_reg: dff::DFF<EppOp>,
    data_reg: dff::DFF<Bits<8>>,
    /// Latched read byte (refreshed at end of each read cycle).
    data_out_reg: dff::DFF<Bits<8>>,
    done_pulse: dff::DFF<bool>,
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
    /// Latched byte from the most-recent read cycle.
    pub data_out: Bits<8>,
    pub busy: bool,
    pub done: bool,
}

impl SynchronousIO for ParallelPortEpp {
    type I = In;
    type O = Out;
    type Kernel = parallel_port_epp;
}

#[kernel]
/// Kernel for [ParallelPortEpp].
pub fn parallel_port_epp(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.op_reg = q.op_reg;
    d.data_reg = q.data_reg;
    d.data_out_reg = q.data_out_reg;
    d.done_pulse = false;

    match q.state {
        EppState::Idle => {
            if i.start {
                d.op_reg = i.op;
                d.data_reg = i.data;
                d.state = EppState::AssertStrobe;
            }
        }
        EppState::AssertStrobe => {
            // Single-cycle setup state: data + nWRITE + strobe asserted via
            // combinational outputs below.  Move on to wait for the device.
            d.state = EppState::WaitForLow;
        }
        EppState::WaitForLow => {
            // Wait for nWAIT low (device acknowledges).
            if !i.wait_n_in {
                d.state = EppState::ReleaseStrobe;
            }
        }
        EppState::ReleaseStrobe => {
            // For read cycles, capture the byte the device drove on D.
            let is_read = (q.op_reg == EppOp::AddrRead) || (q.op_reg == EppOp::DataRead);
            if is_read {
                d.data_out_reg = i.d_in;
            }
            d.state = EppState::WaitForHigh;
        }
        EppState::WaitForHigh => {
            // Wait for nWAIT high (device releases).
            if i.wait_n_in {
                d.state = EppState::Stop;
            }
        }
        EppState::Stop => {
            d.done_pulse = true;
            d.state = EppState::Idle;
        }
    }

    if cr.reset.any() {
        d.state = EppState::Idle;
        d.op_reg = EppOp::AddrWrite;
        d.data_reg = bits::<8>(0);
        d.data_out_reg = bits::<8>(0);
        d.done_pulse = false;
    }

    let busy = q.state != EppState::Idle;

    // Determine cycle direction (high-level) from op_reg.
    let is_write = (q.op_reg == EppOp::AddrWrite) || (q.op_reg == EppOp::DataWrite);
    let is_addr = (q.op_reg == EppOp::AddrWrite) || (q.op_reg == EppOp::AddrRead);

    // Strobes are asserted (low) during the AssertStrobe and WaitForLow states.
    // Strobe goes back high during ReleaseStrobe and WaitForHigh.
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
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

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
        // Device pulls nWAIT low.
        for _ in 0..low_hold {
            out.push(In {
                op,
                data: bits(data as u128),
                d_in: bits(d_in as u128),
                start: false,
                wait_n_in: false,
            });
        }
        // Device releases nWAIT.
        for _ in 0..8 {
            out.push(idle_in());
        }
        out
    }

    #[test]
    fn test_idle_strobes_high() -> miette::Result<()> {
        let uut = ParallelPortEpp::default();
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
        let uut = ParallelPortEpp::default();
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        assert!(outputs.iter().any(|s| s.output.done));
        // While addr_stb is low, wr_n must also be low (write direction)
        // and d_oe must be high (host driving bus).
        let bad = outputs.iter().any(|s| {
            !s.output.addr_stb && (s.output.wr_n || !s.output.d_oe)
        });
        assert!(!bad, "address write must drive bus + assert wr_n low");
        Ok(())
    }

    #[test]
    fn test_data_read_captures_d_in() -> miette::Result<()> {
        let uut = ParallelPortEpp::default();
        let stream_in = drive_cycle(EppOp::DataRead, 0, 0xC0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let done_idx = outputs.iter().position(|s| s.output.done).unwrap();
        assert_eq!(outputs[done_idx].output.data_out.raw(), 0xC0);
        // During a read cycle d_oe must be low (device drives bus).
        let bad = outputs.iter().any(|s| !s.output.data_stb && s.output.d_oe);
        assert!(!bad, "during read, d_oe must be low (device drives D)");
        Ok(())
    }

    #[test]
    fn test_addr_vs_data_strobe_selection() -> miette::Result<()> {
        let uut = ParallelPortEpp::default();
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // For an AddrWrite, addr_stb must go low at some point but
        // data_stb must NEVER go low.
        assert!(outputs.iter().any(|s| !s.output.addr_stb));
        assert!(outputs.iter().all(|s| s.output.data_stb));
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = ParallelPortEpp::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["9534"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_parallel_port_epp_hdl_works() -> miette::Result<()> {
        let uut = ParallelPortEpp::default();
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_parallel_port_epp_trace() -> miette::Result<()> {
        let uut = ParallelPortEpp::default();
        let stream_in = drive_cycle(EppOp::AddrWrite, 0x42, 0, 3, 3);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("parallel_port_epp");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["111715c1ef768e708d9d03211b9391fd391f1a8f82548e8be44574f7438a01b4"];
        let digest = vcd
            .dump_to_file(root.join("parallel_port_epp.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = ParallelPortEpp::fsm_descriptor();
        assert_eq!(desc.widget_name, "ParallelPortEpp");
        assert_eq!(desc.variants().len(), 6);
        assert_eq!(desc.initial_index(), 0);
    }
}
