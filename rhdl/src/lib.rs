pub mod basic_logger;
pub mod bits;
pub mod core;

pub use crate::bits::Bits;
pub use crate::core::Digital;
pub use crate::core::Kind;
pub use crate::core::LogBuilder;
pub use crate::core::LoggerImpl;
pub use crate::core::TagID;
pub use rhdl_macro::Digital;

#[cfg(test)]
mod tests {

    use rhdl_core::{
        path::{bit_range, Path},
        DiscriminantAlignment, Logger,
    };

    use super::*;

    #[test]
    fn test_vcd_enum() {
        #[derive(Clone, Copy, Debug, PartialEq)]
        enum Enum {
            None,
            A(u8, u16),
            B { name: u8 },
            C(bool),
        }

        impl Digital for Enum {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    vec![
                        Kind::make_variant("None", Kind::Empty, 0),
                        Kind::make_variant(
                            "A",
                            Kind::make_tuple(vec![Kind::make_bits(8), Kind::make_bits(16)]),
                            1,
                        ),
                        Kind::make_variant(
                            "B",
                            Kind::make_struct(vec![Kind::make_field("name", Kind::make_bits(8))]),
                            2,
                        ),
                        Kind::make_variant(
                            "C",
                            Kind::make_struct(vec![Kind::make_field("a", Kind::make_bits(1))]),
                            3,
                        ),
                    ],
                    3,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                let raw = match self {
                    Enum::None => rhdl_bits::bits::<2>(0).to_bools(),
                    Enum::A(a, b) => {
                        let mut v = rhdl_bits::bits::<2>(1).to_bools();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                    Enum::B { name } => {
                        let mut v = rhdl_bits::bits::<2>(2).to_bools();
                        v.extend(name.bin());
                        v
                    }
                    Enum::C(a) => {
                        let mut v = rhdl_bits::bits::<2>(3).to_bools();
                        v.extend(a.bin());
                        v
                    }
                };
                let raw = if raw.len() < self.kind().bits() {
                    let missing = self.kind().bits() - raw.len();
                    raw.into_iter()
                        .chain(std::iter::repeat(false).take(missing))
                        .collect()
                } else {
                    raw
                };
                // if alignment is msb, move the bottom bits to the top
                if DiscriminantAlignment::Msb == DiscriminantAlignment::Msb {
                    let (payload, discriminant) = raw.split_at(2);
                    discriminant.iter().chain(payload.iter()).copied().collect()
                } else {
                    raw
                }
            }
            fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
                // Allocate the enum tag
                builder.namespace("$disc").allocate(tag, 0);
                // For the variants, allocate space for them
                // For the None variant, we do not need to allocate additional space
                // For the A variant, we need to allocate space for the u8 and u16
                {
                    let builder = builder.namespace("A");
                    <u8 as Digital>::allocate(tag, builder.namespace("0"));
                    <u16 as Digital>::allocate(tag, builder.namespace("1"));
                }
                // The struct case must be done inline
                {
                    let builder = builder.namespace("B");
                    <u8 as Digital>::allocate(tag, builder.namespace("name"));
                    <u8 as Digital>::allocate(tag, builder.namespace("name_2"));
                }
                <bool as Digital>::allocate(tag, builder.namespace("C"));
            }
            fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
                match self {
                    Enum::None => {
                        logger.write_string(tag, "None");
                        <(u8, u16) as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <bool as Digital>::skip(tag, &mut logger);
                    }
                    Enum::A(t, b) => {
                        logger.write_string(tag, "A");
                        logger.write_bits(tag, *t as u128);
                        logger.write_bits(tag, *b as u128);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <bool as Digital>::skip(tag, &mut logger);
                    }
                    Enum::B { name } => {
                        logger.write_string(tag, "B");
                        <(u8, u16) as Digital>::skip(tag, &mut logger);
                        logger.write_bits(tag, *name as u128);
                        logger.write_bits(tag, *name as u128);
                        <bool as Digital>::skip(tag, &mut logger);
                    }
                    Enum::C(a) => {
                        logger.write_string(tag, "C");
                        <(u8, u16) as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        logger.write_bool(tag, *a);
                    }
                }
            }
            fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
                logger.skip(tag);
                <(u8, u16) as Digital>::skip(tag, &mut logger);
                <u8 as Digital>::skip(tag, &mut logger);
                <bool as Digital>::skip(tag, &mut logger);
            }
        }

        let mut builder = basic_logger::Builder::default();
        let tag = builder.tag::<Enum>("enum");
        let tag2 = builder.tag::<u8>("color");
        let mut logger = builder.build();
        logger.set_time_in_fs(0);
        logger.log(tag, Enum::None);
        logger.log(tag2, 0b10101010);
        logger.set_time_in_fs(1_000);
        logger.log(tag, Enum::A(42, 1024));
        logger.set_time_in_fs(2_000);
        logger.log(tag, Enum::B { name: 67 });
        logger.set_time_in_fs(3_000);
        logger.log(tag, Enum::C(true));
        logger.set_time_in_fs(4_000);
        logger.log(tag, Enum::C(false));
        logger.set_time_in_fs(5_000);
        logger.log(tag, Enum::B { name: 65 });
        logger.set_time_in_fs(6_000);
        logger.log(tag, Enum::A(21, 512));
        logger.set_time_in_fs(7_000);
        logger.log(tag, Enum::None);
        logger.set_time_in_fs(8_000);
        logger.log(tag, Enum::None);
        let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
        logger.vcd(&mut vcd_file).unwrap();
        println!("{:?} {}", Enum::None, Enum::None.binary_string());
        assert_eq!(Enum::None.bin().len(), Enum::static_kind().bits());
        let a = Enum::A(21, 512);
        println!("{:?} {}", a, a.binary_string());
        assert_eq!(a.bin().len(), Enum::static_kind().bits());
    }

    #[test]
    fn test_vcd_basic() {
        #[derive(Clone, Copy, PartialEq)]
        pub struct Simple {
            a: bool,
            b: Bits<8>,
        }

        impl Digital for Simple {
            fn static_kind() -> Kind {
                Kind::make_struct(vec![
                    Kind::make_field("a", Kind::make_bits(1)),
                    Kind::make_field("b", Kind::make_bits(8)),
                ])
            }
            fn bin(self) -> Vec<bool> {
                let mut result = vec![self.a];
                result.extend(self.b.bin());
                result
            }
            fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
                <bool as Digital>::allocate(tag, builder.namespace("a"));
                <Bits<8> as Digital>::allocate(tag, builder.namespace("b"));
            }
            fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
                self.a.record(tag, &mut logger);
                self.b.record(tag, &mut logger);
            }
            fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
                <bool as Digital>::skip(tag, &mut logger);
                <Bits<8> as Digital>::skip(tag, &mut logger);
            }
        }

        let mut builder = basic_logger::Builder::default();
        let tag = builder.tag::<Simple>("simple");
        let simple = Simple {
            a: true,
            b: Bits::from(0b10101010),
        };
        let mut logger = builder.build();
        logger.set_time_in_fs(0);
        logger.log(tag, simple);
        logger.set_time_in_fs(1_000);
        let simple = Simple {
            a: false,
            b: Bits::from(0b01010101),
        };
        logger.log(tag, simple);
        logger.set_time_in_fs(2_000);
        logger.log(tag, simple);
        let mut vcd_file = std::fs::File::create("test.vcd").unwrap();
        logger.vcd(&mut vcd_file).unwrap();
    }

    #[test]
    #[allow(dead_code)]
    #[allow(clippy::just_underscores_and_digits)]
    fn test_derive() {
        #[derive(Clone, Copy, PartialEq)]
        enum Test {
            A,
            B(Bits<16>),
            C { a: Bits<32>, b: Bits<8> },
        }

        impl rhdl_core::Digital for Test {
            fn static_kind() -> rhdl_core::Kind {
                Kind::make_enum(
                    vec![
                        Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1),
                        Kind::make_variant(
                            stringify!(B),
                            rhdl_core::Kind::make_tuple(vec![
                                <Bits<16> as rhdl_core::Digital>::static_kind(),
                            ]),
                            2,
                        ),
                        Kind::make_variant(
                            stringify!(C),
                            rhdl_core::Kind::make_struct(vec![
                                rhdl_core::Kind::make_field(
                                    stringify!(a),
                                    <Bits<32> as rhdl_core::Digital>::static_kind(),
                                ),
                                rhdl_core::Kind::make_field(
                                    stringify!(b),
                                    <Bits<8> as rhdl_core::Digital>::static_kind(),
                                ),
                            ]),
                            3,
                        ),
                    ],
                    2,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind().pad(match self {
                    Self::A => rhdl_bits::bits::<2usize>(0usize as u128).to_bools(),
                    Self::B(_0) => {
                        let mut v = rhdl_bits::bits::<2usize>(1usize as u128).to_bools();
                        v.extend(_0.bin());
                        v
                    }
                    Self::C { a, b } => {
                        let mut v = rhdl_bits::bits::<2usize>(2usize as u128).to_bools();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                })
            }
            fn allocate<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                builder: impl rhdl_core::LogBuilder,
            ) {
                builder.allocate(tag, 0);
                {
                    let builder = builder.namespace(stringify!(B));
                    <Bits<16> as rhdl_core::Digital>::allocate(
                        tag,
                        builder.namespace(stringify!(0)),
                    );
                }
                {
                    let builder = builder.namespace(stringify!(C));
                    <Bits<32> as rhdl_core::Digital>::allocate(
                        tag,
                        builder.namespace(stringify!(a)),
                    );
                    <Bits<8> as rhdl_core::Digital>::allocate(
                        tag,
                        builder.namespace(stringify!(b)),
                    );
                }
            }
            fn record<L: rhdl_core::Digital>(
                &self,
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                match self {
                    Self::A => {
                        logger.write_string(tag, stringify!(A));
                        <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
                    }
                    Self::B(_0) => {
                        logger.write_string(tag, stringify!(B));
                        _0.record(tag, &mut logger);
                        <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
                    }
                    Self::C { a, b } => {
                        logger.write_string(tag, stringify!(C));
                        <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                        a.record(tag, &mut logger);
                        b.record(tag, &mut logger);
                    }
                }
            }
            fn skip<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                logger.skip(tag);
                <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
            }
        }
    }

    #[test]
    #[allow(dead_code)]
    fn test_derive_no_payload() {
        #[derive(Copy, Clone, PartialEq)]
        pub enum State {
            Init,
            Boot,
            Running,
            Stop,
            Boom,
        }
        impl rhdl_core::Digital for State {
            fn static_kind() -> rhdl_core::Kind {
                Kind::make_enum(
                    vec![
                        Kind::make_variant(stringify!(Init), rhdl_core::Kind::Empty, 0),
                        Kind::make_variant(stringify!(Boot), rhdl_core::Kind::Empty, 1),
                        Kind::make_variant(stringify!(Running), rhdl_core::Kind::Empty, 2),
                        Kind::make_variant(stringify!(Stop), rhdl_core::Kind::Empty, 3),
                        Kind::make_variant(stringify!(Boom), rhdl_core::Kind::Empty, 4),
                    ],
                    3,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind().pad(match self {
                    Self::Init => rhdl_bits::bits::<3usize>(0usize as u128).to_bools(),
                    Self::Boot => rhdl_bits::bits::<3usize>(1usize as u128).to_bools(),
                    Self::Running => rhdl_bits::bits::<3usize>(2usize as u128).to_bools(),
                    Self::Stop => rhdl_bits::bits::<3usize>(3usize as u128).to_bools(),
                    Self::Boom => rhdl_bits::bits::<3usize>(4usize as u128).to_bools(),
                })
            }
            fn allocate<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                builder: impl rhdl_core::LogBuilder,
            ) {
                builder.allocate(tag, 0);
            }
            fn record<L: rhdl_core::Digital>(
                &self,
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                match self {
                    Self::Init => {
                        logger.write_string(tag, stringify!(Init));
                    }
                    Self::Boot => {
                        logger.write_string(tag, stringify!(Boot));
                    }
                    Self::Running => {
                        logger.write_string(tag, stringify!(Running));
                    }
                    Self::Stop => {
                        logger.write_string(tag, stringify!(Stop));
                    }
                    Self::Boom => {
                        logger.write_string(tag, stringify!(Boom));
                    }
                }
            }
            fn skip<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                logger.skip(tag);
            }
        }
    }

    #[test]
    fn test_derive_digital_simple_struct() {
        use rhdl_bits::alias::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital)]
        struct Test {
            a: bool,
            b: b8,
        }

        let foo = Test {
            a: true,
            b: b8::from(0b10101011),
        };

        println!("foo val: {}", foo.binary_string());
        let test_kind = Test::static_kind();
        let (range, kind) = bit_range(test_kind, &[Path::Field("b")]).unwrap();
        println!("range: {:?}", range);
        println!("kind: {:?}", kind);
        assert_eq!(range, 1..9);
        assert_eq!(kind, Kind::make_bits(8));
        let bits = foo.bin();
        let bits = &bits[range];
        assert_eq!(bits.len(), 8);
        assert_eq!(bits, [true, true, false, true, false, true, false, true]);
    }

    #[test]
    #[allow(dead_code)]
    fn test_derive_complex_enum_and_decode_with_path() -> anyhow::Result<()> {
        use rhdl_bits::alias::*;
        use rhdl_core::path::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital)]
        enum Test {
            A,
            B(b2, b3),
            C { a: b8, b: b8 },
        }

        let foo = Test::B(b2::from(0b10), b3::from(0b101));
        let disc = vec![Path::EnumPayload(stringify!(B)), Path::Index(1)];
        let index = bit_range(Test::static_kind(), &disc)?;
        println!("{:?}", index);
        let bits = foo.bin();
        let bits = &bits[index.0];
        println!(
            "Extracted bits: {}",
            bits.iter()
                .rev()
                .map(|x| if *x { '1' } else { '0' })
                .collect::<String>()
        );
        let (disc_range, disc_kind) = bit_range(Test::static_kind(), &[Path::EnumDiscriminant])?;
        println!("{:?}", disc_range);
        println!("{:?}", disc_kind);
        let disc_bits = foo.bin();
        let disc_bits = &disc_bits[disc_range];
        assert_eq!(disc_bits, [true, false]);
        Ok(())
    }

    #[test]
    fn test_derive_digital_complex_enum() {
        use rhdl_bits::alias::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital)]
        enum Test {
            A,
            B(b2, b3),
            C { a: b8, b: b8 },
        }

        let foo_1 = Test::C {
            a: b8::from(0b10101010),
            b: b8::from(0b11010111),
        };

        println!("foo val: {}", foo_1.binary_string());

        let foo_2 = Test::B(b2::from(0b10), b3::from(0b101));

        println!("foo val: {}", foo_2.binary_string());

        let foo_3 = Test::A;

        let mut builder = basic_logger::Builder::default();
        let tag = builder.tag("test");
        let mut logger = builder.build();
        logger.set_time_in_fs(0);
        logger.log(tag, foo_1);
        logger.set_time_in_fs(1_000);
        logger.log(tag, foo_2);
        logger.set_time_in_fs(2_000);
        logger.log(tag, foo_3);
        logger.set_time_in_fs(3_000);
        logger.log(tag, foo_1);
        let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
        logger.vcd(&mut vcd_file).unwrap();
    }

    #[test]
    fn test_derive_enum_explicit_discriminant_width() {
        use rhdl_bits::alias::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital)]
        #[rhdl(discriminant_width = 4)]
        enum Test {
            A,
            B(b2, b3),
            C { a: b8, b: b8 },
        }

        let (range, kind) = bit_range(Test::static_kind(), &[Path::EnumDiscriminant]).unwrap();
        assert_eq!(range.len(), 4);
        assert_eq!(kind, Kind::make_bits(4));
    }

    #[test]
    fn test_derive_enum_alignment_lsb() {
        use rhdl_bits::alias::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital)]
        #[rhdl(discriminant_align = "lsb")]
        enum Test {
            A,
            B(b2, b3),
            C { a: b8, b: b8 },
        }
        let (range, kind) = bit_range(Test::static_kind(), &[Path::EnumDiscriminant]).unwrap();
        assert_eq!(range, 0..2);
        assert_eq!(kind, Kind::make_bits(2));
    }

    #[test]
    fn test_nested_data_type() {
        use rhdl_bits::alias::*;

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

        let foo = Packet::State(State::Boom);
        assert_eq!(
            foo.path(&[
                Path::EnumPayload("State"),
                Path::Index(0),
                Path::EnumDiscriminant
            ])
            .unwrap()
            .0,
            b5::from(0b01010).bin()
        );
        assert_eq!(
            foo.path(&[Path::EnumDiscriminant]).unwrap().0,
            b5::from(0b01000).bin()
        );
    }
}
