use internment::Intern;
use rhdl_trace_type as rtt;

use crate::{
    TypedBits,
    bitx::BitX,
    error::{RHDLError, rhdl_error},
    rhif::spec::Member,
    types::path::Path,
};

use super::{domain::Color, error::DynamicTypeError};

/// A run time type representation for RHDL types.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// An array type with a base type and size.
    Array(Intern<Array>),
    /// A tuple type with a list of element types.
    Tuple(Intern<Tuple>),
    /// A struct type with a name and fields.
    Struct(Intern<Struct>),
    /// An enum type with a name and variants.
    Enum(Intern<Enum>),
    /// A bit vector type with a specific width.
    Bits(usize),
    /// A signed integer type with a specific width.
    Signed(usize),
    /// A signal type with a specific kind and color.
    Signal(Intern<Kind>, Color),
    /// A Clock type
    Clock,
    /// A Reset type
    Reset,
    /// An empty type.
    Empty,
}

type Result<T> = std::result::Result<T, RHDLError>;

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Array(array) => write!(f, "[{:?}; {}]", array.base, array.size),
            Kind::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .iter()
                    .map(|x| format!("{x:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({elements})")
            }
            Kind::Struct(s) => {
                // Write struct as name {f1: k1, f2: k2, ...}
                write!(
                    f,
                    "{} {{ ",
                    s.name.replace("rhdl_core::types::domain::", "")
                )?;
                for (i, field) in s.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {:?}", field.name, field.kind)?;
                }
                write!(f, " }}")
            }
            Kind::Enum(e) => write!(f, "{}", e.name),
            Kind::Bits(digits) => write!(f, "b{digits}"),
            Kind::Signed(digits) => write!(f, "s{digits}"),
            Kind::Empty => write!(f, "()"),
            Kind::Signal(kind, color) => write!(f, "{kind:?}@{color:?}"),
            Kind::Clock => write!(f, "clk"),
            Kind::Reset => write!(f, "rst"),
        }
    }
}

/// An array type with a base type and size.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Array {
    /// The base type of the array.
    pub base: Intern<Kind>,
    /// The size of the array.
    pub size: usize,
}

/// A tuple type with a list of element types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tuple {
    /// The element types of the tuple.
    pub elements: Box<[Kind]>,
}

/// A struct type with a name and fields.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Struct {
    /// The name of the struct.
    pub name: Intern<String>,
    /// The fields of the struct.
    pub fields: Box<[Field]>,
}

impl Struct {
    /// Checks if the struct is a tuple struct.
    pub fn is_tuple_struct(&self) -> bool {
        self.fields.iter().any(|x| x.name.parse::<i32>().is_ok())
    }
    /// Gets the kind of a field by its member representation.
    ///
    /// # Errors
    ///
    /// - Returns an error if the field does not exist in the struct.
    ///
    pub fn get_field_kind(&self, member: &Member) -> Result<Kind> {
        let field_name = match member {
            Member::Named(name) => *name,
            Member::Unnamed(ndx) => ndx.to_string().into(),
        };
        let field = self.fields.iter().find(|x| x.name == field_name);
        match field {
            Some(field) => Ok(field.kind),
            None => Err(rhdl_error(DynamicTypeError::NoFieldInStruct {
                kind: Kind::Struct(Intern::new(self.clone())),
                field_name,
            })),
        }
    }
}

/// Alignment of the discriminant in an enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiscriminantAlignment {
    /// The most significant bits are the discriminant.
    Msb,
    /// The least significant bits are the discriminant.
    Lsb,
}

/// The signedness of the discriminant in an enum.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum DiscriminantType {
    /// The discriminant is signed.
    Signed,
    /// The discriminant is unsigned.
    Unsigned,
}

/// Layout information for the discriminant in an enum.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct DiscriminantLayout {
    /// The width of the discriminant in bits.
    pub width: usize,
    /// The alignment of the discriminant.
    pub alignment: DiscriminantAlignment,
    /// The type of the discriminant.
    pub ty: DiscriminantType,
}

/// An enum type with a name, variants, and discriminant layout.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Enum {
    /// The name of the enum.
    pub name: Intern<String>,
    /// The variants of the enum.
    pub variants: Box<[Variant]>,
    /// The layout of the discriminant.
    pub discriminant_layout: DiscriminantLayout,
}

