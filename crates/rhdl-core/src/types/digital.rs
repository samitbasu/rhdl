use std::marker::PhantomData;

use rhdl_bits::{BitWidth, Bits, SignedBits};

use crate::{
    DiscriminantAlignment, DiscriminantType, Kind, TypedBits,
    bitx::{BitX, bitx_vec},
    trace::bit::TraceBit,
};

use crate::const_max;

use super::kind::DiscriminantLayout;

use seq_macro::seq;

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
/// # Unit and Unit Struct
///
/// These are supported as zero sized types.  You can include them in your
/// data structures, and use them to enforce invariants at compile time.  They
/// will be removed by RHDL during synthesis.
///
/// # Fixed Sized Arrays
///
/// You can represent an array of `impl Digital` types provided the size of the
/// array is known at compile time.
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
/// # Structs, Tuples
///
/// These are all supported in `RHDL`.
///
pub trait Digital: Copy + PartialEq + Sized + Clone + 'static {
    /// Associated constant that gives the total number of bits needed to represent the value.
    const BITS: usize;
    /// Associated constant that gives the total number of bits needed to represent the value in a trace.
    const TRACE_BITS: usize = Self::BITS;
    /// Returns the [Kind] (run time type descriptor) of the value as a static method
    fn static_kind() -> Kind;
    /// Returns the [TraceType] (run time trace type descriptor) of the value as a static method
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        Self::static_kind().into()
    }
    /// Returns the number of bits needed to represent the value.
    fn bits() -> usize {
        Self::BITS
    }
    /// Returns the number of bits needed to represent the value in a trace.
    fn trace_bits() -> usize {
        Self::TRACE_BITS
    }
    /// Returns the [Kind] (run time type descriptor) of the value.
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
    /// Returns the [TraceType] (run time trace type descriptor) of the value.
    fn trace_type(&self) -> rhdl_trace_type::TraceType {
        Self::static_trace_type()
    }
    /// Returns the binary representation of the value as a vector of [BitX].
    fn bin(self) -> Box<[BitX]>;
    /// Returns the binary representation of the value as a vector of [TraceBit].
    fn trace(self) -> Box<[TraceBit]> {
        self.bin().into_iter().map(|b| b.into()).collect()
    }
    /// Returns the value as a [TypedBits], which includes both the bits and the kind.
    fn typed_bits(self) -> TypedBits {
        TypedBits {
            bits: self.bin().into(),
            kind: self.kind(),
        }
    }
    /// Returns the discriminant of the value as a [TypedBits].
    fn discriminant(self) -> TypedBits {
        self.typed_bits()
    }
    /// Returns the [Kind] of the variant if the value is an enum with variants.
    fn variant_kind(self) -> Kind {
        self.kind()
    }
    /// Returns the binary representation of the value as a string of '0', '1', and 'X'.
    fn binary_string(self) -> String {
        self.bin()
            .iter()
            .rev()
            .map(|b| {
                let b: char = (*b).into();
                b
            })
            .collect()
    }
    /// Returns a "don't care" value for the type.
    fn dont_care() -> Self;
}

impl<T: Digital> Digital for Option<T> {
    const BITS: usize = 1 + T::BITS;
    fn static_kind() -> Kind {
        Kind::make_enum(
            &format!("Option::<{}>", std::any::type_name::<T>()),
            vec![
                Kind::make_variant("None", Kind::Empty, 0),
                Kind::make_variant("Some", Kind::make_tuple(vec![T::static_kind()].into()), 1),
            ],
            DiscriminantLayout {
                width: 1,
                alignment: DiscriminantAlignment::Msb,
                ty: DiscriminantType::Unsigned,
            },
        )
    }
    fn bin(self) -> Box<[BitX]> {
        self.kind().pad(match self {
            Self::None => vec![BitX::Zero],
            Self::Some(t) => {
                let mut v = vec![BitX::One];
                v.extend(t.bin());
                v
            }
        })
    }
    fn discriminant(self) -> TypedBits {
        match self {
            Self::None => false.typed_bits(),
            Self::Some(_) => true.typed_bits(),
        }
    }
    fn variant_kind(self) -> Kind {
        match self {
            Self::None => Kind::Empty,
            Self::Some(x) => x.kind(),
        }
    }
    fn dont_care() -> Self {
        Self::None
    }
}

