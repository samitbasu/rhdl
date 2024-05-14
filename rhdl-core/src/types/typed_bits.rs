use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::iter::repeat;

use crate::dyn_bit_manip::bits_shr_signed;
use crate::dyn_bit_manip::{
    bit_neg, bit_not, bits_and, bits_or, bits_shl, bits_shr, bits_xor, full_add, full_sub,
};
use crate::ClockColor;
use crate::Digital;
use crate::{
    path::{bit_range, Path},
    Kind,
};

use super::kind::Array;
use super::kind::Enum;
use super::kind::Struct;
use super::kind::Tuple;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct TypedBits {
    pub bits: Vec<bool>,
    pub kind: Kind,
}

impl From<i64> for TypedBits {
    fn from(mut val: i64) -> Self {
        let mut bits = Vec::new();
        for _ in 0..64 {
            bits.push(val & 1 != 0);
            val >>= 1;
        }
        TypedBits {
            bits,
            kind: Kind::make_signed(64),
        }
    }
}

impl TypedBits {
    pub const EMPTY: TypedBits = TypedBits {
        bits: Vec::new(),
        kind: Kind::Empty,
    };

    pub fn path(&self, path: &Path) -> anyhow::Result<TypedBits> {
        let (range, kind) = bit_range(self.kind.clone(), path)?;
        Ok(TypedBits {
            bits: self.bits[range].to_vec(),
            kind,
        })
    }
    pub fn splice(&self, path: &Path, value: TypedBits) -> anyhow::Result<TypedBits> {
        let (range, kind) = bit_range(self.kind.clone(), path)?;
        if kind != value.kind {
            bail!(
                "Cannot update {} with {} because they have different types",
                self,
                value
            );
        }
        let mut new_bits = self.bits.clone();
        new_bits.splice(range, value.bits.iter().cloned());
        Ok(TypedBits {
            bits: new_bits,
            kind: self.kind.clone(),
        })
    }
    pub fn discriminant(&self) -> anyhow::Result<TypedBits> {
        if self.kind.is_enum() {
            self.path(&Path::default().discriminant())
        } else {
            Ok(self.clone())
        }
    }
    pub fn unsigned_cast(&self, bits: usize) -> anyhow::Result<TypedBits> {
        if bits > self.kind.bits() {
            return Ok(TypedBits {
                bits: self
                    .bits
                    .clone()
                    .into_iter()
                    .chain(repeat(false))
                    .take(bits)
                    .collect(),
                kind: Kind::make_bits(bits),
            });
        }
        let (base, rest) = self.bits.split_at(bits);
        if rest.iter().any(|b| *b) {
            anyhow::bail!(
                "Unsigned cast failed: {} is not representable in {} bits",
                self,
                bits
            );
        }
        Ok(TypedBits {
            bits: base.to_vec(),
            kind: Kind::make_bits(bits),
        })
    }
    pub fn signed_cast(&self, bits: usize) -> anyhow::Result<TypedBits> {
        if bits > self.kind.bits() {
            let sign_bit = self.bits.last().cloned().unwrap_or_default();
            return Ok(TypedBits {
                bits: self
                    .bits
                    .clone()
                    .into_iter()
                    .chain(repeat(sign_bit))
                    .take(bits)
                    .collect(),
                kind: Kind::make_signed(bits),
            });
        }
        let (base, rest) = self.bits.split_at(bits);
        let new_sign_bit = base.last().cloned().unwrap_or_default();
        if rest.iter().any(|b| *b != new_sign_bit) {
            anyhow::bail!(
                "Signed cast failed: {} is not representable in {} bits",
                self,
                bits
            );
        }
        Ok(TypedBits {
            bits: base.to_vec(),
            kind: Kind::make_signed(bits),
        })
    }
    pub fn as_i64(&self) -> anyhow::Result<i64> {
        let tb64 = match &self.kind {
            Kind::Bits(_) => self.unsigned_cast(64)?,
            Kind::Signed(_) => self.signed_cast(64)?,
            _ => {
                bail!("Cannot cast {:?} to i64", self.kind)
            }
        };
        let mut ret: u64 = 0;
        for ndx in 0..64 {
            ret |= (tb64.bits[ndx] as u64) << ndx;
        }
        Ok(ret as i64)
    }
    pub fn any(&self) -> TypedBits {
        self.bits.iter().any(|b| *b).typed_bits()
    }
    pub fn all(&self) -> TypedBits {
        self.bits.iter().all(|b| *b).typed_bits()
    }
    pub fn as_signed(&self) -> Result<TypedBits> {
        if let Kind::Bits(ndx) = self.kind {
            Ok(TypedBits {
                bits: self.bits.clone(),
                kind: Kind::Signed(ndx),
            })
        } else {
            bail!("Cannot cast {:?} to signed", self.kind)
        }
    }
    pub fn as_unsigned(&self) -> Result<TypedBits> {
        if let Kind::Signed(ndx) = self.kind {
            Ok(TypedBits {
                bits: self.bits.clone(),
                kind: Kind::Bits(ndx),
            })
        } else {
            bail!("Cannot cast {:?} to unsigned", self.kind)
        }
    }
    pub fn sign_bit(&self) -> Result<TypedBits> {
        if self.kind.is_signed() {
            Ok(TypedBits {
                bits: vec![self.bits.last().cloned().unwrap_or_default()],
                kind: Kind::make_bits(1),
            })
        } else {
            bail!("Cannot get sign bit of {:?}", self.kind)
        }
    }
    pub fn xor(&self) -> TypedBits {
        self.bits.iter().fold(false, |a, b| a ^ b).typed_bits()
    }
    pub fn as_bool(&self) -> Result<bool> {
        if self.kind.is_bool() {
            Ok(self.bits[0])
        } else {
            bail!("Cannot cast {:?} to bool", self.kind)
        }
    }
    pub fn repeat(&self, count: usize) -> TypedBits {
        let my_len = self.bits.len();
        TypedBits {
            bits: self
                .bits
                .iter()
                .cloned()
                .cycle()
                .take(count * my_len)
                .collect(),
            kind: Kind::make_array(self.kind.clone(), count),
        }
    }
    pub fn get_bit(&self, index: usize) -> Result<TypedBits> {
        if index >= self.bits.len() {
            bail!(
                "Cannot get bit {} from {} because it only has {} bits",
                index,
                self,
                self.bits.len()
            );
        }
        Ok(TypedBits {
            bits: vec![self.bits[index]],
            kind: Kind::make_bits(1),
        })
    }
    pub fn set_bit(&self, index: usize, val: bool) -> Result<TypedBits> {
        if index >= self.bits.len() {
            bail!(
                "Cannot set bit {} in {} because it only has {} bits",
                index,
                self,
                self.bits.len()
            );
        }
        if self.kind.is_composite() {
            bail!("Cannot set bit {} in composite {}", index, self);
        }
        let mut new_bits = self.bits.clone();
        new_bits[index] = val;
        Ok(TypedBits {
            bits: new_bits,
            kind: self.kind.clone(),
        })
    }
    pub fn slice(&self, offset: usize, count: usize) -> Result<TypedBits> {
        if self.kind.is_composite() {
            bail!("Cannot slice composite {}", self);
        }
        if offset + count > self.bits.len() {
            bail!(
                "Cannot slice {} bits from {} because it only has {} bits",
                count,
                self,
                self.bits.len()
            );
        }
        Ok(TypedBits {
            bits: self.bits[offset..offset + count].to_vec(),
            kind: Kind::make_bits(count),
        })
    }
    pub fn with_clock(self, color: ClockColor) -> TypedBits {
        TypedBits {
            bits: self.bits,
            kind: Kind::make_signal(self.kind, color),
        }
    }
}