/// A field in a struct with a name and kind.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Field {
    /// The name of the field.
    pub name: Intern<String>,
    /// The kind of the field.
    pub kind: Kind,
}

/// A variant in an enum with a name, discriminant, and kind.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Variant {
    /// The name of the variant.
    pub name: Intern<String>,
    /// The discriminant of the variant.
    pub discriminant: i64,
    /// The kind of the variant.
    pub kind: Kind,
}

impl Variant {
    /// Returns a new `Variant` with the specified discriminant.
    pub fn with_discriminant(self, discriminant: i64) -> Variant {
        Variant {
            discriminant,
            ..self
        }
    }
}

impl Kind {
    /// Creates a new array kind.
    pub fn make_array(base: Kind, size: usize) -> Self {
        Self::Array(Intern::new(Array {
            base: Intern::new(base),
            size,
        }))
    }
    /// Creates a new tuple kind.
    pub fn make_tuple(elements: Box<[Kind]>) -> Self {
        if elements.is_empty() {
            Kind::Empty
        } else {
            Self::Tuple(Intern::new(Tuple { elements }))
        }
    }
    /// Creates a new field with the specified name and kind.
    pub fn make_field(name: &str, kind: Kind) -> Field {
        Field {
            name: name.to_string().into(),
            kind,
        }
    }
    /// Creates a new variant with the specified name, kind, and discriminant.
    pub fn make_variant(name: &str, kind: Kind, discriminant: i64) -> Variant {
        Variant {
            name: name.to_string().into(),
            discriminant,
            kind,
        }
    }
    /// Creates a new struct kind with the specified name and fields.
    pub fn make_struct(name: &str, fields: Box<[Field]>) -> Self {
        Self::Struct(Intern::new(Struct {
            name: name.to_string().into(),
            fields,
        }))
    }
    /// Creates a new discriminant layout with the specified width, alignment, and type.
    pub fn make_discriminant_layout(
        width: usize,
        alignment: DiscriminantAlignment,
        ty: DiscriminantType,
    ) -> DiscriminantLayout {
        DiscriminantLayout {
            width,
            alignment,
            ty,
        }
    }
    /// Creates a new enum kind with the specified name, variants, and discriminant layout.
    pub fn make_enum(
        name: &str,
        variants: Vec<Variant>,
        discriminant_layout: DiscriminantLayout,
    ) -> Self {
        Self::Enum(Intern::new(Enum {
            name: name.to_string().into(),
            variants: variants.into(),
            discriminant_layout,
        }))
    }
    /// Creates a new boolean kind.
    pub fn make_bool() -> Self {
        Self::Bits(1)
    }
    /// Creates a new bits kind with the specified number of digits.
    pub fn make_bits(digits: usize) -> Self {
        Self::Bits(digits)
    }
    /// Creates a new signed kind with the specified number of digits.
    pub fn make_signed(digits: usize) -> Self {
        Self::Signed(digits)
    }
    /// Creates a new signal kind with the specified kind and color.
    pub fn make_signal(kind: Kind, color: Color) -> Self {
        Self::Signal(Intern::new(kind), color)
    }
    /// Returns the number of bits required to represent the kind.
    pub fn bits(&self) -> usize {
        match self {
            Kind::Array(array) => array.base.bits() * array.size,
            Kind::Tuple(tuple) => tuple.elements.iter().map(|x| x.bits()).sum(),
            Kind::Struct(kind) => kind.fields.iter().map(|x| x.kind.bits()).sum(),
            Kind::Enum(kind) => {
                kind.discriminant_layout.width
                    + kind
                        .variants
                        .iter()
                        .map(|x| x.kind.bits())
                        .max()
                        .unwrap_or(0)
            }
            Kind::Bits(digits) => *digits,
            Kind::Signed(digits) => *digits,
            Kind::Signal(kind, _) => kind.bits(),
            Kind::Empty => 0,
            Kind::Clock | Kind::Reset => 1,
        }
    }
    /// Pads the given bits to match the kind's bit width.
    ///
    /// If the kind is an enum, then will pad according to the discriminant alignment.
    ///  - If the number of bits is less than the kind's bit width, it pads with zeros.
    ///  - If the number of bits is equal to the kind's bit width, it returns the bits as is.
    ///  - If the alignment is LSB, then the value is MSB-padded with zeros
    ///  - if the alignment is MSB, then the value is zero padded in the middle, between the discriminant and payload.
    ///
    /// # Panics
    ///
    /// - Panics if the number of bits exceeds the kind's bit width.
    pub fn pad(&self, bits: Vec<BitX>) -> Box<[BitX]> {
        if bits.len() > self.bits() {
            panic!("Too many bits for kind!");
        }
        let pad_len = self.bits() - bits.len();
        let bits = bits
            .into_iter()
            .chain(std::iter::repeat_n(BitX::Zero, pad_len));
        match self {
            Kind::Enum(kind) => match kind.discriminant_layout.alignment {
                DiscriminantAlignment::Lsb => bits.collect(),
                DiscriminantAlignment::Msb => {
                    let discriminant_width = kind.discriminant_layout.width;
                    let discriminant = bits.clone().take(discriminant_width);
                    let payload = bits.skip(discriminant_width);
                    payload.chain(discriminant).collect()
                }
            },
            _ => bits.collect(),
        }
    }
    /// Gets the kind of a field by its member representation.
    ///
    /// # Errors
    /// - Returns an error if the kind is not a struct or if the field does not exist.
    ///
    pub fn get_field_kind(&self, member: &Member) -> Result<Kind> {
        let field_name = match member {
            Member::Named(name) => *name,
            Member::Unnamed(ndx) => ndx.to_string().into(),
        };
        match self {
            Kind::Struct(s) => {
                let field = s.fields.iter().find(|x| x.name == field_name);
                match field {
                    Some(field) => Ok(field.kind),
                    None => Err(rhdl_error(DynamicTypeError::NoFieldInStruct {
                        kind: *self,
                        field_name,
                    })),
                }
            }
            _ => Err(rhdl_error(DynamicTypeError::NotAStruct { kind: *self })),
        }
    }
    /// Gets the kind of a tuple element by its index.
    /// # Errors
    /// - Returns an error if the kind is not a tuple.
    pub fn get_tuple_kind(&self, ndx: usize) -> Result<Kind> {
        match self {
            Kind::Tuple(tuple) => Ok(tuple.elements[ndx]),
            _ => Err(rhdl_error(DynamicTypeError::NotATuple { kind: *self })),
        }
    }
    /// Gets the base kind of an array.
    /// # Errors
    /// - Returns an error if the kind is not an array.
    pub fn get_base_kind(&self) -> Result<Kind> {
        match self {
            Kind::Array(array) => Ok(*array.base),
            _ => Err(rhdl_error(DynamicTypeError::NotAnArray { kind: *self })),
        }
    }
    /// Return a rust type-like name for the kind
    /// (e.g., b8, s16, [b8; 4], (b8, s16), MyStruct, MyEnum, etc.)
    pub fn get_name(&self) -> String {
        match self {
            Kind::Bits(digits) => format!("b{digits}"),
            Kind::Signed(digits) => format!("s{digits}"),
            Kind::Array(array) => format!("[{}; {}]", array.base.get_name(), array.size),
            Kind::Empty => "()".to_string(),
            Kind::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .iter()
                    .map(|x| x.get_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({elements})")
            }
            Kind::Struct(s) => (*s.name).clone(),
            Kind::Enum(e) => (*e.name).clone(),
            Kind::Signal(kind, color) => format!("Sig::<{kind:?},{color:?}>"),
            Kind::Clock => "Clock".to_string(),
            Kind::Reset => "Reset".to_string(),
        }
    }
    /// Gets the kind of the discriminant for an enum.
    /// # Errors
    /// - Returns an error if the kind is not an enum.
    pub fn get_discriminant_kind(&self) -> Result<Kind> {
        let Kind::Enum(e) = &self else {
            return Err(rhdl_error(DynamicTypeError::NotAnEnum { kind: *self }));
        };
        match e.discriminant_layout.ty {
            DiscriminantType::Signed => Ok(Kind::Signed(e.discriminant_layout.width)),
            DiscriminantType::Unsigned => Ok(Kind::Bits(e.discriminant_layout.width)),
        }
    }
    /// Looks up a variant by its discriminant value.
    /// Returns `None` if the kind is not an enum or if the variant does not exist.
    pub fn lookup_variant(&self, discriminant_value: i64) -> Option<&Variant> {
        let Kind::Enum(e) = &self else {
            return None;
        };
        e.variants
            .iter()
            .find(|x| x.discriminant == discriminant_value)
    }
    /// Looks up a variant by its name.
    /// Returns `None` if the kind is not an enum or if the variant does not exist.
    pub fn lookup_variant_kind_by_name(&self, name: &str) -> Option<Kind> {
        let Kind::Enum(e) = &self else {
            return None;
        };
        let variant = e.variants.iter().find(|x| (*x.name) == name)?;
        Some(variant.kind)
    }
    /// Creates a placeholder `TypedBits` with all bits set to `BitX::Zero`.
    /// The length of the bits matches the kind's bit width.
    ///
    /// Note that we use Zero instead of X here because the partial initialization
    /// prover cannot handle the early return logic properly.
    ///
    /// Note that this is not necessarily a valid value for the type!
    pub fn place_holder(&self) -> TypedBits {
        TypedBits::new(
            std::iter::repeat_n(BitX::Zero, self.bits()).collect(),
            *self,
        )
    }
    /// Gets the discriminant value for a variant by its name.
    /// # Errors
    /// - Returns an error if the kind is not an enum or if the variant does not exist.
    pub fn get_discriminant_for_variant_by_name(&self, variant: &str) -> Result<TypedBits> {
        let Kind::Enum(e) = &self else {
            return Err(rhdl_error(DynamicTypeError::NotAnEnum { kind: *self }));
        };
        let Some(variant_kind) = e.variants.iter().find(|x| (*x.name) == variant) else {
            return Err(rhdl_error(DynamicTypeError::NoVariantInEnum {
                name: variant.to_owned(),
                kind: *self,
            }));
        };
        let discriminant: TypedBits = variant_kind.discriminant.into();
        match e.discriminant_layout.ty {
            DiscriminantType::Signed => discriminant.signed_cast(e.discriminant_layout.width),
            DiscriminantType::Unsigned => discriminant.unsigned_cast(e.discriminant_layout.width),
        }
    }
    /// Creates a template `TypedBits` for a variant by its name.
    /// The template has the discriminant set to the variant's value
    /// and the payload bits set to `BitX::Zero`.
    /// The length of the bits matches the kind's bit width.
    /// # Errors
    /// - Returns an error if the kind is not an enum or if the variant does not exist.
    ///   Note that this is not necessarily a valid value for the type!
    pub fn enum_template(&self, variant: &str) -> Result<TypedBits> {
        let Kind::Enum(e) = &self else {
            return Err(rhdl_error(DynamicTypeError::NotAnEnum { kind: *self }));
        };
        let Some(variant_kind) = e.variants.iter().find(|x| (*x.name) == variant) else {
            return Err(rhdl_error(DynamicTypeError::NoVariantInEnum {
                name: variant.into(),
                kind: *self,
            }));
        };
        let discriminant: TypedBits = variant_kind.discriminant.into();
        let discriminant_bits = match e.discriminant_layout.ty {
            DiscriminantType::Signed => discriminant.signed_cast(e.discriminant_layout.width),
            DiscriminantType::Unsigned => discriminant.unsigned_cast(e.discriminant_layout.width),
        }?;
        let all_bits = self.pad(discriminant_bits.bits().to_vec());
        Ok(TypedBits::new(all_bits.to_vec(), *self))
    }
    /// Checks if the kind is an enum.
    pub fn is_enum(&self) -> bool {
        matches!(self, Kind::Enum(_))
    }
    /// Checks if the kind is empty (i.e., `Kind::Empty` or an empty tuple).
    pub fn is_empty(&self) -> bool {
        match self {
            Kind::Empty => true,
            Kind::Tuple(t) => t.elements.is_empty(),
            _ => false,
        }
    }
    /// Checks if the kind is composite (i.e., array, tuple, struct, or enum).
    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            Kind::Array(_) | Kind::Tuple(_) | Kind::Struct(_) | Kind::Enum(_)
        )
    }
    /// Checks if the kind is a signed bitvector or a signal of a signed bitvector.
    pub fn is_signed(&self) -> bool {
        if self.is_signal() {
            self.signal_data().is_signed()
        } else {
            matches!(self, Kind::Signed(_))
        }
    }
    /// Checks if the kind is an unsigned bitvector or a signal of an unsigned bitvector.
    pub fn is_unsigned(&self) -> bool {
        if self.is_signal() {
            self.signal_data().is_unsigned()
        } else {
            matches!(self, Kind::Bits(_))
        }
    }
    /// Checks if the kind is a boolean (i.e., `Kind::Bits(1)`).
    pub fn is_bool(&self) -> bool {
        matches!(self, Kind::Bits(1))
    }
    /// Checks if the kind is a tuple
    pub fn is_tuple(&self) -> bool {
        matches!(self, Kind::Tuple(_))
    }
    /// Checks if the kind is a tuple struct
    pub fn is_tuple_struct(&self) -> bool {
        if let Kind::Struct(s) = self {
            s.fields.iter().all(|x| x.name.parse::<i32>().is_ok())
        } else {
            false
        }
    }
    /// Checks if the kind is a signal
    pub fn is_signal(&self) -> bool {
        matches!(self, Kind::Signal(_, _))
    }
    /// If the kind is a signal, returns the data kind of the signal.
    ///
    /// I.e., if `Signal<foo, clk>` then returns `Some(foo)`.
    pub fn signal_kind(&self) -> Option<Kind> {
        if let Kind::Signal(kind, _) = self {
            Some(**kind)
        } else {
            None
        }
    }
    /// If the kind is a signal, returns the color of the signal.
    pub fn signal_clock(&self) -> Option<Color> {
        if let Kind::Signal(_, color) = self {
            Some(*color)
        } else {
            None
        }
    }
    /// If the kind is a signal, returns the data kind of the signal.
    /// Otherwise, returns the kind itself.
    pub fn signal_data(&self) -> Kind {
        if let Kind::Signal(kind, _) = self {
            **kind
        } else {
            *self
        }
    }
    /// If the kind is a signal, returns the data kind of the signal.
    /// Otherwise, returns the kind itself.
    pub fn val(&self) -> Kind {
        self.signal_data()
    }
    /// Checks if the kind is a `Result<T, E>` enum.
    pub fn is_result(&self) -> bool {
        let Kind::Enum(enumerate) = self else {
            return false;
        };
        if !enumerate.name.starts_with("Result::<")
            || !enumerate.variants.len() == 2
            || (*enumerate.variants[0].name) != "Err"
            || (*enumerate.variants[1].name) != "Ok"
        {
            return false;
        }
        if enumerate.discriminant_layout
            != Kind::make_discriminant_layout(
                1,
                crate::DiscriminantAlignment::Msb,
                crate::DiscriminantType::Unsigned,
            )
        {
            return false;
        }
        true
    }
    /// Checks if the kind is an `Option<T>` enum.
    pub fn is_option(&self) -> bool {
        let Kind::Enum(enumerate) = self else {
            return false;
        };
        if !enumerate.name.starts_with("Option::<")
            || !enumerate.variants.len() == 2
            || *enumerate.variants[0].name != "None"
            || *enumerate.variants[1].name != "Some"
        {
            return false;
        }
        if enumerate.discriminant_layout
            != Kind::make_discriminant_layout(
                1,
                crate::DiscriminantAlignment::Msb,
                crate::DiscriminantType::Unsigned,
            )
        {
            return false;
        }
        true
    }
    /// Generates an SVG representation of the kind.
    pub fn svg(&self, name: &str) -> svg::Document {
        super::svg::kind_svg::svg_grid(self, name)
    }
}

impl From<Kind> for rtt::TraceType {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Array(array) => rtt::make_array((*array.base).into(), array.size),
            Kind::Tuple(tuple) => {
                rtt::make_tuple(tuple.elements.iter().map(|x| (*x).into()).collect())
            }
            Kind::Struct(s) => rtt::make_struct(
                &s.name,
                s.fields
                    .iter()
                    .map(|x| rtt::make_field(&x.name, x.kind.into()))
                    .collect(),
            ),
            Kind::Enum(e) => rtt::make_enum(
                &e.name,
                e.variants
                    .iter()
                    .map(|x| rtt::make_variant(&x.name, x.kind.into(), x.discriminant))
                    .collect(),
                e.discriminant_layout.into(),
            ),
            Kind::Bits(digits) => rtt::TraceType::Bits(digits),
            Kind::Signed(digits) => rtt::TraceType::Signed(digits),
            Kind::Signal(kind, color) => rtt::make_signal((*kind).into(), color.into()),
            Kind::Empty => rtt::TraceType::Empty,
            Kind::Clock => rtt::TraceType::Clock,
            Kind::Reset => rtt::TraceType::Reset,
        }
    }
}
