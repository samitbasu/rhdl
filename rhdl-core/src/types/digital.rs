use rhdl_bits::{Bits, SignedBits};

use crate::{
    bitx::{bitx_vec, BitX},
    const_max,
    trace::bit::TraceBit,
    DiscriminantAlignment, DiscriminantType, Kind, TypedBits,
};

use super::kind::DiscriminantLayout;

use rhdl_trace_type as rtt;
use rhdl_typenum::*;

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
pub trait Digital: Copy + PartialEq + Sized + Clone + 'static {
    const BITS: usize;
    const TRACE_BITS: usize = Self::BITS;
    fn static_kind() -> Kind;
    fn static_trace_type() -> rhdl_trace_type::TraceType;
    fn bits() -> usize {
        Self::BITS
    }
    fn trace_bits() -> usize {
        Self::TRACE_BITS
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
    fn trace_type(&self) -> rhdl_trace_type::TraceType {
        Self::static_trace_type()
    }
    fn bin(self) -> Vec<BitX>;
    fn trace(self) -> Vec<TraceBit> {
        self.bin().into_iter().map(|b| b.into()).collect()
    }
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
            .map(|b| {
                let b: char = (*b).into();
                b
            })
            .collect()
    }
    fn dont_care() -> Self;
}

impl<T: Digital> Digital for Option<T> {
    const BITS: usize = 1 + T::BITS;
    fn static_kind() -> Kind {
        Kind::make_enum(
            &format!("Option::<{}>", std::any::type_name::<T>()),
            vec![
                Kind::make_variant("None", Kind::Empty, 0),
                Kind::make_variant("Some", Kind::make_tuple(vec![T::static_kind()]), 1),
            ],
            DiscriminantLayout {
                width: 1,
                alignment: DiscriminantAlignment::Msb,
                ty: DiscriminantType::Unsigned,
            },
        )
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::make_enum(
            &format!("Option::<{}>", std::any::type_name::<T>()),
            vec![
                rtt::make_variant("None", crate::TraceType::Empty, 0),
                rtt::make_variant("Some", rtt::make_tuple(vec![T::static_trace_type()]), 1),
            ],
            rtt::DiscriminantLayout {
                width: 1,
                alignment: rtt::DiscriminantAlignment::Msb,
                ty: rtt::DiscriminantType::Unsigned,
            },
        )
    }
    fn bin(self) -> Vec<BitX> {
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
                Kind::make_variant("Err", Kind::make_tuple(vec![E::static_kind()]), 0),
                Kind::make_variant("Ok", Kind::make_tuple(vec![O::static_kind()]), 1),
            ],
            Kind::make_discriminant_layout(
                1,
                DiscriminantAlignment::Msb,
                DiscriminantType::Unsigned,
            ),
        )
    }
    fn static_trace_type() -> rtt::TraceType {
        rtt::make_enum(
            &format!(
                "Result::<{}, {}>",
                std::any::type_name::<O>(),
                std::any::type_name::<E>()
            ),
            vec![
                rtt::make_variant("Err", rtt::make_tuple(vec![E::static_trace_type()]), 0),
                rtt::make_variant("Ok", rtt::make_tuple(vec![O::static_trace_type()]), 1),
            ],
            rtt::DiscriminantLayout {
                width: 1,
                alignment: rtt::DiscriminantAlignment::Msb,
                ty: rtt::DiscriminantType::Unsigned,
            },
        )
    }
    fn bin(self) -> Vec<BitX> {
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
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Empty
    }
    fn bin(self) -> Vec<BitX> {
        Vec::new()
    }
    fn dont_care() -> Self {}
}

