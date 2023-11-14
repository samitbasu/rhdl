use rhdl::basic_logger;
use rhdl_bits::alias::*;
use rhdl_core::{path::Path, Digital, Kind, LogBuilder, Logger};
use rhdl_macro::Digital;

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
#[repr(i8)]
enum Packet {
    Color { r: b8, g: b8, b: b8 } = 1,
    Size { w: b16, h: b16 } = 2,
    Position(b4, b4) = 4,
    State(State) = 8,
    Log { msg: b32, level: LogLevel } = 16,
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

#[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
enum State {
    #[default]
    Init = -2,
    Boot,
    Running,
    Stop,
    Boom = 2,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
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
    }
    .typed_bits();
    assert_eq!(
        foo.path(&[Path::EnumPayload("Color"), Path::Field("g")])
            .unwrap()
            .bits,
        b8::from(0b11010101).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Color"), Path::Field("g")])
            .unwrap()
            .kind,
        Kind::make_bits(8)
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Color"), Path::Field("r")])
            .unwrap()
            .bits,
        b8::from(0b10101010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumDiscriminant]).unwrap().bits,
        b5::from(0b00001).bin()
    );
}

#[test]
fn test_size_case() {
    let foo = Packet::Size {
        w: b16::from(0b1010101010101010),
        h: b16::from(0b1101010110101010),
    }
    .typed_bits();
    assert_eq!(
        foo.path(&[Path::EnumPayload("Size"), Path::Field("w")])
            .unwrap()
            .bits,
        b16::from(0b1010101010101010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Size"), Path::Field("w")])
            .unwrap()
            .kind,
        Kind::make_bits(16)
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Size"), Path::Field("h")])
            .unwrap()
            .bits,
        b16::from(0b1101010110101010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumDiscriminant]).unwrap().bits,
        b5::from(0b00010).bin()
    );
}

#[test]
fn test_position_case() {
    let foo = Packet::Position(b4::from(0b1010), b4::from(0b1101)).typed_bits();
    assert_eq!(
        foo.path(&[Path::EnumPayload("Position"), Path::Index(0)])
            .unwrap()
            .bits,
        b4::from(0b1010).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Position"), Path::Index(0)])
            .unwrap()
            .kind,
        Kind::make_bits(4)
    );
    assert_eq!(
        foo.path(&[Path::EnumPayload("Position"), Path::Index(1)])
            .unwrap()
            .bits,
        b4::from(0b1101).bin()
    );
    assert_eq!(
        foo.path(&[Path::EnumDiscriminant]).unwrap().bits,
        b5::from(0b00100).bin()
    );
}

#[test]
fn test_state_case() {
    let packet = Packet::State(State::Boom).typed_bits();
    assert_eq!(
        packet
            .path(&[
                Path::EnumPayload("State"),
                Path::Index(0),
                Path::EnumDiscriminant
            ])
            .unwrap()
            .bits,
        s3::from(2).bin()
    );
    assert_eq!(
        packet.path(&[Path::EnumDiscriminant]).unwrap().bits,
        b5::from(0b01000).bin()
    );
    let packet = Packet::State(State::Init).typed_bits();
    assert_eq!(
        packet
            .path(&[
                Path::EnumPayload("State"),
                Path::Index(0),
                Path::EnumDiscriminant
            ])
            .unwrap()
            .bits,
        s3::from(-2).bin()
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
            .path(&[Path::EnumPayload("Log"), Path::Field("msg")])
            .unwrap()
            .bits,
        b32::from(0xDEAD_BEEF).bin()
    );
    assert_eq!(
        packet
            .path(&[
                Path::EnumPayload("Log"),
                Path::Field("level"),
                Path::Field("active")
            ])
            .unwrap()
            .bits,
        b1::from(1).bin()
    );
    assert_eq!(
        packet
            .path(&[
                Path::EnumPayload("Log"),
                Path::Field("level"),
                Path::Field("level")
            ])
            .unwrap()
            .bits,
        b8::from(0xBA).bin()
    )
}

#[cfg(feature = "svg")]
#[test]
fn test_documentation_svgs() {
    let svg = rhdl_core::svg_grid_vertical(&Packet::static_kind(), "Packet");
    svg::save("packets.svg", &svg).unwrap();
}

#[test]
fn test_vcd_generation() {
    let mut builder = basic_logger::Builder::default();
    let tag = builder.tag("packet");
    let mut logger = builder.build();
    logger.set_time_in_fs(0);
    logger.log(
        tag,
        Packet::Color {
            r: b8::from(0b10101010),
            g: b8::from(0b11010101),
            b: b8::from(0b11110000),
        },
    );
    logger.set_time_in_fs(1_000);
    logger.log(
        tag,
        Packet::Size {
            w: 0xDEAD.into(),
            h: 0xBEEF.into(),
        },
    );
    logger.set_time_in_fs(2_000);
    logger.log(tag, Packet::Position(0b1010.into(), 0b1101.into()));
    logger.set_time_in_fs(3_000);
    logger.log(tag, Packet::State(State::Boom));
    logger.set_time_in_fs(4_000);
    logger.log(tag, Packet::State(State::Init));
    logger.set_time_in_fs(5_000);
    logger.log(
        tag,
        Packet::Log {
            msg: 0xCAFE_BEEF.into(),
            level: LogLevel {
                level: 0xBA.into(),
                active: true,
            },
        },
    );
    logger.set_time_in_fs(6_000);
    logger.log(tag, Packet::State(State::Running));
    let mut vcd_file = std::fs::File::create("packet.vcd").unwrap();
    logger.vcd(&mut vcd_file).unwrap();
}
