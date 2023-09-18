use rhdl_bits::{Bits, SignedBits};

use crate::{logger::LoggerImpl, Kind, LogBuilder, TagID};

/// This is the core trait for all of `RHDL` data elements.  If you
/// want to use a data type in the hardware part of the design,
/// it must implement this trait.  
///
/// From <https://serde.rs/data-model.html>, we get a catalog of
/// different types that represent the data model used by Serde
/// (and by extension Rust).  Here are the list of types and
/// how they are represented in `RHDL`.
///
/// # Primitive types
///
/// Only the `bool` type is directly supported.  Otherwise,
/// use [Bits] or [SignedBits] to ensure that the arithmetic
/// operations model the behavior of the hardware.
///
/// # String, Byte Array, Unit, Unit Struct, Sequence, Map
///
/// These are all unsupported on a hardware target.  They either
/// have variable size or no size at all.
///
/// # Option
///
/// The option _is_ supported in `RHDL`.  It is represented as
/// an enum with two variants, precisely as it is in Rust.
///
/// # Enum Variants
///
/// Enum variants can be either empty or have a payload.  Empty
/// variants are represented via the discriminant, with no payload.
/// Variants with a payload are represented as a the discriminant
/// and the payload packed into binary representation.
///
/// # Structs, Tuples, Arrays, Unions
///
/// These are all supported in `RHDL`.
///
pub trait Digital: Copy + PartialEq + Sized + Clone {
    fn static_kind() -> Kind;
    fn kind(self) -> Kind {
        Self::static_kind()
    }
    fn bin(self) -> Vec<bool>;
    fn binary_string(self) -> String {
        self.bin()
            .iter()
            .rev()
            .map(|b| if *b { '1' } else { '0' })
            .collect()
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder);
    fn record<T: Digital>(&self, tag: TagID<T>, logger: impl LoggerImpl);
    fn skip<T: Digital>(tag: TagID<T>, logger: impl LoggerImpl);
}

impl Digital for bool {
    fn static_kind() -> Kind {
        Kind::make_bits(1)
    }
    fn bin(self) -> Vec<bool> {
        vec![self]
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, 1);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bool(tag, *self);
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.skip(tag);
    }
}

impl Digital for u8 {
    fn static_kind() -> Kind {
        Kind::make_bits(8)
    }
    fn bin(self) -> Vec<bool> {
        Bits::<8>::from(self as u128).to_bools()
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, 8);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bits(tag, Bits::<8>::from(*self as u128).raw());
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.skip(tag);
    }
}

impl Digital for u16 {
    fn static_kind() -> Kind {
        Kind::make_bits(16)
    }
    fn bin(self) -> Vec<bool> {
        Bits::<16>::from(self as u128).to_bools()
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, 16);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bits(tag, Bits::<16>::from(*self as u128).raw());
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.skip(tag);
    }
}

impl<const N: usize> Digital for Bits<N> {
    fn static_kind() -> Kind {
        Kind::make_bits(N)
    }
    fn bin(self) -> Vec<bool> {
        self.to_bools()
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, N);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bits(tag, self.raw());
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.skip(tag);
    }
}

impl<const N: usize> Digital for SignedBits<N> {
    fn static_kind() -> Kind {
        Kind::make_bits(N)
    }
    fn bin(self) -> Vec<bool> {
        self.as_unsigned().to_bools()
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, N);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bits(tag, self.as_unsigned().raw());
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.skip(tag);
    }
}

// Add blanket implementation for tuples up to size 4.
impl<T0: Digital, T1: Digital> Digital for (T0, T1) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![T0::static_kind(), T1::static_kind()])
    }
    fn bin(self) -> Vec<bool> {
        let mut v = self.0.bin();
        v.extend(self.1.bin());
        v
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        T0::allocate(tag, builder.namespace("0"));
        T1::allocate(tag, builder.namespace("1"));
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        T0::skip(tag, &mut logger);
        T1::skip(tag, &mut logger);
    }
}

impl<T0: Digital, T1: Digital, T2: Digital> Digital for (T0, T1, T2) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![
            T0::static_kind(),
            T1::static_kind(),
            T2::static_kind(),
        ])
    }
    fn bin(self) -> Vec<bool> {
        let mut v = self.0.bin();
        v.extend(self.1.bin());
        v.extend(self.2.bin());
        v
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        T0::allocate(tag, builder.namespace("0"));
        T1::allocate(tag, builder.namespace("1"));
        T2::allocate(tag, builder.namespace("2"));
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
        self.2.record(tag, &mut logger);
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        T0::skip(tag, &mut logger);
        T1::skip(tag, &mut logger);
        T2::skip(tag, &mut logger);
    }
}

