use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[non_exhaustive]
pub enum TraceType {
    Array(Array),
    Tuple(Tuple),
    Struct(Struct),
    Enum(Enum),
    Bits(usize),
    Signed(usize),
    Signal(Box<TraceType>, Color),
    Clock,
    Reset,
    Empty,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct Array {
    pub base: Box<TraceType>,
    pub size: usize,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct Tuple {
    pub elements: Vec<TraceType>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct Field {
    pub name: String,
    pub ty: TraceType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DiscriminantAlignment {
    Msb,
    Lsb,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DiscriminantType {
    Signed,
    Unsigned,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DiscriminantLayout {
    pub width: usize,
    pub alignment: DiscriminantAlignment,
    pub ty: DiscriminantType,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<Variant>,
    pub discriminant_layout: DiscriminantLayout,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct Variant {
    pub name: String,
    pub discriminant: i64,
    pub ty: TraceType,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[non_exhaustive]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
}

pub fn make_array(base: TraceType, size: usize) -> TraceType {
    TraceType::Array(Array {
        base: Box::new(base),
        size,
    })
}

pub fn make_tuple(elements: Vec<TraceType>) -> TraceType {
    if elements.is_empty() {
        TraceType::Empty
    } else {
        TraceType::Tuple(Tuple { elements })
    }
}

pub fn make_field(name: &str, ty: TraceType) -> Field {
    Field {
        name: name.to_string(),
        ty,
    }
}

pub fn make_variant(name: &str, ty: TraceType, discriminant: i64) -> Variant {
    Variant {
        name: name.to_string(),
        ty,
        discriminant,
    }
}

pub fn make_signal(ty: TraceType, color: Color) -> TraceType {
    TraceType::Signal(Box::new(ty), color)
}

pub fn make_struct(name: &str, fields: Vec<Field>) -> TraceType {
    TraceType::Struct(Struct {
        name: name.to_string(),
        fields,
    })
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
) -> TraceType {
    TraceType::Enum(Enum {
        name: name.to_string(),
        variants,
        discriminant_layout,
    })
}
