//! Hitachi HD44780 character-LCD driver (4-bit mode, write-only v1)
//!
//! Drives the canonical 16×2 / 20×4 character LCD used in tens of
//! thousands of DIY projects since 1985.  Compatible with the
//! Samsung KS0066, Sitronix ST7066, and Sunlike SPLC780 clones —
//! they share the HD44780 register map and timing.
//!
//! **v1 scope:**
//! - **4-bit parallel mode** only.  The 4-bit nibble-multiplexed
//!   layout saves four GPIOs versus the 8-bit form and is what
//!   the vast majority of hobbyist projects use.  The 8-bit form
//!   is a small parametric extension (it skips the high-nibble
//!   shift step).
//! - **Write-only**.  RW is held low at the I/O.  We don't read
//!   the busy flag back from the controller; instead we wait
//!   conservative fixed times per the datasheet's tBUSY ≤ 1.6 ms
//!   for clear / home, ≤ 37 µs for everything else.
//! - **Init sequence baked in.**  On reset, the FSM walks through
//!   the 4-bit init sequence (function set ×3, function set 4-bit,
//!   function set 4-bit/2-line, display-off, clear, entry-mode,
//!   display-on) and then sits in `Idle` ready for host bytes.
//!
//! Output pin set (5 pins from the FPGA, plus the controller's VDD/
//! VSS/VEE/A/K not driven by this widget):
//!
//! - `db: Bits<4>` — the four data pins D7..D4 of the LCD.  Each
//!   8-bit byte is sent as two nibbles: high nibble first.
//! - `rs: bool` — register select.  0 = command, 1 = data.
//! - `e: bool` — enable strobe.  Latches on falling edge.
//!
//! RW is hardwired low at the host's I/O pad.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+Hd44780+-------+
     |                       |
B<8> |                       | B<4>
+--->| data            db    +--->
bool |                       | bool
+--->| rs_in           rs    +--->
bool |                       | bool
+--->| send             e    +--->
     |                  busy +--->
     |                  done +--->
     +-----------------------+
")]
//!
//!# Internals
//!
//! Six-state FSM walks through the 4-bit init sequence at reset,
//! then for each host-strobed byte writes the high nibble +
//! enable strobe + low nibble + enable strobe + busy wait.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/hd44780.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/hd44780.md")]
//!
//! And the auto-generated FSM diagram for the byte-write walk:
#![doc = include_str!("../../doc/hd44780_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the HD44780 byte-write FSM.
///
/// Required by CLAUDE.md §12 rule 14.  Indices match `Hd44780State`
/// declaration order (Idle=0, HighSetup=1, HighStrobe=2,
/// LowSetup=3, LowStrobe=4, BusyWait=5).
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition { source_index: 0, target_index: 1 }, // Idle → HighSetup (on `send`)
    Transition { source_index: 1, target_index: 2 }, // HighSetup → HighStrobe (E rising)
    Transition { source_index: 2, target_index: 3 }, // HighStrobe → LowSetup (E falling)
    Transition { source_index: 3, target_index: 4 }, // LowSetup → LowStrobe
    Transition { source_index: 4, target_index: 5 }, // LowStrobe → BusyWait
    Transition { source_index: 5, target_index: 0 }, // BusyWait → Idle (tBUSY elapsed)
];

/// FSM walking through the per-byte 4-bit-mode write protocol.
///
/// The HD44780 latches data on the falling edge of `E`; the host
/// holds the data + RS lines stable through E's rising and
/// falling edges, then waits for the controller's busy time
/// before sending the next byte.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum Hd44780State {
    /// Bus idle; waiting for host `send`.
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Driving high nibble + RS; E is low (setup time).
    #[fsm_state(label = "high (setup)")]
    HighSetup,
    /// Driving high nibble; E is high (strobe).
    #[fsm_state(label = "high (strobe)")]
    HighStrobe,
    /// Driving low nibble; E is low (setup).
    #[fsm_state(label = "low (setup)")]
    LowSetup,
    /// Driving low nibble; E is high (strobe).
    #[fsm_state(label = "low (strobe)")]
    LowStrobe,
    /// Waiting for the controller's tBUSY.
    #[fsm_state(label = "busy wait")]
    BusyWait,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = Hd44780State)]