impl<O: Digital, E: Digital> Digital for Result<O, E> {
    const BITS: usize = 1 + const_max!(O::BITS, E::BITS);
    fn static_kind() -> Kind {
        Kind::make_enum(
            &format!(
                "Result::<{}, {}>",
                std::any::type_name::<O>(),
                std::any::type_name::<E>()
            ),
            vec![
                Kind::make_variant("Err", Kind::make_tuple(vec![E::static_kind()].into()), 0),
                Kind::make_variant("Ok", Kind::make_tuple(vec![O::static_kind()].into()), 1),
            ],
            Kind::make_discriminant_layout(
                1,
                DiscriminantAlignment::Msb,
                DiscriminantType::Unsigned,
            ),
        )
    }
    fn bin(self) -> Box<[BitX]> {
        self.kind().pad(match self {
            Self::Ok(o) => {
                let mut v = vec![BitX::One];
                v.extend(o.bin());
                v
            }
            Self::Err(e) => {
                let mut v = vec![BitX::Zero];
                v.extend(e.bin());
                v
            }
        })
    }
    fn discriminant(self) -> TypedBits {
        match self {
            Self::Ok(_) => true.typed_bits(),
            Self::Err(_) => false.typed_bits(),
        }
    }
    fn variant_kind(self) -> Kind {
        match self {
            Self::Ok(x) => x.kind(),
            Self::Err(x) => x.kind(),
        }
    }
    fn dont_care() -> Self {
        Self::Err(E::dont_care())
    }
}

impl Digital for () {
    const BITS: usize = 0;
    fn static_kind() -> Kind {
        Kind::Empty
    }
    fn bin(self) -> Box<[BitX]> {
        [].into()
    }
    fn dont_care() -> Self {}
}

