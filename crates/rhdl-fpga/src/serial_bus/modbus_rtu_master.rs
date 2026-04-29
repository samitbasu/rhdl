//! Modbus RTU master (FC 0x03 read-holding-registers, v1)
//!
//! Modbus is Modicon's 1979 industrial-control protocol — the
//! single most-installed fieldbus on Earth.  Every PLC, HVAC
//! supervisor, solar inverter, water-treatment SCADA, and
//! factory-automation cell speaks it.  Modbus RTU is the binary
//! over-serial framing — typically 8N1 over RS-485 — and is the
//! ~80 % of installed-base case (the others being Modbus ASCII
//! and Modbus TCP).
//!
//! **v1 scope:** Modbus RTU master, **function code 0x03** only
//! (Read Holding Registers).  The host strobes `start` with a
//! `(slave_addr, start_register, register_count)` request; the
//! widget assembles the 6-byte payload, computes the Modbus CRC
//! (polynomial `0xA001`, init `0xFFFF`, reflected), appends the
//! 2-byte CRC (low byte first, per the spec), and exposes the
//! 8-byte frame on `tx_byte` / `tx_valid` for the host's UART
//! transmitter.  The response-side decoder (parse byte_count +
//! N×2 register bytes + CRC), function codes 0x01 / 0x02 / 0x04 /
//! 0x05 / 0x06 / 0x0F / 0x10, exception responses, the symmetric
//! Modbus slave, the ASCII variant, and Modbus TCP are all v2+.
//!
//! Composes [super::super::core::dff::DFF] for state and counters.
//! The host wires `tx_byte` into a UART TX (e.g., `core::uart::Uart`
//! or `serial_bus::rs485_master::Rs485Master`) and pulses
//! `tx_ready` when the UART has consumed the previous byte.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +-------+ModbusRtuMaster+-------+
     |                               |
B<8> |                               | B<8>
+--->| slave_addr            tx_byte +--->
B<16>|                               | bool
+--->| start_addr           tx_valid +--->
B<16>|                               | bool
+--->| num_regs                 busy +--->
bool |                               | bool
+--->| start                    done +--->
bool |                               |
+--->| tx_ready                      |
     +-------------------------------+
")]
//!
//!# Internals
//!
//! Three-state FSM:
//!
//! - `Idle` — latch the request on `start`.
//! - `Crc` — iteratively compute the CRC over the 6 payload
//!   bytes, one *bit* per cycle (48 bits = 48 cycles).  At each
//!   byte boundary the next payload byte is XOR'd into the low
//!   byte of the running CRC; then 8 conditional shift-and-XOR
//!   passes per the Modbus polynomial `0xA001`.
//! - `Send` — emit the 8-byte frame on `tx_byte`, advancing one
//!   byte per `tx_ready` strobe.  Frame layout:
//!   `slave / 0x03 / addr_hi / addr_lo / count_hi / count_lo /
//!   crc_lo / crc_hi`.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/modbus_rtu_master.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/modbus_rtu_master.md")]
//!
//! And the auto-generated FSM diagram:
#![doc = include_str!("../../doc/modbus_rtu_master_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use crate::core::dff;

/// Author-curated transition graph for the Modbus RTU master FSM.
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
        target_index: 0,
    },
];

/// Internal state machine.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum ModbusState {
    #[default]
    #[fsm_state(label = "idle")]
    Idle,
    /// Iteratively shifting bits through the CRC.
    #[fsm_state(label = "compute CRC")]
    Crc,
    /// Emitting the 8-byte frame, one byte per tx_ready.
    #[fsm_state(label = "send")]
    Send,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = ModbusState)]