impl std::ops::Add<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn add(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot add {} and {} because they have different types",
                self,
                rhs
            );
        }
        Ok(TypedBits {
            bits: full_add(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Sub<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn sub(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot subtract {} and {} because they have different types",
                self,
                rhs
            );
        }
        Ok(TypedBits {
            bits: full_sub(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Not for TypedBits {
    type Output = Result<TypedBits>;

    fn not(self) -> Self::Output {
        if self.kind.is_composite() {
            bail!("Cannot negate composite {}", self);
        }
        Ok(TypedBits {
            bits: bit_not(&self.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::BitXor for TypedBits {
    type Output = Result<TypedBits>;

    fn bitxor(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot xor {} and {} because they have different types",
                self,
                rhs
            );
        }
        if self.kind.is_composite() {
            bail!("Cannot xor composite {}", self);
        }
        Ok(TypedBits {
            bits: bits_xor(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::BitAnd for TypedBits {
    type Output = Result<TypedBits>;

    fn bitand(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot and {} and {} because they have different types",
                self,
                rhs
            );
        }
        if self.kind.is_composite() {
            bail!("Cannot and composite {}", self);
        }
        Ok(TypedBits {
            bits: bits_and(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::BitOr for TypedBits {
    type Output = Result<TypedBits>;

    fn bitor(self, rhs: TypedBits) -> Self::Output {
        if self.kind != rhs.kind {
            bail!(
                "Cannot or {} and {} because they have different types",
                self,
                rhs
            );
        }
        if self.kind.is_composite() {
            bail!("Cannot or composite {}", self);
        }
        Ok(TypedBits {
            bits: bits_or(&self.bits, &rhs.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Neg for TypedBits {
    type Output = Result<TypedBits>;

    fn neg(self) -> Self::Output {
        if !self.kind.is_signed() {
            bail!("Only signed values can be negated: {}", self);
        }
        Ok(TypedBits {
            bits: bit_neg(&self.bits),
            kind: self.kind,
        })
    }
}

impl std::ops::Shl<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn shl(self, rhs: TypedBits) -> Self::Output {
        if self.kind.is_composite() {
            bail!("Cannot shift composite {}", self);
        }
        if !rhs.kind.is_unsigned() {
            bail!("Shift amount must be unsigned: {}", rhs);
        }
        let shift = rhs.as_i64()?;
        if shift >= self.bits.len() as i64 {
            bail!(
                "Shift amount {} is greater than the number of bits in {}",
                shift,
                self
            );
        }
        Ok(TypedBits {
            bits: bits_shl(&self.bits, shift),
            kind: self.kind,
        })
    }
}

impl std::ops::Shr<TypedBits> for TypedBits {
    type Output = Result<TypedBits>;

    fn shr(self, rhs: TypedBits) -> Self::Output {
        if self.kind.is_composite() {
            bail!("Cannot shift composite {}", self);
        }
        if !rhs.kind.is_unsigned() {
            bail!("Shift amount must be unsigned: {}", rhs);
        }
        let shift = rhs.as_i64()?;
        if shift >= self.bits.len() as i64 {
            bail!(
                "Shift amount {} is greater than the number of bits in {}",
                shift,
                self
            );
        }
        if self.kind.is_signed() {
            Ok(TypedBits {
                bits: bits_shr_signed(&self.bits, shift),
                kind: self.kind,
            })
        } else {
            Ok(TypedBits {
                bits: bits_shr(&self.bits, shift),
                kind: self.kind,
            })
        }
    }
}

impl std::cmp::PartialOrd for TypedBits {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.kind != other.kind {
            return None;
        }
        if self.kind.is_unsigned() {
            let mut a_as_u128 = 0;
            let mut b_as_u128 = 0;
            for ndx in 0..self.bits.len() {
                a_as_u128 |= (self.bits[ndx] as u128) << ndx;
                b_as_u128 |= (other.bits[ndx] as u128) << ndx;
            }
            a_as_u128.partial_cmp(&b_as_u128)
        } else {
            let mut a_as_i128 = 0;
            let mut b_as_i128 = 0;
            for ndx in 0..self.bits.len() {
                a_as_i128 |= (self.bits[ndx] as i128) << ndx;
                b_as_i128 |= (other.bits[ndx] as i128) << ndx;
            }
            let me_sign = self.bits.last().cloned().unwrap_or_default();
            let other_sign = other.bits.last().cloned().unwrap_or_default();
            for ndx in self.bits.len()..128 {
                a_as_i128 |= (me_sign as i128) << ndx;
                b_as_i128 |= (other_sign as i128) << ndx;
            }
            a_as_i128.partial_cmp(&b_as_i128)
        }
    }
}

impl std::fmt::Display for TypedBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_kind_with_bits(&self.kind, &self.bits, f)
    }
}

fn write_kind_with_bits(
    kind: &Kind,
    bits: &[bool],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    match kind {
        Kind::Array(array) => write_array(array, bits, f),
        Kind::Tuple(tuple) => write_tuple(tuple, bits, f),
        Kind::Struct(structure) => write_struct(structure, bits, f),
        Kind::Enum(enumerate) => write_enumerate(enumerate, bits, f),
        Kind::Bits(_) => write_bits(bits, f),
        Kind::Signed(_) => write_signed(bits, f),
        Kind::Empty => write!(f, "()"),
        Kind::Signal(base, color) => {
            write_kind_with_bits(base, bits, f)?;
            write!(f, "@{}", color)
        }
    }
}

fn interpret_bits_as_i64(bits: &[bool], signed: bool) -> i64 {
    // If the value is signed, then we sign extend it to 128 bits
    let value = if signed {
        let sign = bits.last().copied().unwrap_or_default();
        repeat(&sign)
            .take(128 - bits.len())
            .chain(bits.iter().rev())
            .fold(0_i128, |acc, b| (acc << 1) | (*b as i128))
    } else {
        bits.iter()
            .rev()
            .fold(0_u128, |acc, b| (acc << 1) | (*b as u128)) as i128
    };
    value as i64
}

fn write_enumerate(
    enumerate: &Enum,
    bits: &[bool],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let root_kind = Kind::Enum(enumerate.clone());
    let (range, kind) = bit_range(root_kind.clone(), &Path::default().discriminant()).unwrap();
    let discriminant_value = interpret_bits_as_i64(&bits[range], kind.is_signed());
    // Get the variant for this discriminant
    let variant = enumerate
        .variants
        .iter()
        .find(|v| v.discriminant == discriminant_value)
        .unwrap();
    write!(f, "{}::{}", enumerate.name, variant.name)?;
    let (payload_range, payload_kind) = bit_range(
        root_kind,
        &Path::default().payload_by_value(discriminant_value),
    )
    .unwrap();
    let payload = &bits[payload_range];
    write_kind_with_bits(&payload_kind, payload, f)
}

fn write_struct(
    structure: &Struct,
    bits: &[bool],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{} {{", structure.name)?;
    let root_kind = Kind::Struct(structure.clone());
    for (ndx, field) in structure.fields.iter().enumerate() {
        let (bit_range, sub_kind) =
            bit_range(root_kind.clone(), &Path::default().field(&field.name)).unwrap();
        let slice = &bits[bit_range];
        write!(f, "{}: ", field.name)?;
        write_kind_with_bits(&sub_kind, slice, f)?;
        if ndx < structure.fields.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "}}")
}

fn write_array(array: &Array, bits: &[bool], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    let root_kind = Kind::Array(array.clone());
    for ndx in 0..(array.size) {
        let (bit_range, sub_kind) =
            bit_range(root_kind.clone(), &Path::default().index(ndx)).unwrap();
        let slice = &bits[bit_range];
        write_kind_with_bits(&sub_kind, slice, f)?;
        if ndx < array.size - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "]")
}

fn write_bits(bits: &[bool], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if bits.len() == 1 {
        return write!(f, "{}", if bits[0] { "true" } else { "false" });
    }
    // We know that the bits array will fit into a u128.
    let val = bits
        .iter()
        .rev()
        .fold(0_u128, |acc, b| (acc << 1) | (*b as u128));
    write!(f, "{:x}_b{}", val, bits.len())
}

fn write_signed(bits: &[bool], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if bits.len() == 1 {
        return write!(f, "{}", if bits[0] { "-1" } else { "0" });
    }
    // We know that the bits array will fit into a i128.
    let bit_len = bits.len();
    let sign_bit = bits.last().cloned().unwrap_or_default();
    let val = repeat(&sign_bit)
        .take(128 - bit_len)
        .chain(bits.iter().rev())
        .fold(0_i128, |acc, b| (acc << 1_i128) | (*b as i128));
    write!(f, "{}_s{}", val, bits.len())
}

fn write_tuple(tuple: &Tuple, bits: &[bool], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "(")?;
    let root_kind = Kind::Tuple(tuple.clone());
    for ndx in 0..(tuple.elements.len()) {
        let (bit_range, sub_kind) =
            bit_range(root_kind.clone(), &Path::default().tuple_index(ndx)).unwrap();
        let slice = &bits[bit_range];
        write_kind_with_bits(&sub_kind, slice, f)?;
        if ndx < tuple.elements.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, ")")
}

#[cfg(test)]
mod tests {
    use crate::{
        util::id, Digital, DiscriminantAlignment, DiscriminantType, Kind, Notable, TypedBits,
    };

    #[test]
    fn test_typed_bits_add() {
        let a = 42_u8.typed_bits();
        let b = 196_u8.typed_bits();
        assert!(a < b);
        assert!(a <= b);
        assert!(b > a);
        assert!(b >= a);
        let c = (a + b).unwrap();
        assert_eq!(c, 238_u8.typed_bits());
    }

    #[test]
    fn test_display_typed_bits() {
        #[derive(Debug, Clone, PartialEq, Copy)]
        enum Baz {
            A(Bar),
            B { foo: Foo },
            C(u8),
        }

        impl Default for Baz {
            fn default() -> Self {
                Self::A(Default::default())
            }
        }

        impl Notable for Baz {
            fn note(&self, key: impl crate::NoteKey, writer: impl crate::NoteWriter) {
                todo!()
            }
        }

        impl Digital for Baz {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Baz)),
                    vec![
                        Kind::make_variant(
                            stringify!(A),
                            Kind::make_tuple(vec![<Bar as Digital>::static_kind()]),
                            0i64,
                        ),
                        Kind::make_variant(
                            stringify!(B),
                            Kind::make_struct(
                                stringify!(_Baz__B),
                                vec![Kind::make_field(
                                    stringify!(foo),
                                    <Foo as Digital>::static_kind(),
                                )],
                            ),
                            1i64,
                        ),
                        Kind::make_variant(
                            stringify!(C),
                            Kind::make_tuple(vec![<u8 as Digital>::static_kind()]),
                            2i64,
                        ),
                    ],
                    Kind::make_discriminant_layout(
                        2usize,
                        DiscriminantAlignment::Msb,
                        DiscriminantType::Unsigned,
                    ),
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind().pad(match self {
                    Self::A(_0) => {
                        let mut v = rhdl_bits::bits::<2usize>(0i64 as u128).to_bools();
                        v.extend(_0.bin());
                        v
                    }
                    Self::B { foo } => {
                        let mut v = rhdl_bits::bits::<2usize>(1i64 as u128).to_bools();
                        v.extend(foo.bin());
                        v
                    }
                    Self::C(_0) => {
                        let mut v = rhdl_bits::bits::<2usize>(2i64 as u128).to_bools();
                        v.extend(_0.bin());
                        v
                    }
                })
            }
            fn discriminant(self) -> TypedBits {
                match self {
                    Self::A(_0) => rhdl_bits::bits::<2usize>(0i64 as u128).typed_bits(),
                    Self::B { foo } => rhdl_bits::bits::<2usize>(1i64 as u128).typed_bits(),
                    Self::C(_0) => rhdl_bits::bits::<2usize>(2i64 as u128).typed_bits(),
                }
            }
            fn variant_kind(self) -> Kind {
                match self {
                    Self::A(_0) => Kind::make_tuple(vec![<Bar as Digital>::static_kind()]),
                    Self::B { foo } => Kind::make_struct(
                        stringify!(_Baz__B),
                        vec![Kind::make_field(
                            stringify!(foo),
                            <Foo as Digital>::static_kind(),
                        )],
                    ),
                    Self::C(_0) => Kind::make_tuple(vec![<u8 as Digital>::static_kind()]),
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Copy, Default)]
        struct Bar(u8, u8, bool);

        impl Notable for Bar {
            fn note(&self, key: impl crate::NoteKey, writer: impl crate::NoteWriter) {
                todo!()
            }
        }

        impl Digital for Bar {
            fn static_kind() -> Kind {
                Kind::make_struct(
                    "Bar",
                    vec![
                        Kind::make_field("0", Kind::Bits(8)),
                        Kind::make_field("1", Kind::Bits(8)),
                        Kind::make_field("2", Kind::Bits(1)),
                    ],
                )
            }
            fn bin(self) -> Vec<bool> {
                [self.0.bin(), self.1.bin(), self.2.bin()].concat()
            }
        }

        #[derive(Debug, Clone, PartialEq, Copy, Default)]
        struct Foo {
            a: u8,
            b: u8,
            c: bool,
        }
        impl Notable for Foo {
            fn note(&self, key: impl crate::NoteKey, writer: impl crate::NoteWriter) {
                todo!()
            }
        }

        impl Digital for Foo {
            fn static_kind() -> Kind {
                Kind::make_struct(
                    "Foo",
                    vec![
                        Kind::make_field("a", Kind::Bits(8)),
                        Kind::make_field("b", Kind::Bits(8)),
                        Kind::make_field("c", Kind::Bits(1)),
                    ],
                )
            }
            fn bin(self) -> Vec<bool> {
                [self.a.bin(), self.b.bin(), self.c.bin()].concat()
            }
        }

        let a = 0x47_u8.typed_bits();
        assert_eq!(format!("{}", a), "47_b8");
        let c = (0x12_u8, 0x80_u8, false).typed_bits();
        assert_eq!(format!("{}", c), "(12_b8, 80_b8, false)");
        let b = (-0x53_i32).typed_bits();
        assert_eq!(format!("{}", b), "-83_s32");
        let d = [1_u8, 3_u8, 4_u8].typed_bits();
        assert_eq!(format!("{}", d), "[1_b8, 3_b8, 4_b8]");
        let e = Foo {
            a: 0x47,
            b: 0x80,
            c: true,
        }
        .typed_bits();
        assert_eq!(format!("{}", e), "Foo {a: 47_b8, b: 80_b8, c: true}");
        let e = Bar(0x47, 0x80, true).typed_bits();
        assert_eq!(format!("{}", e), "Bar {0: 47_b8, 1: 80_b8, 2: true}");
        let d = [Bar(0x47, 0x80, true), Bar(0x42, 0x13, false)].typed_bits();
        assert_eq!(
            format!("{}", d),
            "[Bar {0: 47_b8, 1: 80_b8, 2: true}, Bar {0: 42_b8, 1: 13_b8, 2: false}]"
        );
        let h = Baz::A(Bar(0x47, 0x80, true)).typed_bits();
        assert_eq!(
            format!("{}", h),
            "rhdl_core::types::typed_bits::tests::Baz::A(Bar {0: 47_b8, 1: 80_b8, 2: true})"
        );
    }
}