impl Digital for bool {
    const BITS: usize = 1;
    fn static_kind() -> Kind {
        Kind::make_bits(1)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Bits(1)
    }
    fn bin(self) -> Vec<BitX> {
        vec![self.into()]
    }
    fn dont_care() -> Self {
        Self::default()
    }
}
/*

impl Digital for u64 {
    const BITS: usize = 64;
    fn static_kind() -> Kind {
        Kind::make_bits(64)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Bits(64)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&Bits::<64>::from(self as u128).to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl Digital for u8 {
    const BITS: usize = 8;
    fn static_kind() -> Kind {
        Kind::make_bits(8)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Bits(8)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&Bits::<8>::from(self as u128).to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl Digital for u16 {
    const BITS: usize = 16;
    fn static_kind() -> Kind {
        Kind::make_bits(16)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Bits(16)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&Bits::<16>::from(self as u128).to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}



impl Digital for i32 {
    const BITS: usize = 32;
    fn static_kind() -> Kind {
        Kind::Signed(32)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Signed(32)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(
            &SignedBits::<32>::from(self as i128)
                .as_unsigned()
                .to_bools(),
        )
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl Digital for i8 {
    const BITS: usize = 8;
    fn static_kind() -> Kind {
        Kind::Signed(8)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Signed(8)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&SignedBits::<8>::from(self as i128).as_unsigned().to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl Digital for i64 {
    const BITS: usize = 64;
    fn static_kind() -> Kind {
        Kind::Signed(64)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Signed(64)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(
            &SignedBits::<64>::from(self as i128)
                .as_unsigned()
                .to_bools(),
        )
    }
    fn dont_care() -> Self {
        Self::default()
    }
}
*/

impl Digital for u128 {
    const BITS: usize = 128;
    fn static_kind() -> Kind {
        Kind::make_bits(128)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Bits(128)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&Bits::<W128>::from(self).to_bools())
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
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Signed(128)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&SignedBits::<W128>::from(self).as_unsigned().to_bools())
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
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Bits(usize::BITS as usize)
    }
    fn bin(self) -> Vec<BitX> {
        match usize::BITS {
            32 => bitx_vec(&Bits::<W32>::from(self as u128).to_bools()),
            64 => bitx_vec(&Bits::<W64>::from(self as u128).to_bools()),
            _ => panic!("Unsupported usize bit width"),
        }
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl<N: BitWidth> Digital for Bits<N> {
    const BITS: usize = N::BITS;
    fn static_kind() -> Kind {
        Kind::make_bits(N::BITS)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Bits(N::BITS)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&self.to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

impl<N: BitWidth> Digital for SignedBits<N> {
    const BITS: usize = N::BITS;
    fn static_kind() -> Kind {
        Kind::make_signed(N::BITS)
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::TraceType::Signed(N::BITS)
    }
    fn bin(self) -> Vec<BitX> {
        bitx_vec(&self.as_unsigned().to_bools())
    }
    fn dont_care() -> Self {
        Self::default()
    }
}

// Add blanket implementation for tuples up to size 4.
impl<T0: Digital> Digital for (T0,) {
    const BITS: usize = T0::BITS;
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![T0::static_kind()])
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::make_tuple(vec![T0::static_trace_type()])
    }
    fn bin(self) -> Vec<BitX> {
        self.0.bin()
    }
    fn dont_care() -> Self {
        (T0::dont_care(),)
    }
}

impl<T0: Digital, T1: Digital> Digital for (T0, T1) {
    const BITS: usize = T0::BITS + T1::BITS;
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![T0::static_kind(), T1::static_kind()])
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::make_tuple(vec![T0::static_trace_type(), T1::static_trace_type()])
    }
    fn bin(self) -> Vec<BitX> {
        let mut v = Vec::with_capacity(Self::BITS);
        v.extend(self.0.bin());
        v.extend(self.1.bin());
        v
    }
    fn dont_care() -> Self {
        (T0::dont_care(), T1::dont_care())
    }
}

impl<T0: Digital, T1: Digital, T2: Digital> Digital for (T0, T1, T2) {
    const BITS: usize = T0::BITS + T1::BITS + T2::BITS;
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![
            T0::static_kind(),
            T1::static_kind(),
            T2::static_kind(),
        ])
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::make_tuple(vec![
            T0::static_trace_type(),
            T1::static_trace_type(),
            T2::static_trace_type(),
        ])
    }
    fn bin(self) -> Vec<BitX> {
        let mut v = Vec::with_capacity(Self::BITS);
        v.extend(self.0.bin());
        v.extend(self.1.bin());
        v.extend(self.2.bin());
        v
    }
    fn dont_care() -> Self {
        (T0::dont_care(), T1::dont_care(), T2::dont_care())
    }
}

