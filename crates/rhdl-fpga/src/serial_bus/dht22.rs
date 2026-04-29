//! DHT22 / AM2302 humidity-temperature reader
//!
//! Single-wire popular hobbyist sensor protocol.  The host pulls
//! the line low for >18 ms to wake the sensor, releases, and the
//! sensor responds with an 80 µs / 80 µs ACK pulse pair followed by
//! 40 data bits (humidity ×2, temperature ×2, checksum).  Each
//! data bit is encoded as a fixed ~50 µs low followed by either a
//! short (~26 µs) or long (~70 µs) high.
//!
//! This v1 reads one frame per `start` strobe.  The bit-decision
//! threshold is supplied at construction so the same widget covers
//! DHT22 and DHT11 (different timings) by changing constants.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-----+Dht22Reader+-----+
     |                       |
bool |                       | bool
+--->| start          drv_low+--->
bool |                       | B<16>
+--->| data_in       humidity+--->
     |                       | B<16>
     |            temperature+--->
     |                       | bool
     |                  valid+--->
     |                  error+--->
     |                   busy+--->
     +-----------------------+
")]
//!
//!# Internals (high level)
//!
//! State machine: `Idle → StartLow → StartRelease → AckLow →
//! AckHigh → BitLow → BitHigh → Done`.  A bit-counter counts the
//! 40 data bits; a high-time register accumulates the high-pulse
//! duration of each bit and classifies it against the
//! `bit_threshold` constant.  Timeout in any non-`Idle` state
//! transitions to `Done` with `error` asserted.
//!
//!# Parameters
//!
//! - `CW` — bit width of the timing counter
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/dht22.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/dht22.md")]
//!
//! And the auto-generated FSM diagram for the read transaction:
#![doc = include_str!("../../doc/dht22_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::{constant::Constant, dff};

/// Author-curated transition graph for the DHT22 read FSM.
///
/// Required by CLAUDE.md §12 rule 14.  Indices match `Dht22State`
/// declaration order (Idle=0, StartLow=1, StartReleaseHigh=2,
/// StartReleaseLow=3, AckLow=4, AckHigh=5, BitLow=6, BitHigh=7).
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    }, // Idle → StartLow (on `start`)
    Transition {
        source_index: 1,
        target_index: 2,
    }, // StartLow → StartReleaseHigh
    Transition {
        source_index: 2,
        target_index: 3,
    }, // StartReleaseHigh → StartReleaseLow
    Transition {
        source_index: 3,
        target_index: 4,
    }, // StartReleaseLow → AckLow (sensor pulls low)
    Transition {
        source_index: 4,
        target_index: 5,
    }, // AckLow → AckHigh
    Transition {
        source_index: 5,
        target_index: 6,
    }, // AckHigh → BitLow (first data bit)
    Transition {
        source_index: 6,
        target_index: 7,
    }, // BitLow → BitHigh
    Transition {
        source_index: 7,
        target_index: 6,
    }, // BitHigh → BitLow (next bit)
    Transition {
        source_index: 7,
        target_index: 0,
    }, // BitHigh → Idle (40th bit done)
    Transition {
        source_index: 3,
        target_index: 0,
    }, // StartReleaseLow → Idle (timeout)
    Transition {
        source_index: 4,
        target_index: 0,
    }, // AckLow → Idle (timeout)
    Transition {
        source_index: 5,
        target_index: 0,
    }, // AckHigh → Idle (timeout)
];

/// State of the DHT22 reader.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum Dht22State {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Master pulls line low to begin a read.
    #[fsm_state(label = "start (low)")]
    StartLow,
    /// Line released by master; waiting for it to go high (pull-up wins).
    #[fsm_state(label = "start (release H)")]
    StartReleaseHigh,
    /// Line is high; waiting for sensor to pull it low (ACK begins).
    #[fsm_state(label = "start (release L)")]
    StartReleaseLow,
    /// Sensor's ACK low pulse.
    #[fsm_state(label = "ACK (low)")]
    AckLow,
    /// Sensor's ACK high pulse.
    #[fsm_state(label = "ACK (high)")]
    AckHigh,
    /// Per-bit low phase (~50 µs).
    #[fsm_state(label = "bit (low)")]
    BitLow,
    /// Per-bit high phase whose duration encodes 0 or 1.
    #[fsm_state(label = "bit (high)")]
    BitHigh,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = Dht22State)]
