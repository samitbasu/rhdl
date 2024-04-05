use rhdl_bits::{Bits, SignedBits};

use crate::{Kind, NoteKey, NoteWriter, TypedBits};

use super::note::Notable;

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
///
///

pub trait KindBits: Copy + PartialEq + Sized + Clone + 'static + Notable {
    fn static_kind() -> Kind;
    fn bits() -> usize {
        Self::static_kind().bits()
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
    fn bin(self) -> Vec<bool>;
    fn typed_bits(self) -> TypedBits {
        TypedBits {
            bits: self.bin(),
            kind: self.kind(),
        }
    }
    fn discriminant(self) -> TypedBits {
        self.typed_bits()
    }
    fn variant_kind(self) -> Kind {
        self.kind()
    }
    fn binary_string(self) -> String {
        self.bin()
            .iter()
            .rev()
            .map(|b| if *b { '1' } else { '0' })
            .collect()
    }
}

pub trait Digital: KindBits + Default {}

pub trait Control: KindBits {}

impl KindBits for () {
    fn static_kind() -> Kind {
        Kind::Empty
    }
    fn bin(self) -> Vec<bool> {
        Vec::new()
    }
}

impl Digital for () {}

impl Control for () {}

impl Notable for () {
    fn note(&self, _key: impl NoteKey, _writer: impl NoteWriter) {}
}

impl KindBits for bool {
    fn static_kind() -> Kind {
        Kind::make_bits(1)
    }
    fn bin(self) -> Vec<bool> {
        vec![self]
    }
}

impl Notable for bool {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_bool(key, *self);
    }
}

impl Digital for bool {}

impl Control for bool {}

impl KindBits for u8 {
    fn static_kind() -> Kind {
        Kind::make_bits(8)
    }
    fn bin(self) -> Vec<bool> {
        Bits::<8>::from(self as u128).to_bools()
    }
}

impl Digital for u8 {}

impl Notable for u8 {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_bits(key, *self as u128, 8);
    }
}

impl KindBits for u16 {
    fn static_kind() -> Kind {
        Kind::make_bits(16)
    }
    fn bin(self) -> Vec<bool> {
        Bits::<16>::from(self as u128).to_bools()
    }
}

impl Digital for u16 {}

impl Notable for u16 {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_bits(key, *self as u128, 16);
    }
}

impl KindBits for usize {
    fn static_kind() -> Kind {
        Kind::make_bits(usize::BITS as usize)
    }
    fn bin(self) -> Vec<bool> {
        Bits::<{ usize::BITS as usize }>::from(self as u128).to_bools()
    }
}

impl Digital for usize {}

impl Notable for usize {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_bits(key, *self as u128, usize::BITS as u8);
    }
}

impl KindBits for u128 {
    fn static_kind() -> Kind {
        Kind::make_bits(128)
    }
    fn bin(self) -> Vec<bool> {
        Bits::<128>::from(self).to_bools()
    }
}

impl Digital for u128 {}

impl Notable for u128 {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_bits(key, *self, 128);
    }
}

impl KindBits for i128 {
    fn static_kind() -> Kind {
        Kind::make_signed(128)
    }
    fn bin(self) -> Vec<bool> {
        SignedBits::<128>::from(self).as_unsigned().to_bools()
    }
}

impl Digital for i128 {}

impl Notable for i128 {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_signed(key, *self, 128);
    }
}

impl KindBits for i32 {
    fn static_kind() -> Kind {
        Kind::Signed(32)
    }
    fn bin(self) -> Vec<bool> {
        SignedBits::<32>::from(self as i128)
            .as_unsigned()
            .to_bools()
    }
}

impl Digital for i32 {}

impl Notable for i32 {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_signed(key, *self as i128, 32);
    }
}

impl KindBits for i8 {
    fn static_kind() -> Kind {
        Kind::Signed(8)
    }
    fn bin(self) -> Vec<bool> {
        SignedBits::<8>::from(self as i128).as_unsigned().to_bools()
    }
}

impl Digital for i8 {}

impl Notable for i8 {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_signed(key, *self as i128, 8);
    }
}

impl KindBits for i64 {
    fn static_kind() -> Kind {
        Kind::Signed(64)
    }
    fn bin(self) -> Vec<bool> {
        SignedBits::<64>::from(self as i128)
            .as_unsigned()
            .to_bools()
    }
}

impl Digital for i64 {}

impl Notable for i64 {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_signed(key, *self as i128, 64);
    }
}

impl<const N: usize> KindBits for Bits<N> {
    fn static_kind() -> Kind {
        Kind::make_bits(N)
    }
    fn bin(self) -> Vec<bool> {
        self.to_bools()
    }
}

impl<const N: usize> Digital for Bits<N> {}

impl Control for Bits<0> {}

impl Control for Bits<1> {}

impl<const N: usize> Notable for Bits<N> {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_bits(key, self.raw(), N as u8);
    }
}