impl<T0: Digital, T1: Digital, T2: Digital, T3: Digital> Digital for (T0, T1, T2, T3) {
    const BITS: usize = T0::BITS + T1::BITS + T2::BITS + T3::BITS;
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![
            T0::static_kind(),
            T1::static_kind(),
            T2::static_kind(),
            T3::static_kind(),
        ])
    }
    fn static_trace_type() -> rhdl_trace_type::TraceType {
        rtt::make_tuple(vec![
            T0::static_trace_type(),
            T1::static_trace_type(),
            T2::static_trace_type(),
            T3::static_trace_type(),
        ])
    }
    fn bin(self) -> Vec<BitX> {
        let mut v = Vec::with_capacity(Self::BITS);
        v.extend(self.0.bin());
        v.extend(self.1.bin());
        v.extend(self.2.bin());
        v.extend(self.3.bin());
        v
    }
    fn dont_care() -> Self {
        (
            T0::dont_care(),
            T1::dont_care(),
            T2::dont_care(),
            T3::dont_care(),
        )
    }
}

// macro to add digital trait for array of size N
// The following macro is used to generate the implementations for
// arrays of size N.  This is done because Rust does not allow
// for the implementation of traits for arrays of arbitrary size.
/* macro_rules! impl_array {
    ($N:expr) => {
        impl<T: Digital> Digital for [T; $N] {
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
            fn uninit() -> Self {
                [T::uninit(); $N]
            }
        }
    };
}

impl_array!(1);
impl_array!(2);
impl_array!(3);
impl_array!(4);
impl_array!(5);
impl_array!(6);
impl_array!(7);
impl_array!(8);
 */

impl<T: Digital, const N: usize> Digital for [T; N] {
    const BITS: usize = T::BITS * N;
    fn static_kind() -> Kind {
        Kind::make_array(T::static_kind(), N)
    }
    fn static_trace_type() -> rtt::TraceType {
        rtt::make_array(T::static_trace_type(), N)
    }
    fn bin(self) -> Vec<BitX> {
        let mut v = Vec::with_capacity(Self::BITS);
        for x in self.iter() {
            v.extend(x.bin());
        }
        v
    }
    fn dont_care() -> Self {
        [T::dont_care(); N]
    }
}

#[cfg(test)]
mod test {
    use std::iter::repeat;

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
            Tuple(bool, Bits<W3>),
            Array([bool; 3]),
            Strct {
                a: bool,
                b: Bits<W3>,
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
                        Variant {
                            name: "Invalid".to_string(),
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
            fn bin(self) -> Vec<BitX> {
                let raw = match self {
                    Self::None => bitx_vec(&rhdl_bits::bits::<W3>(0).to_bools()),
                    Self::Bool(b) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<W3>(1).to_bools());
                        v.extend(b.bin());
                        v
                    }
                    Self::Tuple(b, c) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<W3>(2).to_bools());
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v
                    }
                    Self::Array([b, c, d]) => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<W3>(3).to_bools());
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v.extend(d.bin());
                        v
                    }
                    Self::Strct { a, b } => {
                        let mut v = bitx_vec(&rhdl_bits::bits::<W3>(4).to_bools());
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                    Self::Invalid => bitx_vec(&rhdl_bits::bits::<W3>(5).to_bools()),
                };
                if raw.len() < self.kind().bits() {
                    let missing = self.kind().bits() - raw.len();
                    raw.into_iter()
                        .chain(repeat(BitX::Zero).take(missing))
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
            Invalid,
        }
        impl Digital for State {
            const BITS: usize = 3;
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
                        Variant {
                            name: "Invalid".to_string(),
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
            fn bin(self) -> Vec<BitX> {
                bitx_vec(&match self {
                    Self::Init => rhdl_bits::bits::<W3>(0).to_bools(),
                    Self::Boot => rhdl_bits::bits::<W3>(1).to_bools(),
                    Self::Running => rhdl_bits::bits::<W3>(2).to_bools(),
                    Self::Stop => rhdl_bits::bits::<W3>(3).to_bools(),
                    Self::Boom => rhdl_bits::bits::<W3>(4).to_bools(),
                    Self::Invalid => rhdl_bits::bits::<W3>(5).to_bools(),
                })
            }
            fn dont_care() -> Self {
                Self::default()
            }
        }

        let val = State::Boom;
        assert_eq!(val.bin(), bitx_vec(&rhdl_bits::bits::<W3>(4).to_bools()));
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
                    Variant {
                        name: "Invalid".to_string(),
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
