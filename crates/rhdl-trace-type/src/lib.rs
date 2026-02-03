#![warn(missing_docs)]
//! This crate defines the trace type (RTT) format used by RHDL for
//! exporting types used in tracing simulations.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The root trace type structure.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub enum RTT {
    /// A mapping from signal names to their trace type information.
    TraceInfo(BTreeMap<String, TraceType>),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[non_exhaustive]
/// The trace type enumeration - algebraic union of all supported types.
pub enum TraceType {
    /// Array type, e.g., [T; N]
    Array(Array),
    /// Tuple type, e.g., (T1, T2, ...)
    Tuple(Tuple),
    /// Struct type
    Struct(Struct),
    /// Enum type
    Enum(Enum),
    /// Unsigned bit vector of given width
    Bits(usize),
    /// Signed bit vector of given width
    Signed(usize),
    /// Signal carrying data of type T with given color
    Signal(Box<TraceType>, Color),
    /// Clock signal
    Clock,
    /// Reset signal
    Reset,
    /// Empty type (ZST)
    Empty,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
/// Array type information
pub struct Array {
    /// The base type of the array
    pub base: Box<TraceType>,
    /// The size of the array
    pub size: usize,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
/// Tuple type information
pub struct Tuple {
    /// The element types of the tuple
    pub elements: Box<[TraceType]>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
/// Struct type information
pub struct Struct {
    /// The name of the struct
    pub name: String,
    /// The fields of the struct
    pub fields: Box<[Field]>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
/// Struct field information
pub struct Field {
    /// The name of the field
    pub name: String,
    /// The type of the field
    pub ty: TraceType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
/// Alignment of enum discriminants
pub enum DiscriminantAlignment {
    /// Discriminant is stored in the most significant bits of the enum
    Msb,
    /// Discriminant is stored in the least significant bits of the enum
    Lsb,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
/// The type of enum discriminants
pub enum DiscriminantType {
    /// Signed discriminant
    Signed,
    /// Unsigned discriminant
    Unsigned,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
/// Layout information for enum discriminants
pub struct DiscriminantLayout {
    /// The width of the discriminant in bits
    pub width: usize,
    /// The alignment of the discriminant within the enum
    pub alignment: DiscriminantAlignment,
    /// The type of the discriminant (signed or unsigned)
    pub ty: DiscriminantType,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
/// Enum type information
pub struct Enum {
    /// The name of the enum
    pub name: String,
    /// The variants of the enum
    pub variants: Vec<Variant>,
    /// The layout information for the discriminant
    pub discriminant_layout: DiscriminantLayout,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
/// Variant information for enums
pub struct Variant {
    /// The name of the variant
    pub name: String,
    /// The discriminant value of the variant
    pub discriminant: i64,
    /// The type associated with the variant
    pub ty: TraceType,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[non_exhaustive]
#[allow(missing_docs)]
/// The color of a signal
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
}

/// Helper function to construct an array trace type
pub fn make_array(base: TraceType, size: usize) -> TraceType {
    TraceType::Array(Array {
        base: Box::new(base),
        size,
    })
}

/// Helper function to construct a tuple trace type
pub fn make_tuple(elements: Box<[TraceType]>) -> TraceType {
    if elements.is_empty() {
        TraceType::Empty
    } else {
        TraceType::Tuple(Tuple { elements })
    }
}

/// Helper function to construct a field trace type
pub fn make_field(name: &str, ty: TraceType) -> Field {
    Field {
        name: name.to_string(),
        ty,
    }
}

/// Helper function to construct a variant trace type
pub fn make_variant(name: &str, ty: TraceType, discriminant: i64) -> Variant {
    Variant {
        name: name.to_string(),
        ty,
        discriminant,
    }
}

/// Helper function to construct a signal trace type
pub fn make_signal(ty: TraceType, color: Color) -> TraceType {
    TraceType::Signal(Box::new(ty), color)
}

/// Helper function to construct a struct trace type
pub fn make_struct(name: &str, fields: Box<[Field]>) -> TraceType {
    TraceType::Struct(Struct {
        name: name.to_string(),
        fields,
    })
}

/// Helper function to construct a discriminant layout
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

/// Helper function to construct an enum trace type
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