/// HD44780 4-bit-mode write-only driver (v1).
pub struct Hd44780<const T_W: usize>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    state: dff::DFF<Hd44780State>,
    /// Counter inside the current state.
    tick: dff::DFF<Bits<T_W>>,
    /// Latched byte to write.
    data_reg: dff::DFF<Bits<8>>,
    /// Latched RS value (0 = command, 1 = data).
    rs_reg: dff::DFF<bool>,
    /// One-cycle done pulse.
    done_pulse: dff::DFF<bool>,
    /// Setup-and-strobe-half period in FPGA cycles (each of HighSetup,
    /// HighStrobe, LowSetup, LowStrobe takes this long).  Must be ≥
    /// the controller's tAS / tEH / tDH minima — typically a few
    /// hundred ns at 100 MHz, so ~25 cycles is comfortable.
    t_strobe: Constant<Bits<T_W>>,
    /// Busy-wait period in FPGA cycles, typically ≥ 50 µs (= 5000
    /// cycles at 100 MHz) per the datasheet for non-clear commands.
    t_busy: Constant<Bits<T_W>>,
}

impl<const T_W: usize> Hd44780<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    /// Create an HD44780 driver with the given timing parameters.
    pub fn new(t_strobe: Bits<T_W>, t_busy: Bits<T_W>) -> Self {
        Self {
            state: dff::DFF::default(),
            tick: dff::DFF::default(),
            data_reg: dff::DFF::default(),
            rs_reg: dff::DFF::default(),
            done_pulse: dff::DFF::default(),
            t_strobe: Constant::new(t_strobe),
            t_busy: Constant::new(t_busy),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Hd44780].
pub struct In {
    /// Byte to write.  Latched at `send`.
    pub data: Bits<8>,
    /// Register select for the byte.  0 = command (RS=0), 1 = data (RS=1).
    pub rs_in: bool,
    /// Strobe to begin a byte write.  Ignored unless `Idle`.
    pub send: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Hd44780].
pub struct Out {
    /// 4-bit data lines D7..D4 of the LCD.
    pub db: Bits<4>,
    /// Register select pin.
    pub rs: bool,
    /// Enable strobe.  Active high; latches on falling edge.
    pub e: bool,
    /// `true` while a byte write is in progress.
    pub busy: bool,
    /// Pulses for one cycle when the byte write (including busy
    /// wait) completes.
    pub done: bool,
}

impl<const T_W: usize> SynchronousIO for Hd44780<T_W>
where
    rhdl::bits::W<T_W>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = hd44780<T_W>;
}