impl<T0: Digital, T1: Digital, T2: Digital, T3: Digital> Digital for (T0, T1, T2, T3) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![
            T0::static_kind(),
            T1::static_kind(),
            T2::static_kind(),
            T3::static_kind(),
        ])
    }
    fn bin(self) -> Vec<bool> {
        let mut v = self.0.bin();
        v.extend(self.1.bin());
        v.extend(self.2.bin());
        v.extend(self.3.bin());
        v
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        T0::allocate(tag, builder.namespace("0"));
        T1::allocate(tag, builder.namespace("1"));
        T2::allocate(tag, builder.namespace("2"));
        T3::allocate(tag, builder.namespace("3"));
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
        self.2.record(tag, &mut logger);
        self.3.record(tag, &mut logger);
    }
    fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
        T0::skip(tag, &mut logger);
        T1::skip(tag, &mut logger);
        T2::skip(tag, &mut logger);
        T3::skip(tag, &mut logger);
    }
}

impl<T: Digital, const N: usize> Digital for [T; N] {
    fn static_kind() -> Kind {
        Kind::make_array(T::static_kind(), N)
    }
    fn bin(self) -> Vec<bool> {
        let mut v = Vec::new();
        for x in self.iter() {
            v.extend(x.bin());
        }
        v
    }
    fn allocate<U: Digital>(tag: TagID<U>, builder: impl LogBuilder) {
        for i in 0..N {
            T::allocate(tag, builder.namespace(&format!("{}", i)));
        }
    }
    fn record<U: Digital>(&self, tag: TagID<U>, mut logger: impl LoggerImpl) {
        for x in self.iter() {
            x.record(tag, &mut logger);
        }
    }
    fn skip<U: Digital>(tag: TagID<U>, mut logger: impl LoggerImpl) {
        for _ in 0..N {
            T::skip(tag, &mut logger);
        }
    }
}

#[cfg(test)]
mod test {
    use std::iter::repeat;

    use super::*;
    use crate::kind::{DiscriminantAlignment, Variant};

