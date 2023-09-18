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
                    Enum::A(a, b) => {
                        logger.write_string(tag, "A");
                        logger.write_bits(tag, *a as u128);
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
}
