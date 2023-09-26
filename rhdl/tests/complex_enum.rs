use rhdl_bits::alias::*;
use rhdl_core::{path::Path, Digital, Kind};
use rhdl_macro::Digital;

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
#[repr(u8)]
enum Packet {
    Color { r: b8, g: b8, b: b8 } = 1,
    Size { w: b16, h: b16 } = 2,
    Position(b4, b4) = 4,
    State(State) = 8,
    Log { msg: b32, level: LogLevel } = 16,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
enum State {
    Init = -2,
    Boot,
    Running,
    Stop,
    Boom = 2,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
struct LogLevel {
    level: b8,
    active: bool,
}

#[test]
fn test_color_case() {
    let foo = Packet::Color {
        r: b8::from(0b10101010),
        g: b8::from(0b11010101),
        b: b8::from(0b11110000),
    };
    assert_eq!(
        foo.path(&[Path::EnumPayload("Color"), Path::Field("g")])
            .unwrap()
            .0,
        b8::from(0b11010101).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Color"), Path::Field("g")])
            .unwrap()
            .1,
        Kind::make_bits(8)
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Color"), Path::Field("r")])
            .unwrap()
            .0,
        b8::from(0b10101010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumDiscriminant]).unwrap().0,
        b5::from(0b00001).bin()
    );
}

#[test]
fn test_size_case() {
    let foo = Packet::Size {
        w: b16::from(0b1010101010101010),
        h: b16::from(0b1101010110101010),
    };
    assert_eq!(
        foo.path(&[Path::EnumPayload("Size"), Path::Field("w")])
            .unwrap()
            .0,
        b16::from(0b1010101010101010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Size"), Path::Field("w")])
            .unwrap()
            .1,
        Kind::make_bits(16)
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Size"), Path::Field("h")])
            .unwrap()
            .0,
        b16::from(0b1101010110101010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumDiscriminant]).unwrap().0,
        b5::from(0b00010).bin()
    );
}

#[test]
fn test_position_case() {
    let foo = Packet::Position(b4::from(0b1010), b4::from(0b1101));
    assert_eq!(
        foo.path(&[Path::EnumPayload("Position"), Path::Index(0)])
            .unwrap()
            .0,
        b4::from(0b1010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Position"), Path::Index(0)])
            .unwrap()
            .1,
        Kind::make_bits(4)
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Position"), Path::Index(1)])
            .unwrap()
            .0,
        b4::from(0b1101).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumDiscriminant]).unwrap().0,
        b5::from(0b00100).bin()
    );
}

#[test]
fn test_state_case() {
    let foo = Packet::State(State::Boom);
    dbg!(foo.path(&[Path::EnumPayload("State")]));
    assert_eq!(
        foo.path(&[Path::EnumPayload("State"), Path::EnumDiscriminant])
            .unwrap()
            .0,
        b5::from(0b01010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumDiscriminant]).unwrap().0,
        b5::from(0b01000).bin()
    );
}