impl<const N: usize> KindBits for SignedBits<N> {
    fn static_kind() -> Kind {
        Kind::make_signed(N)
    }
    fn bin(self) -> Vec<bool> {
        self.as_unsigned().to_bools()
    }
}

impl<const N: usize> Digital for SignedBits<N> {}

impl<const N: usize> Notable for SignedBits<N> {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_signed(key, self.raw(), N as u8);
    }
}

// Add blanket implementation for tuples up to size 4.
impl<T0: KindBits> KindBits for (T0,) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![T0::static_kind()])
    }
    fn bin(self) -> Vec<bool> {
        self.0.bin()
    }
}

impl<T0: Digital> Digital for (T0,) {}

impl<T0: Control> Control for (T0,) {}

impl<T0: Notable> Notable for (T0,) {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.0.note((key, ".0"), &mut writer);
    }
}

impl<T0: KindBits, T1: KindBits> KindBits for (T0, T1) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![T0::static_kind(), T1::static_kind()])
    }
    fn bin(self) -> Vec<bool> {
        let mut v = self.0.bin();
        v.extend(self.1.bin());
        v
    }
}

impl<T0: Digital, T1: Digital> Digital for (T0, T1) {}

impl<T0: Control, T1: Control> Control for (T0, T1) {}

impl<T0: Notable, T1: Notable> Notable for (T0, T1) {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.0.note((key, ".0"), &mut writer);
        self.1.note((key, ".1"), &mut writer);
    }
}

impl<T0: KindBits, T1: KindBits, T2: KindBits> KindBits for (T0, T1, T2) {
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
}

impl<T0: Digital, T1: Digital, T2: Digital> Digital for (T0, T1, T2) {}

impl<T0: Control, T1: Control, T2: Control> Control for (T0, T1, T2) {}

impl<T0: Notable, T1: Notable, T2: Notable> Notable for (T0, T1, T2) {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.0.note((key, ".0"), &mut writer);
        self.1.note((key, ".1"), &mut writer);
        self.2.note((key, ".2"), &mut writer);
    }
}

impl<T0: KindBits, T1: KindBits, T2: KindBits, T3: KindBits> KindBits for (T0, T1, T2, T3) {
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
}

impl<T0: Digital, T1: Digital, T2: Digital, T3: Digital> Digital for (T0, T1, T2, T3) {}

impl<T0: Control, T1: Control, T2: Control, T3: Control> Control for (T0, T1, T2, T3) {}

impl<T0: Notable, T1: Notable, T2: Notable, T3: Notable> Notable for (T0, T1, T2, T3) {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.0.note((key, ".0"), &mut writer);
        self.1.note((key, ".1"), &mut writer);
        self.2.note((key, ".2"), &mut writer);
        self.3.note((key, ".3"), &mut writer);
    }
}

impl<T0: Notable, T1: Notable, T2: Notable, T3: Notable, T4: Notable> Notable
    for (T0, T1, T2, T3, T4)
{
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        self.0.note((key, ".0"), &mut writer);
        self.1.note((key, ".1"), &mut writer);
        self.2.note((key, ".2"), &mut writer);
        self.3.note((key, ".3"), &mut writer);
        self.4.note((key, ".4"), &mut writer);
    }
}

// Because of the way Rust works, we cannot simply use a const-generic
// array here.  Instead, we have to implement each size of array
// separately.  This is unfortunate, but it is the only way to
// make this work.

// The following macro makes it easier
macro_rules! impl_array {
    ($($N:literal),*) => {
        $(
            impl<T: KindBits> KindBits for [T; $N] {
                fn static_kind() -> Kind {
                    Kind::make_array(T::static_kind(), $N)
                }
                fn bin(self) -> Vec<bool> {
                    let mut v = Vec::new();
                    for x in self.iter() {
                        v.extend(x.bin());
                    }
                    v
                }
            }

            impl<T: Digital> Digital for [T; $N] {}

            impl<T: Control> Control for [T; $N] {}

            impl<T: Notable> Notable for [T; $N] {
                fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
                    for (i, x) in self.iter().enumerate() {
                        x.note((key, i), &mut writer);
                    }
                }
            }
        )*
    };
}

impl_array!(1, 2, 3, 4, 5, 6, 7, 8);

#[cfg(test)]
mod test {
    use std::iter::repeat;

    use super::*;
    use crate::types::kind::{DiscriminantAlignment, Variant};
    use rhdl_bits::alias::*;

