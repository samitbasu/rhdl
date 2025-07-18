use internment::Intern;

use crate::rhdl_core::{
    TypedBits,
    bitx::BitX,
    error::{RHDLError, rhdl_error},
    rhif::spec::Member,
};

use super::{domain::Color, error::DynamicTypeError};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Array(Intern<Array>),
    Tuple(Intern<Tuple>),
    Struct(Intern<Struct>),
    Enum(Intern<Enum>),
    Bits(usize),
    Signed(usize),
    Signal(Intern<Kind>, Color),
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
            Kind::Struct(s) => write!(f, "{}", s.name),
            Kind::Enum(e) => write!(f, "{}", e.name),
            Kind::Bits(digits) => write!(f, "b{digits}"),
            Kind::Signed(digits) => write!(f, "s{digits}"),
            Kind::Empty => write!(f, "()"),
            Kind::Signal(kind, color) => write!(f, "{kind:?}@{color:?}"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Array {
    pub base: Box<Kind>,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tuple {
    pub elements: Vec<Kind>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Struct {
    pub name: Intern<String>,
    pub fields: Vec<Field>,
}

impl Struct {
    pub fn is_tuple_struct(&self) -> bool {
        self.fields.iter().any(|x| x.name.parse::<i32>().is_ok())
    }
    pub fn get_field_kind(&self, member: &Member) -> Result<Kind> {
        let field_name = match member {
            Member::Named(name) => name.clone(),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiscriminantAlignment {
    Msb,
    Lsb,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum DiscriminantType {
    Signed,
    Unsigned,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct DiscriminantLayout {
    pub width: usize,
    pub alignment: DiscriminantAlignment,
    pub ty: DiscriminantType,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Enum {
    pub name: Intern<String>,
    pub variants: Vec<Variant>,
    pub discriminant_layout: DiscriminantLayout,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Field {
    pub name: Intern<String>,
    pub kind: Kind,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Variant {
    pub name: Intern<String>,
    pub discriminant: i64,
    pub kind: Kind,
}

impl Variant {
    pub fn with_discriminant(self, discriminant: i64) -> Variant {
        Variant {
            discriminant,
            ..self
        }
    }
}

impl Kind {
    pub fn make_array(base: Kind, size: usize) -> Self {
        Self::Array(Intern::new(Array {
            base: Box::new(base),
            size,
        }))
    }
    pub fn make_tuple(elements: Vec<Kind>) -> Self {
        if elements.is_empty() {
            Kind::Empty
        } else {
            Self::Tuple(Intern::new(Tuple { elements }))
        }
    }
    pub fn make_field(name: &str, kind: Kind) -> Field {
        Field {
            name: name.to_string().into(),
            kind,
        }
    }
    pub fn make_variant(name: &str, kind: Kind, discriminant: i64) -> Variant {
        Variant {
            name: name.to_string().into(),
            discriminant,
            kind,
        }
    }
    pub fn make_struct(name: &str, fields: Vec<Field>) -> Self {
        Self::Struct(Intern::new(Struct {
            name: name.to_string().into(),
            fields,
        }))
    }
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
    pub fn make_enum(
        name: &str,
        variants: Vec<Variant>,
        discriminant_layout: DiscriminantLayout,
    ) -> Self {
        Self::Enum(Intern::new(Enum {
            name: name.to_string().into(),
            variants,
            discriminant_layout,
        }))
    }
    pub fn make_bool() -> Self {
        Self::Bits(1)
    }
    pub fn make_bits(digits: usize) -> Self {
        Self::Bits(digits)
    }
    pub fn make_signed(digits: usize) -> Self {
        Self::Signed(digits)
    }
    pub fn make_signal(kind: Kind, color: Color) -> Self {
        Self::Signal(Intern::new(kind), color)
    }
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
        }
    }
    pub fn pad(&self, bits: Vec<BitX>) -> Vec<BitX> {
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
    pub fn get_field_kind(&self, member: &Member) -> Result<Kind> {
        let field_name = match member {
            Member::Named(name) => name.clone(),
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
    pub fn get_tuple_kind(&self, ndx: usize) -> Result<Kind> {
        match self {
            Kind::Tuple(tuple) => Ok(tuple.elements[ndx]),
            _ => Err(rhdl_error(DynamicTypeError::NotATuple { kind: *self })),
        }
    }
    pub fn get_base_kind(&self) -> Result<Kind> {
        match self {
            Kind::Array(array) => Ok(*array.base.clone()),
            _ => Err(rhdl_error(DynamicTypeError::NotAnArray { kind: *self })),
        }
    }
    // Return a rust type-like name for the kind
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
        }
    }

    pub fn get_discriminant_kind(&self) -> Result<Kind> {
        let Kind::Enum(e) = &self else {
            return Err(rhdl_error(DynamicTypeError::NotAnEnum { kind: *self }));
        };
        match e.discriminant_layout.ty {
            DiscriminantType::Signed => Ok(Kind::Signed(e.discriminant_layout.width)),
            DiscriminantType::Unsigned => Ok(Kind::Bits(e.discriminant_layout.width)),
        }
    }

    pub fn lookup_variant(&self, discriminant_value: i64) -> Option<&Variant> {
        let Kind::Enum(e) = &self else {
            return None;
        };
        e.variants
            .iter()
            .find(|x| x.discriminant == discriminant_value)
    }

    pub fn lookup_variant_kind_by_name(&self, name: &str) -> Option<Kind> {
        let Kind::Enum(e) = &self else {
            return None;
        };
        let variant = e.variants.iter().find(|x| (*x.name) == name)?;
        Some(variant.kind)
    }

    // Note that we use Zero instead of X here because the partial initialization
    // prover cannot handle the early return logic properly.
    pub fn place_holder(&self) -> TypedBits {
        TypedBits {
            bits: std::iter::repeat_n(BitX::Zero, self.bits()).collect(),
            kind: *self,
        }
    }

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

    pub fn enum_template(&self, variant: &str) -> Result<TypedBits> {
        // Create an empty template for a variant.
        // Note that this would be `unsafe` in the sense that
        // an all-zeros value for the payload is not necessarily valid.
        // Thus, we assume that the caller will fill in the payload
        // with valid values.
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
        let all_bits = self.pad(discriminant_bits.bits);
        Ok(TypedBits {
            kind: *self,
            bits: all_bits,
        })
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Kind::Enum(_))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Kind::Empty => true,
            Kind::Tuple(t) => t.elements.is_empty(),
            _ => false,
        }
    }

    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            Kind::Array(_) | Kind::Tuple(_) | Kind::Struct(_) | Kind::Enum(_)
        )
    }

    pub fn is_signed(&self) -> bool {
        if self.is_signal() {
            self.signal_data().is_signed()
        } else {
            matches!(self, Kind::Signed(_))
        }
    }

    pub fn is_unsigned(&self) -> bool {
        if self.is_signal() {
            self.signal_data().is_unsigned()
        } else {
            matches!(self, Kind::Bits(_))
        }
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Kind::Bits(1))
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, Kind::Tuple(_))
    }

    pub fn is_tuple_struct(&self) -> bool {
        if let Kind::Struct(s) = self {
            s.fields.iter().all(|x| x.name.parse::<i32>().is_ok())
        } else {
            false
        }
    }

    pub fn is_signal(&self) -> bool {
        matches!(self, Kind::Signal(_, _))
    }

    pub fn signal_kind(&self) -> Option<Kind> {
        if let Kind::Signal(kind, _) = self {
            Some(**kind)
        } else {
            None
        }
    }

    pub fn signal_clock(&self) -> Option<Color> {
        if let Kind::Signal(_, color) = self {
            Some(*color)
        } else {
            None
        }
    }

    pub fn signal_data(&self) -> Kind {
        if let Kind::Signal(kind, _) = self {
            **kind
        } else {
            *self
        }
    }

    pub fn val(&self) -> Kind {
        self.signal_data()
    }

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
                crate::rhdl_core::DiscriminantAlignment::Msb,
                crate::rhdl_core::DiscriminantType::Unsigned,
            )
        {
            return false;
        }
        true
    }

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
                crate::rhdl_core::DiscriminantAlignment::Msb,
                crate::rhdl_core::DiscriminantType::Unsigned,
            )
        {
            return false;
        }
        true
    }

    pub fn svg(&self, name: &str) -> svg::Document {
        crate::core::svg_grid(self, name)
    }
}
