//! PS/2 mouse receiver (host side, 3-byte standard packet)
//!
//! Same wire protocol as the PS/2 keyboard ([super::ps2_keyboard])
//! — two open-drain wires (`CLK` and `DATA`), 11-bit framed bytes
//! with odd parity.  The mouse-specific layer is a 3-byte
//! *packet* assembler:
//!
//! ```text
//!   Byte 0:  Y_OVF | X_OVF | Y_SIGN | X_SIGN | 1 | M | R | L
//!   Byte 1:  X delta (8 bits, signed magnitude — sign from byte 0 bit 4)
//!   Byte 2:  Y delta (8 bits, signed magnitude — sign from byte 0 bit 5)
//! ```
//!
//! Bit 3 of byte 0 is **always 1** — the *sync bit*.  If a host
//! ever sees a "byte 0" with sync == 0 it knows it has slipped a
//! frame and should treat that byte as actually being byte 1 or 2.
//! In v1 we use sync == 0 to drop the byte and stay in `Byte0`,
//! re-synchronising on the next frame.
//!
//! **v1 scope:** standard 3-byte packet (no scroll wheel).  The
//! Microsoft IntelliMouse extension (4-byte packet with Z scroll
//! wheel + buttons 4 / 5) is enabled by issuing a magic
//! "Set Sample Rate to 200 / 100 / 80" sequence over the
//! host→mouse direction; that extension and the host-to-mouse
//! transmit path are deferred to v2.
//!
//! Composes [super::ps2_keyboard::Ps2Keyboard] for the byte-level
//! receiver — the mouse widget owns the 3-byte FSM and reuses the
//! keyboard's frame validator entirely.  Concrete demonstration of
//! the layering story: the wire-level PS/2 protocol is *one*
//! widget; the keyboard scan-code interpretation, the mouse packet
//! interpretation, the (eventually) IntelliMouse 4-byte packet,
//! and any other PS/2-class device are all *separate* widgets that
//! delegate to it.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r"
     +--------+Ps2Mouse+--------+
     |                          |
bool |                          | B<8>
+--->| clk_in           x_delta +--->
bool |                          | B<8>
+--->| data_in          y_delta +--->
     |                  buttons +--->
     |                    valid +--->
     |                frame_err +--->
     +--------------------------+
")]
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/ps2_mouse.rs")]
//!```
//!
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/ps2_mouse.md")]
//!
//! And the auto-generated FSM diagram for the packet assembler:
#![doc = include_str!("../../doc/ps2_mouse_fsm.md")]
use rhdl::core::fsm::analysis::Transition;
use rhdl::prelude::*;

use super::ps2_keyboard::{Ps2Keyboard, ps2_keyboard as ps2_keyboard_kernel};
use crate::core::dff;

#[allow(unused_imports)]
use ps2_keyboard_kernel as _;

/// Author-curated transition graph for the mouse packet FSM.
pub const FSM_TRANSITIONS: &[Transition] = &[
    Transition {
        source_index: 0,
        target_index: 1,
    },
    Transition {
        source_index: 0,
        target_index: 0,
    }, // sync-bit fail keeps us in Byte0
    Transition {
        source_index: 1,
        target_index: 2,
    },
    Transition {
        source_index: 2,
        target_index: 0,
    },
];

/// Packet-assembler state.
#[derive(PartialEq, Debug, Digital, Clone, Copy, Default, Fsm)]
pub enum MouseByte {
    /// Awaiting byte 0 (status / buttons).
    #[default]
    #[fsm_state(label = "byte 0")]
    Byte0,
    /// Awaiting byte 1 (X delta).
    #[fsm_state(label = "byte 1 (X)")]
    Byte1,
    /// Awaiting byte 2 (Y delta).
    #[fsm_state(label = "byte 2 (Y)")]
    Byte2,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default, FsmWidget)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state", state_enum = MouseByte)]