    #[test]
    #[allow(dead_code)]
    fn test_digital_enum_with_payloads() {
        #[derive(Copy, Clone, PartialEq)]
        enum Mixed {
            None,
            Bool(bool),
            Tuple(bool, Bits<3>),
            Array([bool; 3]),
            Strct { a: bool, b: Bits<3> },
        }

        impl Digital for Mixed {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    vec![
                        Variant {
                            name: "None".to_string(),
                            discriminant: Some(0),
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Bool".to_string(),
                            discriminant: Some(1),
                            kind: Kind::make_bits(1),
                        },
                        Variant {
                            name: "Tuple".to_string(),
                            discriminant: Some(2),
                            kind: Kind::make_tuple(vec![Kind::make_bits(1), Kind::make_bits(3)]),
                        },
                        Variant {
                            name: "Array".to_string(),
                            discriminant: Some(3),
                            kind: Kind::make_array(Kind::make_bits(1), 3),
                        },
                        Variant {
                            name: "Strct".to_string(),
                            discriminant: Some(4),
                            kind: Kind::make_struct(vec![
                                Kind::make_field("a", Kind::make_bits(1)),
                                Kind::make_field("b", Kind::make_bits(3)),
                            ]),
                        },
                    ],
                    Some(3),
                    DiscriminantAlignment::Lsb,
                )
            }
            fn bin(self) -> Vec<bool> {
                let raw = match self {
                    Self::None => rhdl_bits::bits::<3>(0).to_bools(),
                    Self::Bool(b) => {
                        let mut v = rhdl_bits::bits::<3>(1).to_bools();
                        v.extend(b.bin());
                        v
                    }
                    Self::Tuple(b, c) => {
                        let mut v = rhdl_bits::bits::<3>(2).to_bools();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v
                    }
                    Self::Array([b, c, d]) => {
                        let mut v = rhdl_bits::bits::<3>(3).to_bools();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v.extend(d.bin());
                        v
                    }
                    Self::Strct { a, b } => {
                        let mut v = rhdl_bits::bits::<3>(4).to_bools();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                };
                if raw.len() < self.kind().bits() {
                    let missing = self.kind().bits() - raw.len();
                    raw.into_iter().chain(repeat(false).take(missing)).collect()
                } else {
                    raw
                }
            }
            fn allocate<L: Digital>(tag: TagID<L>, builder: impl LogBuilder) {
                builder.allocate(tag, 0);
                <bool as Digital>::allocate(tag, builder.namespace("Bool"));
                <(bool, Bits<3>) as Digital>::allocate(tag, builder.namespace("Tuple"));
                <[bool; 3] as Digital>::allocate(tag, builder.namespace("Array"));
                {
                    let builder = builder.namespace("Strct");
                    <bool as Digital>::allocate(tag, builder.namespace("a"));
                    <Bits<3> as Digital>::allocate(tag, builder.namespace("b"));
                }
            }
            fn record<L: Digital>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
                match self {
                    Self::None => {
                        logger.write_string(tag, stringify!(None));
                        <bool as Digital>::skip(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                        <[bool; 3] as Digital>::skip(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                    }
                    Self::Bool(b) => {
                        logger.write_string(tag, stringify!(Bool));
                        b.record(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                        <[bool; 3] as Digital>::skip(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                    }
                    Self::Tuple(b, c) => {
                        logger.write_string(tag, stringify!(Tuple));
                        <bool as Digital>::skip(tag, &mut logger);
                        b.record(tag, &mut logger);
                        c.record(tag, &mut logger);
                        <[bool; 3] as Digital>::skip(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                    }
                    Self::Array([b, c, d]) => {
                        logger.write_string(tag, stringify!(Array));
                        <bool as Digital>::skip(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                        b.record(tag, &mut logger);
                        c.record(tag, &mut logger);
                        d.record(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                    }
                    Self::Strct { a, b } => {
                        logger.write_string(tag, stringify!(Strct));
                        <bool as Digital>::skip(tag, &mut logger);
                        <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                        <[bool; 3] as Digital>::skip(tag, &mut logger);
                        a.record(tag, &mut logger);
                        b.record(tag, &mut logger);
                    }
                }
            }
            fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
                logger.skip(tag); // Discriminant
                                  // None - no skip required
                <bool as Digital>::skip(tag, &mut logger);
                <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
                <[bool; 3] as Digital>::skip(tag, &mut logger);
                <(bool, Bits<3>) as Digital>::skip(tag, &mut logger);
            }
        }
        println!("{:?}", Mixed::None.bin());
        println!("{:?}", Mixed::Bool(true).bin());
        println!("{}", crate::text_grid(&Mixed::static_kind(), "val"));
        #[cfg(feature = "svg")]
        {
            let svg = crate::svg_grid(&Mixed::static_kind(), "val");
            svg::save("mixed.svg", &svg).unwrap();
        }
    }

    #[test]
    #[allow(dead_code)]
    fn test_digital_enum() {
        #[derive(Copy, Clone, PartialEq)]
        enum State {
            Init,
            Boot,
            Running,
            Stop,
            Boom,
        }
        impl Digital for State {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    vec![
                        Variant {
                            name: "Init".to_string(),
                            discriminant: Some(0),
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boot".to_string(),
                            discriminant: Some(1),
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Running".to_string(),
                            discriminant: Some(2),
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Stop".to_string(),
                            discriminant: Some(3),
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boom".to_string(),
                            discriminant: Some(4),
                            kind: Kind::Empty,
                        },
                    ],
                    Some(3),
                    DiscriminantAlignment::Lsb,
                )
            }
            fn bin(self) -> Vec<bool> {
                match self {
                    Self::Init => rhdl_bits::bits::<3>(0).to_bools(),
                    Self::Boot => rhdl_bits::bits::<3>(1).to_bools(),
                    Self::Running => rhdl_bits::bits::<3>(2).to_bools(),
                    Self::Stop => rhdl_bits::bits::<3>(3).to_bools(),
                    Self::Boom => rhdl_bits::bits::<3>(4).to_bools(),
                }
            }
            fn allocate<L: Digital>(tag: TagID<L>, builder: impl LogBuilder) {
                builder.allocate(tag, 0);
            }
            fn record<L: Digital>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
                match self {
                    Self::Init => logger.write_string(tag, stringify!(Init)),
                    Self::Boot => logger.write_string(tag, stringify!(Boot)),
                    Self::Running => logger.write_string(tag, stringify!(Running)),
                    Self::Stop => logger.write_string(tag, stringify!(Stop)),
                    Self::Boom => logger.write_string(tag, stringify!(Boom)),
                }
            }
            fn skip<L: Digital>(tag: TagID<L>, mut logger: impl LoggerImpl) {
                logger.skip(tag);
            }
        }
        let val = State::Boom;
        assert_eq!(val.bin(), rhdl_bits::bits::<3>(4).to_bools());
        assert_eq!(
            val.kind(),
            Kind::make_enum(
                vec![
                    Variant {
                        name: "Init".to_string(),
                        discriminant: Some(0),
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boot".to_string(),
                        discriminant: Some(1),
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Running".to_string(),
                        discriminant: Some(2),
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Stop".to_string(),
                        discriminant: Some(3),
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boom".to_string(),
                        discriminant: Some(4),
                        kind: Kind::Empty,
                    },
                ],
                Some(3),
                DiscriminantAlignment::Lsb,
            )
        );
    }
}