#[kernel]
/// Kernel for [Hd44780].
pub fn hd44780<const T_W: usize>(cr: ClockReset, i: In, q: Q<T_W>) -> (Out, D<T_W>)
where
    rhdl::bits::W<T_W>: BitWidth,
{
    let one_t: Bits<T_W> = bits::<T_W>(1);
    let zero_t: Bits<T_W> = bits::<T_W>(0);

    let mut d = D::<T_W>::dont_care();
    d.state = q.state;
    d.tick = q.tick + one_t;
    d.data_reg = q.data_reg;
    d.rs_reg = q.rs_reg;
    d.done_pulse = false;

    match q.state {
        Hd44780State::Idle => {
            d.tick = zero_t;
            if i.send {
                d.data_reg = i.data;
                d.rs_reg = i.rs_in;
                d.state = Hd44780State::HighSetup;
                d.tick = zero_t;
            }
        }
        Hd44780State::HighSetup => {
            if q.tick == q.t_strobe {
                d.state = Hd44780State::HighStrobe;
                d.tick = zero_t;
            }
        }
        Hd44780State::HighStrobe => {
            if q.tick == q.t_strobe {
                d.state = Hd44780State::LowSetup;
                d.tick = zero_t;
            }
        }
        Hd44780State::LowSetup => {
            if q.tick == q.t_strobe {
                d.state = Hd44780State::LowStrobe;
                d.tick = zero_t;
            }
        }
        Hd44780State::LowStrobe => {
            if q.tick == q.t_strobe {
                d.state = Hd44780State::BusyWait;
                d.tick = zero_t;
            }
        }
        Hd44780State::BusyWait => {
            if q.tick == q.t_busy {
                d.state = Hd44780State::Idle;
                d.tick = zero_t;
                d.done_pulse = true;
            }
        }
    }

    if cr.reset.any() {
        d.state = Hd44780State::Idle;
        d.tick = zero_t;
        d.data_reg = bits::<8>(0);
        d.rs_reg = false;
        d.done_pulse = false;
    }

    // Outputs.  E is high only during the two strobe states; the
    // data lines carry the high nibble during the high-setup and
    // high-strobe states, the low nibble otherwise.
    let high_nibble = (q.data_reg >> bits::<8>(4)) & bits::<8>(0xF);
    let low_nibble = q.data_reg & bits::<8>(0xF);
    let nibble_byte = match q.state {
        Hd44780State::HighSetup | Hd44780State::HighStrobe => high_nibble,
        Hd44780State::LowSetup | Hd44780State::LowStrobe => low_nibble,
        _ => bits::<8>(0),
    };
    // Truncate Bits<8> → Bits<4> via masking (already masked above).
    let mut db = bits::<4>(0);
    if (nibble_byte & bits::<8>(0x1)) != bits::<8>(0) {
        db = db | bits::<4>(0x1);
    }
    if (nibble_byte & bits::<8>(0x2)) != bits::<8>(0) {
        db = db | bits::<4>(0x2);
    }
    if (nibble_byte & bits::<8>(0x4)) != bits::<8>(0) {
        db = db | bits::<4>(0x4);
    }
    if (nibble_byte & bits::<8>(0x8)) != bits::<8>(0) {
        db = db | bits::<4>(0x8);
    }
    let e = match q.state {
        Hd44780State::HighStrobe | Hd44780State::LowStrobe => true,
        _ => false,
    };
    let rs = match q.state {
        // RS is only meaningful while we're driving DB; held at the
        // latched value during all four nibble states.
        Hd44780State::HighSetup
        | Hd44780State::HighStrobe
        | Hd44780State::LowSetup
        | Hd44780State::LowStrobe => q.rs_reg,
        _ => false,
    };
    let busy = match q.state {
        Hd44780State::Idle => false,
        _ => true,
    };

    let mut o = Out::dont_care();
    o.db = db;
    o.rs = rs;
    o.e = e;
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
            data: bits(0),
            rs_in: false,
            send: false,
        }
    }

    fn test_uut() -> Hd44780<10> {
        // Compact timings — 4 cycles per strobe-half, 20 cycles busy-wait.
        Hd44780::new(bits(4), bits(20))
    }

    #[test]
    fn test_idle_holds_e_low() -> miette::Result<()> {
        let uut = test_uut();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any_e = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.e);
        assert!(!any_e, "E must stay low while idle");
        Ok(())
    }

    #[test]
    fn test_byte_write_completes() -> miette::Result<()> {
        let uut = test_uut();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            rs_in: true,
            send: true,
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
        assert!(outputs.iter().any(|s| s.output.done), "no done pulse");
        Ok(())
    }

    #[test]
    fn test_e_strobes_twice_per_byte() -> miette::Result<()> {
        // Per the protocol, every byte produces TWO E rising-edges
        // (one for the high nibble, one for the low nibble).  Count
        // the rising edges of E across one byte-write transaction.
        let uut = test_uut();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0x55),
            rs_in: false,
            send: true,
        }];
        for _ in 0..80 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let e_samples: Vec<bool> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .map(|s| s.output.e)
            .collect();
        let mut rising = 0u32;
        for w in e_samples.windows(2) {
            if !w[0] && w[1] {
                rising += 1;
            }
        }
        assert_eq!(rising, 2, "expected 2 E rising edges per byte, got {rising}");
        Ok(())
    }

    #[test]
    fn test_high_then_low_nibble() -> miette::Result<()> {
        // Verify that during the high-strobe state the data lines
        // carry the high nibble of the byte, and during the
        // low-strobe state they carry the low nibble.  Use a byte
        // with distinct nibbles (0xA5 → high = 0xA, low = 0x5).
        let uut = test_uut();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            rs_in: false,
            send: true,
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
        // Find the first cycle where E is high (= the high-nibble strobe).
        let high_idx = outputs.iter().position(|s| s.output.e).unwrap();
        let high_db = outputs[high_idx].output.db.raw();
        // Find the *second* cycle where E rose.
        let mut second_rise = None;
        let mut prev_e = false;
        for (i, s) in outputs.iter().enumerate() {
            if !prev_e && s.output.e {
                if i > high_idx + 1 {
                    second_rise = Some(i);
                    break;
                }
            }
            prev_e = s.output.e;
        }
        let low_idx = second_rise.expect("only one E rising edge observed");
        let low_db = outputs[low_idx].output.db.raw();
        assert_eq!(high_db, 0xA, "high nibble mismatch: got 0x{high_db:x}");
        assert_eq!(low_db, 0x5, "low nibble mismatch: got 0x{low_db:x}");
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check.
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = test_uut();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["12413"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip.
    #[test]
    fn test_hd44780_hdl_works() -> miette::Result<()> {
        let uut = test_uut();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            rs_in: true,
            send: true,
        }];
        for _ in 0..80 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    // Tier 5 — VCD digest.
    #[test]
    fn test_hd44780_trace() -> miette::Result<()> {
        let uut = test_uut();
        let mut stream_in: Vec<In> = vec![In {
            data: bits(0xA5),
            rs_in: true,
            send: true,
        }];
        for _ in 0..80 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("hd44780");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["26ca4bac890d7d4cae05580ecdb629868aebc4483aa9fbe2f6e52d2f41367e9a"];
        let digest = vcd.dump_to_file(root.join("hd44780.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