/// PS/2 mouse receiver (3-byte standard packet).
pub struct Ps2Mouse {
    kbd: Ps2Keyboard,
    state: dff::DFF<MouseByte>,
    /// Latched byte-0 status.
    status_reg: dff::DFF<Bits<8>>,
    /// Latched X delta.
    x_reg: dff::DFF<Bits<8>>,
    /// Latched Y delta.
    y_reg: dff::DFF<Bits<8>>,
    /// One-cycle pulses.
    valid_pulse: dff::DFF<bool>,
    err_pulse: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to [Ps2Mouse] (same shape as [super::ps2_keyboard::In]).
pub struct In {
    pub clk_in: bool,
    pub data_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from [Ps2Mouse].
pub struct Out {
    /// Latched X delta from the most-recent packet (low byte; sign in `buttons`).
    pub x_delta: Bits<8>,
    /// Latched Y delta from the most-recent packet (low byte; sign in `buttons`).
    pub y_delta: Bits<8>,
    /// Latched byte-0 status word — bit layout per the rustdoc.
    pub buttons: Bits<8>,
    /// Pulses for one cycle when a complete 3-byte packet is captured.
    pub valid: bool,
    /// Pulses for one cycle on a sync-bit failure (resync) or upstream
    /// keyboard-receiver framing error.
    pub frame_err: bool,
}

impl SynchronousIO for Ps2Mouse {
    type I = In;
    type O = Out;
    type Kernel = ps2_mouse;
}

#[kernel]
/// Kernel for [Ps2Mouse].
pub fn ps2_mouse(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.kbd = super::ps2_keyboard::In {
        clk_in: i.clk_in,
        data_in: i.data_in,
    };
    d.state = q.state;
    d.status_reg = q.status_reg;
    d.x_reg = q.x_reg;
    d.y_reg = q.y_reg;
    d.valid_pulse = false;
    d.err_pulse = false;

    let kbd_valid = q.kbd.valid;
    let kbd_err = q.kbd.frame_err;
    let kbd_byte = q.kbd.scan_code;

    // Forward keyboard-level framing errors to the mouse-level err pulse —
    // the host doesn't need to subscribe to two error streams.
    if kbd_err {
        d.err_pulse = true;
    }

    if kbd_valid {
        match q.state {
            MouseByte::Byte0 => {
                // Validate sync bit (bit 3 must be 1).
                let sync_bit = (kbd_byte & bits::<8>(0x08)) != bits::<8>(0);
                if sync_bit {
                    d.status_reg = kbd_byte;
                    d.state = MouseByte::Byte1;
                } else {
                    d.err_pulse = true;
                    // Stay in Byte0 — next byte gets re-evaluated.
                }
            }
            MouseByte::Byte1 => {
                d.x_reg = kbd_byte;
                d.state = MouseByte::Byte2;
            }
            MouseByte::Byte2 => {
                d.y_reg = kbd_byte;
                d.valid_pulse = true;
                d.state = MouseByte::Byte0;
            }
        }
    }

    if cr.reset.any() {
        d.state = MouseByte::Byte0;
        d.status_reg = bits::<8>(0);
        d.x_reg = bits::<8>(0);
        d.y_reg = bits::<8>(0);
        d.valid_pulse = false;
        d.err_pulse = false;
    }

    let mut o = Out::dont_care();
    o.x_delta = q.x_reg;
    o.y_delta = q.y_reg;
    o.buttons = q.status_reg;
    o.valid = q.valid_pulse;
    o.frame_err = q.err_pulse;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::path::PathBuf;

    fn idle_in() -> In {
        In {
            clk_in: true,
            data_in: true,
        }
    }

    /// Build a stream that drives one PS/2 byte, with correct odd parity.
    fn drive_byte(data: u8) -> Vec<In> {
        let parity = (data.count_ones() % 2) == 0;
        let mut frame_bits: Vec<bool> = Vec::with_capacity(11);
        frame_bits.push(false);
        for i in 0..8 {
            frame_bits.push((data >> i) & 1 == 1);
        }
        frame_bits.push(parity);
        frame_bits.push(true);
        let mut out = Vec::new();
        for &b in &frame_bits {
            for _ in 0..3 {
                out.push(In { clk_in: true, data_in: b });
            }
            for _ in 0..3 {
                out.push(In { clk_in: false, data_in: b });
            }
        }
        // Trailing idle settle.
        for _ in 0..3 {
            out.push(idle_in());
        }
        out
    }

    /// Build a 3-byte packet stream.
    fn drive_packet(b0: u8, b1: u8, b2: u8) -> Vec<In> {
        let mut out = Vec::new();
        for _ in 0..2 {
            out.push(idle_in());
        }
        out.extend(drive_byte(b0));
        out.extend(drive_byte(b1));
        out.extend(drive_byte(b2));
        for _ in 0..6 {
            out.push(idle_in());
        }
        out
    }

    #[test]
    fn test_idle_no_valid() -> miette::Result<()> {
        let uut = Ps2Mouse::default();
        let stream = std::iter::repeat_n(idle_in(), 64)
            .with_reset(2)
            .clock_pos_edge(100);
        let any = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .any(|s| s.output.valid || s.output.frame_err);
        assert!(!any);
        Ok(())
    }

    #[test]
    fn test_full_packet_left_button_x10_yneg5() -> miette::Result<()> {
        let uut = Ps2Mouse::default();
        // status: sync(1) + Y_sign=0 X_sign=1 (negative Y delta),
        //         Y_OVF=0, X_OVF=0, M=0 R=0 L=1.
        // Byte 0 layout (bit 7 → 0): YO XO YS XS 1 M R L
        // To make byte 0 with sync=1, X_sign=0 (positive X), Y_sign=1 (neg Y),
        //   no overflows, only L button:
        // = 0 0 1 0 1 0 0 1 = 0x29
        // Byte 1 = 0x10 (X delta = +16)
        // Byte 2 = 0xFB (Y delta byte = -5 in two's complement; the sign in
        //          byte0 says "negative", magnitude is 0xFB → 251 unsigned but
        //          the host re-signs as needed).
        let stream_in = drive_packet(0x29, 0x10, 0xFB);
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let v_idx = outputs.iter().position(|s| s.output.valid);
        assert!(v_idx.is_some(), "no valid pulse from a complete packet");
        let v = &outputs[v_idx.unwrap()];
        assert_eq!(v.output.buttons.raw(), 0x29);
        assert_eq!(v.output.x_delta.raw(), 0x10);
        assert_eq!(v.output.y_delta.raw(), 0xFB);
        Ok(())
    }

    #[test]
    fn test_byte0_with_bad_sync_pulses_err() -> miette::Result<()> {
        // Byte 0 with sync bit (bit 3) cleared — must pulse frame_err and stay
        // in Byte0.  Send 0x21 (sync=0).  We send 3 such bytes; expect at
        // least one frame_err and no valid.
        let uut = Ps2Mouse::default();
        let mut stream_in = vec![idle_in(); 2];
        stream_in.extend(drive_byte(0x21));
        stream_in.extend(drive_byte(0x21));
        stream_in.extend(drive_byte(0x21));
        for _ in 0..6 {
            stream_in.push(idle_in());
        }
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let outputs: Vec<_> = uut
            .run(stream)
            .synchronous_sample()
            .filter(|s| !s.input.0.reset.any())
            .collect();
        let any_valid = outputs.iter().any(|s| s.output.valid);
        let err_count = outputs.iter().filter(|s| s.output.frame_err).count();
        assert!(!any_valid, "must NOT produce a valid packet from sync=0 bytes");
        assert!(err_count >= 1);
        Ok(())
    }

    #[test]
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Ps2Mouse::default();
        let desc = uut.descriptor("top".into())?;
        let hdl = desc.hdl()?.modules.pretty();
        let expect = expect!["22283"];
        expect.assert_eq(&hdl.len().to_string());
        Ok(())
    }

    #[test]
    fn test_ps2_mouse_hdl_works() -> miette::Result<()> {
        let uut = Ps2Mouse::default();
        let stream_in = drive_packet(0x29, 0x10, 0xFB);
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ps2_mouse_trace() -> miette::Result<()> {
        let uut = Ps2Mouse::default();
        let stream_in = drive_packet(0x29, 0x10, 0xFB);
        let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
        let vcd = uut.run(stream).collect::<VcdFile>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ps2_mouse");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["462672f46ac7e657e87d8c299111524b430e05450198baecfb3166849af13cb5"];
        let digest = vcd.dump_to_file(root.join("ps2_mouse.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_fsm_descriptor_round_trip() {
        let desc = Ps2Mouse::fsm_descriptor();
        assert_eq!(desc.widget_name, "Ps2Mouse");
        assert_eq!(desc.variants().len(), 3);
        assert_eq!(desc.initial_index(), 0);
    }
}
