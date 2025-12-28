#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rhdl::prelude::*;

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
#[repr(i8)]
enum Packet {
    Color { r: b8, g: b8, b: b8 } = 1,
    Size { w: b16, h: b16 } = 2,
    Position(b4, b4) = 4,
    State(State) = 8,
    Log { msg: b32, level: LogLevel } = 16,
    Invalid,
}

impl Default for Packet {
    fn default() -> Self {
        Self::Color {
            r: b8::from(0),
            g: b8::from(0),
            b: b8::from(0),
        }
    }
}

#[test]
fn test_packet_random() {
    for _ in 0..10 {
        let packet = Packet::default();
        eprintln!("{packet:?}");
    }
}

#[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
enum State {
    #[default]
    Init = -2,
    Boot,
    Running,
    Stop,
    Boom = 2,
    Invalid,
}

#[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
struct LogLevel {
    level: b8,
    active: bool,
}

#[test]
fn test_color_case() {
    let foo_test = Packet::Color {
        r: b8::from(0b10101010),
        g: b8::from(0b11010101),
        b: b8::from(0b11110000),
    }
    .typed_bits();
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Color").field("g"))
            .unwrap()
            .bits(),
        b8::from(0b11010101).bin().to_vec()
    );
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Color").field("g"))
            .unwrap()
            .kind(),
        Kind::make_bits(8)
    );
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Color").field("r"))
            .unwrap()
            .bits(),
        b8::from(0b10101010).bin().to_vec()
    );
    assert_eq!(
        foo_test
            .path(&Path::default().discriminant())
            .unwrap()
            .bits(),
        b5::from(0b00001).bin().to_vec()
    );
}

#[test]
fn test_size_case() {
    let foo_test = Packet::Size {
        w: b16::from(0b1010101010101010),
        h: b16::from(0b1101010110101010),
    }
    .typed_bits();
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Size").field("w"))
            .unwrap()
            .bits(),
        b16::from(0b1010101010101010).bin().to_vec()
    );
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Size").field("w"))
            .unwrap()
            .kind(),
        Kind::make_bits(16)
    );
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Size").field("h"))
            .unwrap()
            .bits(),
        b16::from(0b1101010110101010).bin().to_vec()
    );
    assert_eq!(
        foo_test
            .path(&Path::default().discriminant())
            .unwrap()
            .bits(),
        b5::from(0b00010).bin().to_vec()
    );
}

#[test]
fn test_position_case() {
    let foo_test = Packet::Position(b4::from(0b1010), b4::from(0b1101)).typed_bits();
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Position").tuple_index(0))
            .unwrap()
            .bits(),
        b4::from(0b1010).bin().to_vec()
    );
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Position").tuple_index(0))
            .unwrap()
            .kind(),
        Kind::make_bits(4)
    );
    assert_eq!(
        foo_test
            .path(&Path::default().payload("Position").tuple_index(1))
            .unwrap()
            .bits(),
        b4::from(0b1101).bin().to_vec()
    );
    assert_eq!(
        foo_test
            .path(&Path::default().discriminant())
            .unwrap()
            .bits(),
        b5::from(0b00100).bin().to_vec()
    );
}

#[test]
fn test_state_case() {
    let packet = Packet::State(State::Boom).typed_bits();
    assert_eq!(
        packet
            .path(
                &Path::default()
                    .payload("State")
                    .tuple_index(0)
                    .discriminant()
            )
            .unwrap()
            .bits(),
        s3::from(2).bin().to_vec()
    );
    assert_eq!(
        packet.path(&Path::default().discriminant()).unwrap().bits(),
        b5::from(0b01000).bin().to_vec()
    );
    let packet = Packet::State(State::Init).typed_bits();
    assert_eq!(
        packet
            .path(
                &Path::default()
                    .payload("State")
                    .tuple_index(0)
                    .discriminant()
            )
            .unwrap()
            .bits(),
        s3::from(-2).bin().to_vec()
    );
}

#[test]
fn test_nested_struct_case() {
    let packet = Packet::Log {
        msg: b32::from(0xDEAD_BEEF),
        level: LogLevel {
            level: b8::from(0xBA),
            active: true,
        },
    }
    .typed_bits();
    assert_eq!(
        packet
            .path(&Path::default().payload("Log").field("msg"))
            .unwrap()
            .bits(),
        b32::from(0xDEAD_BEEF).bin().to_vec()
    );
    assert_eq!(
        packet
            .path(
                &Path::default()
                    .payload("Log")
                    .field("level")
                    .field("active")
            )
            .unwrap()
            .bits(),
        b1::from(1).bin().to_vec()
    );
    assert_eq!(
        packet
            .path(&Path::default().payload("Log").field("level").field("level"))
            .unwrap()
            .bits(),
        b8::from(0xBA).bin().to_vec()
    )
}

#[test]
fn test_documentation_svgs() {
    let svg = Packet::static_kind().svg("Packet");
    expect_test::expect_file!["expect/complex_enum_packet.svg"].assert_eq(&svg.to_string());
}