/// Modbus RTU master (FC 0x03 only, v1).
pub struct ModbusRtuMaster {
    state: dff::DFF<ModbusState>,
    /// Latched request fields.
    slave: dff::DFF<Bits<8>>,
    addr_hi: dff::DFF<Bits<8>>,
    addr_lo: dff::DFF<Bits<8>>,
    count_hi: dff::DFF<Bits<8>>,
    count_lo: dff::DFF<Bits<8>>,
    /// Running CRC during Crc state; final CRC during Send.
    crc: dff::DFF<Bits<16>>,
    /// Byte index 0..6 during Crc, 0..8 during Send.
    byte_idx: dff::DFF<Bits<4>>,
    /// Bit index 0..8 inside the current byte during Crc.
    bit_idx: dff::DFF<Bits<4>>,
    done_pulse: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [ModbusRtuMaster].
pub struct In {
    /// Modbus slave address (1..247 per spec; 0 = broadcast).
    pub slave_addr: Bits<8>,
    /// Starting register address (big-endian on the wire).
    pub start_addr: Bits<16>,
    /// Number of registers to read (big-endian on the wire).
    pub num_regs: Bits<16>,
    /// Strobe to begin assembling and transmitting a request.  Ignored while busy.
    pub start: bool,
    /// One-cycle pulse from the downstream UART: "I consumed the previous
    /// `tx_byte`; advance to the next."
    pub tx_ready: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [ModbusRtuMaster].
pub struct Out {
    /// Current byte to transmit (only meaningful while `tx_valid == true`).
    pub tx_byte: Bits<8>,
    /// `true` while a fresh `tx_byte` is available for the UART to consume.
    pub tx_valid: bool,
    /// True while the master is assembling, computing CRC, or transmitting.
    pub busy: bool,
    /// Pulses for one cycle when the 8-byte frame has finished transmission.
    pub done: bool,
}

impl SynchronousIO for ModbusRtuMaster {
    type I = In;
    type O = Out;
    type Kernel = modbus_rtu_master;
}

#[kernel]
/// Kernel for [ModbusRtuMaster].
pub fn modbus_rtu_master(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.state = q.state;
    d.slave = q.slave;
    d.addr_hi = q.addr_hi;
    d.addr_lo = q.addr_lo;
    d.count_hi = q.count_hi;
    d.count_lo = q.count_lo;
    d.crc = q.crc;
    d.byte_idx = q.byte_idx;
    d.bit_idx = q.bit_idx;
    d.done_pulse = false;

    // Default tx output (only valid while in Send).
    let mut tx_byte: Bits<8> = bits::<8>(0);
    let mut tx_valid: bool = false;

    // Helper: select payload byte by index.
    let crc_lo: Bits<8> = (q.crc & bits::<16>(0xFF)).resize();
    let crc_hi: Bits<8> = (q.crc >> bits::<16>(8)).resize();
    let func: Bits<8> = bits::<8>(0x03);

    match q.state {
        ModbusState::Idle => {
            if i.start {
                d.slave = i.slave_addr;
                d.addr_hi = (i.start_addr >> bits::<16>(8)).resize();
                d.addr_lo = (i.start_addr & bits::<16>(0xFF)).resize();
                d.count_hi = (i.num_regs >> bits::<16>(8)).resize();
                d.count_lo = (i.num_regs & bits::<16>(0xFF)).resize();
                d.crc = bits::<16>(0xFFFF);
                d.byte_idx = bits::<4>(0);
                d.bit_idx = bits::<4>(0);
                d.state = ModbusState::Crc;
            }
        }
        ModbusState::Crc => {
            // Pick the current payload byte (depends on byte_idx).
            // We use d.* (the *next* values) for the latched fields because
            // they hold the request data — q.slave and q.* are also valid
            // (latched on the previous cycle); use q.* for clarity.
            let cur_byte: Bits<8> = if q.byte_idx == bits::<4>(0) {
                q.slave
            } else if q.byte_idx == bits::<4>(1) {
                func
            } else if q.byte_idx == bits::<4>(2) {
                q.addr_hi
            } else if q.byte_idx == bits::<4>(3) {
                q.addr_lo
            } else if q.byte_idx == bits::<4>(4) {
                q.count_hi
            } else {
                q.count_lo
            };

            // At the very first bit of a byte (bit_idx == 0), XOR the byte
            // into the low half of the CRC.
            let mut work: Bits<16> = q.crc;
            if q.bit_idx == bits::<4>(0) {
                let cur_byte_w: Bits<16> = cur_byte.resize();
                work = q.crc ^ cur_byte_w;
            }
            // Shift right one bit; conditional XOR with 0xA001.
            let lsb_set = (work & bits::<16>(1)) != bits::<16>(0);
            let shifted: Bits<16> = work >> bits::<16>(1);
            let next_crc: Bits<16> = if lsb_set {
                shifted ^ bits::<16>(0xA001)
            } else {
                shifted
            };
            d.crc = next_crc;

            if q.bit_idx == bits::<4>(7) {
                // Last bit of this byte; advance.
                d.bit_idx = bits::<4>(0);
                if q.byte_idx == bits::<4>(5) {
                    // Last byte's last bit — CRC is complete; transition to Send.
                    d.byte_idx = bits::<4>(0);
                    d.state = ModbusState::Send;
                } else {
                    d.byte_idx = q.byte_idx + bits::<4>(1);
                }
            } else {
                d.bit_idx = q.bit_idx + bits::<4>(1);
            }
        }
        ModbusState::Send => {
            // Drive tx_byte from byte_idx (0..7).
            tx_byte = if q.byte_idx == bits::<4>(0) {
                q.slave
            } else if q.byte_idx == bits::<4>(1) {
                func
            } else if q.byte_idx == bits::<4>(2) {
                q.addr_hi
            } else if q.byte_idx == bits::<4>(3) {
                q.addr_lo
            } else if q.byte_idx == bits::<4>(4) {
                q.count_hi
            } else if q.byte_idx == bits::<4>(5) {
                q.count_lo
            } else if q.byte_idx == bits::<4>(6) {
                crc_lo
            } else {
                crc_hi
            };
            tx_valid = true;
            if i.tx_ready {
                if q.byte_idx == bits::<4>(7) {
                    d.done_pulse = true;
                    d.byte_idx = bits::<4>(0);
                    d.state = ModbusState::Idle;
                } else {
                    d.byte_idx = q.byte_idx + bits::<4>(1);
                }
            }
        }
    }

    if cr.reset.any() {
        d.state = ModbusState::Idle;
        d.slave = bits::<8>(0);
        d.addr_hi = bits::<8>(0);
        d.addr_lo = bits::<8>(0);
        d.count_hi = bits::<8>(0);
        d.count_lo = bits::<8>(0);
        d.crc = bits::<16>(0xFFFF);
        d.byte_idx = bits::<4>(0);
        d.bit_idx = bits::<4>(0);
        d.done_pulse = false;
    }

    let mut o = Out::dont_care();
    o.tx_byte = tx_byte;
    o.tx_valid = tx_valid;
    o.busy = q.state != ModbusState::Idle;
    o.done = q.done_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    /// Reference Modbus CRC-16 (polynomial 0xA001) — used to cross-check
    /// the kernel's iterative computation.
    fn ref_crc(payload: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &b in payload {
            crc ^= b as u16;
            for _ in 0..8 {
                if crc & 1 == 1 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }

    fn idle_in() -> In {
        In {
            slave_addr: bits(0),
            start_addr: bits(0),
            num_regs: bits(0),
            start: false,
            tx_ready: false,
        }
    }

    /// Drive the widget through one full request: load on cycle 0, then
    /// wait long enough for CRC, then strobe `tx_ready` 8 times to drain
    /// the frame.  Returns the captured `tx_byte` sequence.
    fn drive_request(slave: u8, start: u16, count: u16) -> (Vec<In>, Vec<u8>) {
        let mut stream_in: Vec<In> = vec![In {
            slave_addr: bits(slave as u128),
            start_addr: bits(start as u128),
            num_regs: bits(count as u128),
            start: true,
            tx_ready: false,
        }];
        // 60 cycles of waiting (covers the 48-cycle CRC + slack).
        for _ in 0..60 {
            stream_in.push(idle_in());
        }
        // 8 tx_ready pulses with idle gaps.
        for _ in 0..8 {
            stream_in.push(In {
                slave_addr: bits(0),
                start_addr: bits(0),
                num_regs: bits(0),
                start: false,
                tx_ready: true,
            });
            stream_in.push(idle_in());
        }
        for _ in 0..6 {
            stream_in.push(idle_in());
        }
        let payload = vec![
            slave,
            0x03,
            (start >> 8) as u8,
            (start & 0xFF) as u8,
            (count >> 8) as u8,
            (count & 0xFF) as u8,
        ];
        let crc = ref_crc(&payload);
        let mut expected = payload.clone();
        expected.push((crc & 0xFF) as u8);
        expected.push((crc >> 8) as u8);
        (stream_in, expected)
    }

    #[test]
    fn test_idle_no_tx_valid() -> miette::Result<()> {
        let uut = ModbusRtuMaster::default();
        let stream = std::iter::repeat_n(idle_in(), 32)
            .with_reset(1)
            .clock_pos_edge(100);
        let any = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.tx_valid);
        assert!(!any);
        Ok(())
    }

    /// Cross-check the kernel's CRC against a Rust reference implementation.
    /// Read 5 holding regs from slave 1 starting at addr 0 — full frame is
    /// `[01, 03, 00, 00, 00, 05, crc_lo, crc_hi]`.  The reference Rust
    /// implementation `ref_crc` defines the expected CRC; the kernel must
    /// agree with it byte-for-byte.
    #[test]
    fn test_kernel_crc_matches_reference() -> miette::Result<()> {
        let (stream_in, expected) = drive_request(0x01, 0x0000, 0x0005);
        let uut = ModbusRtuMaster::default();
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        // Capture tx_byte at every cycle where tx_ready is asserted (the
        // byte the host consumed).
        let captured: Vec<u8> = outputs
            .iter()
            .filter(|s| s.output.tx_valid && s.input.1.tx_ready)
            .map(|s| s.output.tx_byte.raw() as u8)
            .collect();
        assert_eq!(
            captured, expected,
            "frame mismatch: got {captured:02x?}, expected {expected:02x?}"
        );
        Ok(())
    }

    /// A second round-trip with different parameters confirms the CRC engine
    /// works for arbitrary inputs, not just the spec example.
    #[test]
    fn test_arbitrary_request_crc() -> miette::Result<()> {
        let (stream_in, expected) = drive_request(0x11, 0x0042, 0x000A);
        let uut = ModbusRtuMaster::default();
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let captured: Vec<u8> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.tx_valid && s.input.1.tx_ready)
            .map(|s| s.output.tx_byte.raw() as u8)
            .collect();
        assert_eq!(captured, expected);
        Ok(())
    }

    #[test]
    fn test_done_pulses_after_8th_byte() -> miette::Result<()> {
        let (stream_in, _) = drive_request(0x01, 0x0000, 0x0005);
        let uut = ModbusRtuMaster::default();
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let done_count = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .filter(|s| s.output.done)
            .count();
        assert_eq!(done_count, 1);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = ModbusRtuMaster::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["18379"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_modbus_rtu_master_hdl_works() -> miette::Result<()> {
        let (stream_in, _) = drive_request(0x01, 0x0000, 0x0005);
        let uut = ModbusRtuMaster::default();
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_modbus_rtu_master_trace() -> miette::Result<()> {
        let (stream_in, _) = drive_request(0x01, 0x0000, 0x0005);
        let uut = ModbusRtuMaster::default();
        let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("modbus_rtu_master");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["39f6b9e9792f248cbdbff8b538c563048e0424de535e21d7b4933940d6dce868"];
        let digest = vcd
            .dump_to_file(root.join("modbus_rtu_master.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = ModbusRtuMaster::fsm_descriptor();
        assert_eq!(desc.widget_name, "ModbusRtuMaster");
        assert_eq!(desc.variants().len(), 3);
        assert_eq!(desc.initial_index(), 0);
    }
}
