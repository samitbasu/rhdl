pub mod basic_logger;
pub mod bits;
pub mod core;

pub use crate::bits::Bits;
pub use crate::core::Digital;
pub use crate::core::Kind;
pub use crate::core::LogBuilder;
pub use crate::core::LoggerImpl;
pub use crate::core::TagID;

#[cfg(test)]
mod tests {

    use rhdl_core::{DiscriminantAlignment, Logger};

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
                        Kind::make_variant("None", Kind::Empty),
                        Kind::make_variant(
                            "A",
                            Kind::make_tuple(vec![Kind::make_bits(8), Kind::make_bits(16)]),
                        ),
                        Kind::make_variant(
                            "B",
                            Kind::make_struct(vec![Kind::make_field("name", Kind::make_bits(8))]),
                        ),
                        Kind::make_variant(
                            "C",
                            Kind::make_struct(vec![Kind::make_field("a", Kind::make_bits(1))]),
                        ),
                    ],
                    None,
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
                        Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty)
                            .with_discriminant(Some(1)),
                        Kind::make_variant(
                            stringify!(B),
                            rhdl_core::Kind::make_tuple(vec![
                                <Bits<16> as rhdl_core::Digital>::static_kind(),
                            ]),
                        )
                        .with_discriminant(None),
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
                        )
                        .with_discriminant(None),
                    ],
                    None,
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
                    let mut builder = builder.namespace(stringify!(B));
                    <Bits<16> as rhdl_core::Digital>::allocate(
                        tag,
                        builder.namespace(stringify!(0)),
                    );
                }
                {
                    let mut builder = builder.namespace(stringify!(C));
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
                        Kind::make_variant(stringify!(Init), rhdl_core::Kind::Empty)
                            .with_discriminant(None),
                        Kind::make_variant(stringify!(Boot), rhdl_core::Kind::Empty)
                            .with_discriminant(None),
                        Kind::make_variant(stringify!(Running), rhdl_core::Kind::Empty)
                            .with_discriminant(None),
                        Kind::make_variant(stringify!(Stop), rhdl_core::Kind::Empty)
                            .with_discriminant(None),
                        Kind::make_variant(stringify!(Boom), rhdl_core::Kind::Empty)
                            .with_discriminant(None),
                    ],
                    None,
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
}