/// DHT22 reader core.
pub struct Dht22Reader<const CW: usize>
where
    rhdl::bits::W<CW>: BitWidth,
{
    state: dff::DFF<Dht22State>,
    timer: dff::DFF<Bits<CW>>,
    bit_counter: dff::DFF<Bits<6>>,
    shift_reg: dff::DFF<Bits<40>>,
    valid_pulse: dff::DFF<bool>,
    error_flag: dff::DFF<bool>,
    /// Cycles to hold the line low at start (>= 18 ms in real units).
    start_low_cycles: Constant<Bits<CW>>,
    /// Threshold high-time in cycles separating "0" and "1" bits.
    bit_threshold: Constant<Bits<CW>>,
    /// Timeout in cycles per state (used for AckLow, AckHigh, BitLow, BitHigh).
    timeout: Constant<Bits<CW>>,
}

impl<const CW: usize> Dht22Reader<CW>
where
    rhdl::bits::W<CW>: BitWidth,
{
    /// Construct with timings (in FPGA clock cycles).
    pub fn new(start_low_cycles: Bits<CW>, bit_threshold: Bits<CW>, timeout: Bits<CW>) -> Self {
        Self {
            state: dff::DFF::default(),
            timer: dff::DFF::default(),
            bit_counter: dff::DFF::default(),
            shift_reg: dff::DFF::default(),
            valid_pulse: dff::DFF::default(),
            error_flag: dff::DFF::default(),
            start_low_cycles: Constant::new(start_low_cycles),
            bit_threshold: Constant::new(bit_threshold),
            timeout: Constant::new(timeout),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Dht22Reader].
pub struct In {
    /// Strobe to start a read.
    pub start: bool,
    /// Sampled value of the data line.
    pub data_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Dht22Reader].
pub struct Out {
    /// `1` ⇒ master pulls line low (open-drain).
    pub drv_low: bool,
    /// The full 40-bit frame: bits 39..24 = humidity, 23..8 = temperature, 7..0 = checksum.
    /// (Splitting Bits<40> → Bits<16> inside the kernel runs into RHDL type inference
    /// edge cases; the host can mask and shift as needed.)
    pub frame: Bits<40>,
    /// Pulses for one cycle when a fresh frame is available.
    pub valid: bool,
    /// High if the last attempt timed out before completing.
    pub error: bool,
    /// High while reading.
    pub busy: bool,
}

impl<const CW: usize> SynchronousIO for Dht22Reader<CW>
where
    rhdl::bits::W<CW>: BitWidth,
{
    type I = In;
    type O = Out;
    type Kernel = dht22<CW>;
}

#[kernel]
/// Kernel for [Dht22Reader].
pub fn dht22<const CW: usize>(cr: ClockReset, i: In, q: Q<CW>) -> (Out, D<CW>)
where
    rhdl::bits::W<CW>: BitWidth,
{
    let one_cw: Bits<CW> = bits::<CW>(1);
    let zero_cw: Bits<CW> = bits::<CW>(0);
    let one_b6: Bits<6> = bits::<6>(1);
    let zero_b6: Bits<6> = bits::<6>(0);
    let forty_b6: Bits<6> = bits::<6>(40);
    let zero_b40: Bits<40> = bits::<40>(0);

    let mut d = D::<CW>::dont_care();
    d.state = q.state;
    d.timer = q.timer;
    d.bit_counter = q.bit_counter;
    d.shift_reg = q.shift_reg;
    d.valid_pulse = false;
    d.error_flag = q.error_flag;

    let timer_inc = q.timer + one_cw;
    let timer_zero = zero_cw;

    let timeout_hit = q.timer >= q.timeout;
    let start_low_done = q.timer >= q.start_low_cycles;

    match q.state {
        Dht22State::Idle => {
            if i.start {
                d.state = Dht22State::StartLow;
                d.timer = timer_zero;
                d.bit_counter = zero_b6;
                d.shift_reg = zero_b40;
                d.error_flag = false;
            }
        }
        Dht22State::StartLow => {
            // Hold drv_low for `start_low_cycles`; then release.
            if start_low_done {
                d.state = Dht22State::StartReleaseHigh;
                d.timer = timer_zero;
            } else {
                d.timer = timer_inc;
            }
        }
        Dht22State::StartReleaseHigh => {
            // Wait for the line to actually go high (pull-up after master release).
            if i.data_in {
                d.state = Dht22State::StartReleaseLow;
                d.timer = timer_zero;
            } else if timeout_hit {
                d.state = Dht22State::Idle;
                d.error_flag = true;
            } else {
                d.timer = timer_inc;
            }
        }
        Dht22State::StartReleaseLow => {
            // Line was high; wait for sensor to pull low (start of ACK).
            if !i.data_in {
                d.state = Dht22State::AckLow;
                d.timer = timer_zero;
            } else if timeout_hit {
                d.state = Dht22State::Idle;
                d.error_flag = true;
            } else {
                d.timer = timer_inc;
            }
        }
        Dht22State::AckLow => {
            // Sensor holding low; wait for high.
            if i.data_in {
                d.state = Dht22State::AckHigh;
                d.timer = timer_zero;
            } else if timeout_hit {
                d.state = Dht22State::Idle;
                d.error_flag = true;
            } else {
                d.timer = timer_inc;
            }
        }
        Dht22State::AckHigh => {
            // Sensor holding high (post-ACK); wait for low (start of first data bit).
            if !i.data_in {
                d.state = Dht22State::BitLow;
                d.timer = timer_zero;
            } else if timeout_hit {
                d.state = Dht22State::Idle;
                d.error_flag = true;
            } else {
                d.timer = timer_inc;
            }
        }
        Dht22State::BitLow => {
            // Per-bit ~50 µs low; wait for rising edge.
            if i.data_in {
                d.state = Dht22State::BitHigh;
                d.timer = timer_zero;
            } else if timeout_hit {
                d.state = Dht22State::Idle;
                d.error_flag = true;
            } else {
                d.timer = timer_inc;
            }
        }
        Dht22State::BitHigh => {
            // Measuring high time; falling edge ends the bit.
            if !i.data_in {
                // Classify: timer > threshold ⇒ 1, else 0.
                let bit_one = q.timer > q.bit_threshold;
                let bit_in: Bits<40> = if bit_one { bits::<40>(1) } else { zero_b40 };
                let next_shift = (q.shift_reg << 1) | bit_in;
                d.shift_reg = next_shift;
                let next_count = q.bit_counter + one_b6;
                d.bit_counter = next_count;
                if next_count == forty_b6 {
                    // Frame complete.
                    d.state = Dht22State::Idle;
                    d.valid_pulse = true;
                } else {
                    d.state = Dht22State::BitLow;
                    d.timer = timer_zero;
                }
            } else if timeout_hit {
                d.state = Dht22State::Idle;
                d.error_flag = true;
            } else {
                d.timer = timer_inc;
            }
        }
    }

    if cr.reset.any() {
        d.state = Dht22State::Idle;
        d.timer = zero_cw;
        d.bit_counter = zero_b6;
        d.shift_reg = zero_b40;
        d.valid_pulse = false;
        d.error_flag = false;
    }

    let drv_low = match q.state {
        Dht22State::StartLow => true,
        _ => false,
    };
    let busy = match q.state {
        Dht22State::Idle => false,
        _ => true,
    };
    let mut o = Out::dont_care();
    o.drv_low = drv_low;
    o.frame = q.shift_reg;
    o.valid = q.valid_pulse;
    o.error = q.error_flag;
    o.busy = busy;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            start: false,
            data_in: true,
        }
    }

    /// Build a sensor-side stimulus: 40 data bits, MSB-first, encoded as
    /// (low_cycles, high_cycles) pairs after the ACK pulses.
    fn build_sensor_stream(frame: u128, t_unit: usize) -> Vec<In> {
        // t_unit is "cycles per microsecond" effectively.  Use t_unit=1 for fast tests.
        let mut out = Vec::new();
        // Idle high before start.
        for _ in 0..2 {
            out.push(In {
                start: false,
                data_in: true,
            });
        }
        // start strobe + master pulls low
        out.push(In {
            start: true,
            data_in: false,
        }); // start cycle, master pulls low (we pretend data_in tracks it)
            // While master holds low (start_low_cycles cycles), data_in is low.
        for _ in 0..(20 * t_unit) {
            out.push(In {
                start: false,
                data_in: false,
            });
        }
        // Master releases.  Wait a few cycles before sensor responds.
        for _ in 0..2 {
            out.push(In {
                start: false,
                data_in: true,
            });
        }
        // Sensor ACK low (~80 µs).
        for _ in 0..(8 * t_unit) {
            out.push(In {
                start: false,
                data_in: false,
            });
        }
        // Sensor ACK high (~80 µs).
        for _ in 0..(8 * t_unit) {
            out.push(In {
                start: false,
                data_in: true,
            });
        }
        // 40 data bits, MSB-first.
        for k in (0..40).rev() {
            let bit = ((frame >> k) & 1) != 0;
            // 50 µs low.
            for _ in 0..(5 * t_unit) {
                out.push(In {
                    start: false,
                    data_in: false,
                });
            }
            // High time depends on bit: short for 0 (~26 µs), long for 1 (~70 µs).
            let high_cycles = if bit { 7 * t_unit } else { 3 * t_unit };
            for _ in 0..high_cycles {
                out.push(In {
                    start: false,
                    data_in: true,
                });
            }
        }
        // Trailing low (sensor pulls low to end last bit).
        for _ in 0..(5 * t_unit) {
            out.push(In {
                start: false,
                data_in: false,
            });
        }
        // Idle.
        for _ in 0..(8 * t_unit) {
            out.push(In {
                start: false,
                data_in: true,
            });
        }
        out
    }

    // Tier 2 — receive a frame and check humidity / temperature parsing.
    #[test]
    fn test_receive_frame() -> miette::Result<()> {
        // Frame layout: humidity (16 bits) << 24 | temperature (16 bits) << 8 | checksum (8 bits)
        let humidity: u128 = 0x1234;
        let temperature: u128 = 0x5678;
        let checksum: u128 = 0xAB;
        let frame: u128 = (humidity << 24) | (temperature << 8) | checksum;
        let t_unit = 1usize;
        // start_low_cycles: 18 (small for test); bit_threshold: 5 (between 3 and 7); timeout: 50.
        let uut = Dht22Reader::<10>::new(bits(18), bits(5), bits(60));
        let stream_in = build_sensor_stream(frame, t_unit);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let valid_idx = outputs.iter().position(|s| s.output.valid);
        assert!(valid_idx.is_some(), "no valid pulse");
        let v = &outputs[valid_idx.unwrap()].output;
        let frame_raw = v.frame.raw();
        let humidity_decoded = (frame_raw >> 24) & 0xFFFF;
        let temperature_decoded = (frame_raw >> 8) & 0xFFFF;
        assert_eq!(humidity_decoded, humidity, "humidity mismatch");
        assert_eq!(temperature_decoded, temperature, "temperature mismatch");
        assert!(!v.error);
        Ok(())
    }

    #[test]
    fn test_idle_no_pulse() -> miette::Result<()> {
        let uut = Dht22Reader::<10>::new(bits(18), bits(5), bits(60));
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.valid || s.output.error || s.output.busy);
        assert!(!any);
        Ok(())
    }

    // Tier 3 — HDL emission length sanity check
    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Dht22Reader::<10>::new(bits(18), bits(5), bits(60));
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["15294"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    // Tier 4 — iverilog round-trip
    #[test]
    fn test_dht22_hdl_works() -> miette::Result<()> {
        let humidity: u128 = 0x1234;
        let temperature: u128 = 0x5678;
        let checksum: u128 = 0xAB;
        let frame = (humidity << 24) | (temperature << 8) | checksum;
        let uut = Dht22Reader::<10>::new(bits(18), bits(5), bits(60));
        let stream_in = build_sensor_stream(frame, 1);
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
    fn test_dht22_trace() -> miette::Result<()> {
        let frame = (0x1234u128 << 24) | (0x5678u128 << 8) | 0xAB;
        let uut = Dht22Reader::<10>::new(bits(18), bits(5), bits(60));
        let stream_in = build_sensor_stream(frame, 1);
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("dht22");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["55d8cdae5dfccc2ff6666be7cc7668283ba605ea6533b36757673289575eb2b0"];
        let digest = vcd.dump_to_file(root.join("dht22.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
