use serde::{Deserialize, Serialize};

/// This enum describes how bits are laid out in a trace file, which is
/// a proxy for the physical layout of the bits in the hardware.  As such,
/// it does not include empty types (which have no meaning), or signal types
/// which carry clock metadata.  Types that have multi-bit representations internal
/// to `rhdl`, such as tri-state busses, will have only a smaller number of bits
/// when represented in hardware, and this is encoded in the representation.
/// This data structure is broken out into a separate crate to make reuse and sharing feasible.
/// The best way to think of this is as a "symbol map" used to decode the hardware
/// signals.
///
/// The enum is marked non-exhaustive to allow for future expansion.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[non_exhaustive]
pub enum TraceType {
    Array(Array),
    Tuple(Tuple),
    Struct(Struct),
    Enum(Enum),
    Bits(usize),
    Signed(usize),
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
    pub fields: Vec<(String, TraceType)>,
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