    #[test]
    #[allow(dead_code)]
    fn test_digital_enum_with_payloads() {
        #[derive(Copy, Clone, PartialEq, Default)]
        enum Mixed {
            #[default]
            None,
            Bool(bool),
            Tuple(bool, Bits<3>),
            Array([bool; 3]),
            Strct {
                a: bool,
                b: Bits<3>,
            },
        }

        impl Digital for Mixed {}
        impl KindBits for Mixed {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    "Mixed",
                    vec![
                        Variant {
                            name: "None".to_string(),
                            discriminant: 0,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Bool".to_string(),
                            discriminant: 1,
                            kind: Kind::make_bits(1),
                        },
                        Variant {
                            name: "Tuple".to_string(),
                            discriminant: 2,
                            kind: Kind::make_tuple(vec![Kind::make_bits(1), Kind::make_bits(3)]),
                        },
                        Variant {
                            name: "Array".to_string(),
                            discriminant: 3,
                            kind: Kind::make_array(Kind::make_bits(1), 3),
                        },
                        Variant {
                            name: "Strct".to_string(),
                            discriminant: 4,
                            kind: Kind::make_struct(
                                "Mixed::Strct",
                                vec![
                                    Kind::make_field("a", Kind::make_bits(1)),
                                    Kind::make_field("b", Kind::make_bits(3)),
                                ],
                            ),
                        },
                    ],
                    Kind::make_discriminant_layout(
                        3,
                        DiscriminantAlignment::Lsb,
                        crate::types::kind::DiscriminantType::Unsigned,
                    ),
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
        }

        impl Notable for Mixed {
            fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
                match self {
                    Self::None => {
                        writer.write_string(key, stringify!(None));
                    }
                    Self::Bool(b) => {
                        writer.write_string(key, stringify!(Bool));
                        Notable::note(b, key, &mut writer);
                    }
                    Self::Tuple(b, c) => {
                        writer.write_string(key, stringify!(Tuple));
                        b.note((key, "b"), &mut writer);
                        c.note((key, "c"), &mut writer);
                    }
                    Self::Array([b, c, d]) => {
                        writer.write_string(key, stringify!(Array));
                        b.note(key, &mut writer);
                        c.note(key, &mut writer);
                        d.note(key, &mut writer);
                    }
                    Self::Strct { a, b } => {
                        writer.write_string(key, stringify!(Strct));
                        a.note(key, &mut writer);
                        b.note(key, &mut writer);
                    }
                }
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
        #[derive(Copy, Clone, PartialEq, Default)]
        enum State {
            #[default]
            Init,
            Boot,
            Running,
            Stop,
            Boom,
        }

        impl Digital for State {}
        impl KindBits for State {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    "State",
                    vec![
                        Variant {
                            name: "Init".to_string(),
                            discriminant: 0,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boot".to_string(),
                            discriminant: 1,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Running".to_string(),
                            discriminant: 2,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Stop".to_string(),
                            discriminant: 3,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boom".to_string(),
                            discriminant: 4,
                            kind: Kind::Empty,
                        },
                    ],
                    Kind::make_discriminant_layout(
                        3,
                        DiscriminantAlignment::Lsb,
                        crate::types::kind::DiscriminantType::Unsigned,
                    ),
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
        }

        impl Notable for State {
            fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
                match self {
                    Self::Init => writer.write_string(key, stringify!(Init)),
                    Self::Boot => writer.write_string(key, stringify!(Boot)),
                    Self::Running => writer.write_string(key, stringify!(Running)),
                    Self::Stop => writer.write_string(key, stringify!(Stop)),
                    Self::Boom => writer.write_string(key, stringify!(Boom)),
                }
            }
        }
        let val = State::Boom;
        assert_eq!(val.bin(), rhdl_bits::bits::<3>(4).to_bools());
        assert_eq!(
            val.kind(),
            Kind::make_enum(
                "State",
                vec![
                    Variant {
                        name: "Init".to_string(),
                        discriminant: 0,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boot".to_string(),
                        discriminant: 1,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Running".to_string(),
                        discriminant: 2,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Stop".to_string(),
                        discriminant: 3,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boom".to_string(),
                        discriminant: 4,
                        kind: Kind::Empty,
                    },
                ],
                Kind::make_discriminant_layout(
                    3,
                    DiscriminantAlignment::Lsb,
                    crate::types::kind::DiscriminantType::Unsigned,
                ),
            )
        );
    }

    #[test]
    fn test_typed_bits_cast() {
        let x = b8(0b1010_1010).typed_bits();
        assert!(x.unsigned_cast(4).is_err());
        assert!(x.unsigned_cast(9).is_ok());
        let x = b8(0b0010_1010).typed_bits();
        assert!(x.signed_cast(4).is_err());
        assert!(x.unsigned_cast(6).is_ok());
        let s = b8(0b1010_1010).as_signed().typed_bits();
        assert!(s.signed_cast(4).is_err());
        assert!(s.signed_cast(7).is_err());
        let s = b8(0b1110_1010).as_signed().typed_bits();
        assert!(s.signed_cast(7).is_ok());
    }

    #[test]
    fn test_typed_bits_i64_cast() {
        let x = s8(-6).typed_bits();
        let y = x.as_i64().unwrap();
        assert_eq!(y, -6);
    }
}