impl Digital for bool {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::make_bits(1)
    }
    fn bin(self) -> Box<[BitX]> {
        [self.into()].into()
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl Digital for u128 {
    const BITS: usize = 128;
    fn static_kind() -> Kind {
        Kind::make_bits(128)
    }
    fn bin(self) -> Box<[BitX]> {
        bitx_vec(&Bits::<128>::from(self).to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl Digital for i128 {
    const BITS: usize = 128;
    fn static_kind() -> Kind {
        Kind::make_signed(128)
    }
    fn bin(self) -> Box<[BitX]> {
        bitx_vec(&SignedBits::<128>::from(self).as_unsigned().to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl Digital for usize {
    const BITS: usize = usize::BITS as usize;
    fn static_kind() -> Kind {
        Kind::make_bits(usize::BITS as usize)
    }
    fn bin(self) -> Box<[BitX]> {
        match usize::BITS {
            32 => bitx_vec(&Bits::<32>::from(self as u128).to_bools()),
            64 => bitx_vec(&Bits::<64>::from(self as u128).to_bools()),
            _ => panic!("Unsupported usize bit width"),
        }
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl<const N: usize> Digital for Bits<N>
where
    rhdl_bits::W<N>: BitWidth,
{
    const BITS: usize = N;
    fn static_kind() -> Kind {
        Kind::make_bits(N)
    }
    fn bin(self) -> Box<[BitX]> {
        bitx_vec(&self.to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl<const N: usize> Digital for SignedBits<N>
where
    rhdl_bits::W<N>: BitWidth,
{
    const BITS: usize = N;
    fn static_kind() -> Kind {
        Kind::make_signed(N)
    }
    fn bin(self) -> Box<[BitX]> {
        bitx_vec(&self.as_unsigned().to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

// Use the seq! macro to generate an implementation for a tuple of size N
macro_rules! impl_tuple_for_digital {
    ($size: expr) => {
        seq!(N in 0..$size {
            impl<
              #(T~N: Digital,)*
            > Digital for
            (
                #(T~N,)*
            ) {
                const BITS: usize = 0_usize #(+T~N::BITS)*;
                fn static_kind() -> Kind {
                    Kind::make_tuple(vec![
                        #(T~N::static_kind(),)*
                        ].into())
                }
                fn bin(self) -> Box<[BitX]> {
                    let mut v = Vec::with_capacity(Self::BITS);
                    #(
                        v.extend(self.N.bin());
                    )*
                    v.into()
                }
                fn discriminant(self) -> TypedBits {
                    // The discriminant of a tuple is the
                    // tuple of the discriminants.
                    let mut k = Vec::default();
                    let mut v = Vec::with_capacity(Self::BITS);
                    #(
                        let d = self.N.discriminant();
                        k.push(d.kind);
                        v.extend(d.bits);
                    )*
                    TypedBits {
                        kind: Kind::make_tuple(k.into()),
                        bits: v,
                    }
                }
                fn dont_care() -> Self {
                    (
                        #( T~N::dont_care(), )*
                    )
                }
            }
        });
    }
}

impl_tuple_for_digital!(1);
impl_tuple_for_digital!(2);
impl_tuple_for_digital!(3);
impl_tuple_for_digital!(4);
impl_tuple_for_digital!(5);
impl_tuple_for_digital!(6);
impl_tuple_for_digital!(7);
impl_tuple_for_digital!(8);
impl_tuple_for_digital!(9);
impl_tuple_for_digital!(10);
impl_tuple_for_digital!(11);
impl_tuple_for_digital!(12);

impl<T: Digital> Digital for PhantomData<T> {
    const BITS: usize = 0;

    fn static_kind() -> Kind {
        Kind::Empty
    }
    fn bin(self) -> Box<[BitX]> {
        [].into()
    }
    fn dont_care() -> Self {
        Self
    }
}

impl<T: Digital, const N: usize> Digital for [T; N] {
    const BITS: usize = T::BITS * N;
    fn static_kind() -> Kind {
        Kind::make_array(T::static_kind(), N)
    }
    fn bin(self) -> Box<[BitX]> {
        let mut v = Vec::with_capacity(Self::BITS);
        for x in self.iter() {
            v.extend(x.bin());
        }
        v.into()
    }
    fn dont_care() -> Self {
        [T::dont_care(); N]
    }
    fn discriminant(self) -> TypedBits {
        // The discriminant of an array is an array of
        // discriminants.
        if N == 0 {
            return TypedBits::EMPTY;
        }
        let kind = Kind::make_array(self[0].discriminant().kind, N);
        let mut v = Vec::with_capacity(Self::BITS);
        for x in self.iter() {
            v.extend(x.discriminant().bits)
        }
        TypedBits { kind, bits: v }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        rtt::test::kind_to_trace,
        types::kind::{DiscriminantAlignment, Variant},
    };
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
            Invalid,
        }

        impl Digital for Mixed {
            const BITS: usize = 7;
            fn static_kind() -> Kind {
                Kind::make_enum(
                    "Mixed",
                    vec![
                        Variant {
                            name: "None".to_string().into(),
                            discriminant: 0,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Bool".to_string().into(),
                            discriminant: 1,
                            kind: Kind::make_bits(1),
                        },
                        Variant {
                            name: "Tuple".to_string().into(),
                            discriminant: 2,
                            kind: Kind::make_tuple(
                                vec![Kind::make_bits(1), Kind::make_bits(3)].into(),
                            ),
                        },
                        Variant {
                            name: "Array".to_string().into(),
                            discriminant: 3,
                            kind: Kind::make_array(Kind::make_bits(1), 3),
                        },
                        Variant {
                            name: "Strct".to_string().into(),
                            discriminant: 4,
                            kind: Kind::make_struct(
                                "Mixed::Strct",
                                vec![
                                    Kind::make_field("a", Kind::make_bits(1)),
                                    Kind::make_field("b", Kind::make_bits(3)),
                                ]
                                .into(),
                            ),
                        },
                        Variant {
                            name: "Invalid".to_string().into(),
                            discriminant: 5,
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
            fn static_trace_type() -> rhdl_trace_type::TraceType {
                kind_to_trace(&Self::static_kind())
            }
            fn bin(self) -> Box<[BitX]> {
                let raw = match self {
                    Self::None => bitx_vec(&rhdl_bits::bits::<3>(0).to_bools()),
                    Self::Bool(b) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<3>(1).to_bools()).to_vec();
                        v.extend(b.bin());
                        v.into()
                    }
                    Self::Tuple(b, c) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<3>(2).to_bools()).to_vec();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v.into()
                    }
                    Self::Array([b, c, d]) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<3>(3).to_bools()).to_vec();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v.extend(d.bin());
                        v.into()
                    }
                    Self::Strct { a, b } => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<3>(4).to_bools()).to_vec();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v.into()
                    }
                    Self::Invalid => bitx_vec(&rhdl_bits::bits::<3>(5).to_bools()),
                };
                if raw.len() < self.kind().bits() {
                    let missing = self.kind().bits() - raw.len();
                    raw.into_iter()
                        .chain(std::iter::repeat_n(BitX::Zero, missing))
                        .collect()
                } else {
                    raw
                }
            }
            fn dont_care() -> Self {
                Self::default()
            }
        }

        assert_eq!(Mixed::BITS, Mixed::static_kind().bits());
        println!("{:?}", Mixed::None.bin());
        println!("{:?}", Mixed::Bool(true).bin());
        let svg = crate::svg_grid(&Mixed::static_kind(), "val");
        svg::save("mixed.svg", &svg).unwrap();
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
            Invalid,
        }
        impl Digital for State {
            const BITS: usize = 3;
            fn static_kind() -> Kind {
                Kind::make_enum(
                    "State",
                    vec![
                        Variant {
                            name: "Init".to_string().into(),
                            discriminant: 0,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boot".to_string().into(),
                            discriminant: 1,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Running".to_string().into(),
                            discriminant: 2,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Stop".to_string().into(),
                            discriminant: 3,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boom".to_string().into(),
                            discriminant: 4,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Invalid".to_string().into(),
                            discriminant: 5,
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
            fn bin(self) -> Box<[BitX]> {
                bitx_vec(&match self {
                    Self::Init => rhdl_bits::bits::<3>(0).to_bools(),
                    Self::Boot => rhdl_bits::bits::<3>(1).to_bools(),
                    Self::Running => rhdl_bits::bits::<3>(2).to_bools(),
                    Self::Stop => rhdl_bits::bits::<3>(3).to_bools(),
                    Self::Boom => rhdl_bits::bits::<3>(4).to_bools(),
                    Self::Invalid => rhdl_bits::bits::<3>(5).to_bools(),
                })
            }
            fn dont_care() -> Self {
                Self::default()
            }
        }

        let val = State::Boom;
        assert_eq!(val.bin(), bitx_vec(&rhdl_bits::bits::<3>(4).to_bools()));
        assert_eq!(
            val.kind(),
            Kind::make_enum(
                "State",
                vec![
                    Variant {
                        name: "Init".to_string().into(),
                        discriminant: 0,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boot".to_string().into(),
                        discriminant: 1,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Running".to_string().into(),
                        discriminant: 2,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Stop".to_string().into(),
                        discriminant: 3,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boom".to_string().into(),
                        discriminant: 4,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Invalid".to_string().into(),
                        discriminant: 5,
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

    #[test]
    fn test_result_discriminant() {
        let x: Result<b8, b8> = Ok(b8(5));
        assert_eq!(x.discriminant().bits, vec![BitX::One]);
        let x: Result<b8, b8> = Err(b8(5));
        assert_eq!(x.discriminant().bits, vec![BitX::Zero]);
    }

    #[test]
    fn test_option_discriminant() {
        let x: Option<b8> = Some(b8(5));
        assert_eq!(x.discriminant().bits, vec![BitX::One]);
        let x: Option<b8> = None;
        assert_eq!(x.discriminant().bits, vec![BitX::Zero]);
    }
}
