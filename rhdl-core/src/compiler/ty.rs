use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
// This module provides the type system for RHDL.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;

// First we define a type id - this is equivalent to the type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Bits {
    Clock(ClockColor),
    Signed(usize),
    Unsigned(usize),
    Usize,
    Empty,
}

impl Bits {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn len(&self) -> usize {
        match self {
            Bits::Usize => std::mem::size_of::<usize>() * 8,
            Bits::Signed(width) | Bits::Unsigned(width) => *width,
            Bits::Empty => 0,
            Bits::Clock(_) => 0,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub(super) struct TyMap {
    pub name: String,
    pub fields: BTreeMap<String, Ty>,
    pub kind: Kind,
}

#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub(super) struct TyEnum {
    pub payload: TyMap,
    pub discriminant: Box<Ty>,
}

use crate::ast::ast_impl::NodeId;
use crate::types::kind::DiscriminantLayout;
use crate::types::kind::DiscriminantType;
use crate::ClockColor;
use crate::Kind;

// Start simple, modelling as in the Eli Bendersky example.
// https://eli.thegreenplace.net/2018/type-inference/
// Support for function types as a target for inference
// has been dropped for now.
#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub(super) enum Ty {
    Var(TypeId),
    Const(Bits),
    Tuple(Vec<Ty>),
    Array(Vec<Ty>),
    Struct(TyMap),
    Enum(TyEnum),
    Signal(Box<Ty>, Box<Ty>),
    Integer,
}
impl Ty {
    pub fn is_empty(&self) -> bool {
        self == &ty_empty()
    }
    pub fn is_unsigned(&self) -> bool {
        matches!(self, Ty::Const(Bits::Unsigned(_)) | Ty::Const(Bits::Usize))
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Ty::Const(Bits::Unsigned(1)))
    }
    pub fn is_constant(&self) -> bool {
        matches!(self, Ty::Const(_))
    }
    pub fn unsigned_bits(&self) -> Result<usize> {
        match self {
            Ty::Const(Bits::Unsigned(width)) => Ok(*width),
            Ty::Const(Bits::Usize) => Ok(std::mem::size_of::<usize>() * 8),
            _ => bail!("Ty - Expected unsigned type, got {:?}", self),
        }
    }
    pub fn signed_bits(&self) -> Result<usize> {
        match self {
            Ty::Const(Bits::Signed(width)) => Ok(*width),
            Ty::Integer => Ok(32),
            _ => bail!("Ty - Expected signed type, got {:?}", self),
        }
    }
    pub fn is_signal(&self) -> bool {
        matches!(self, Ty::Signal(_, _))
    }
    pub fn try_clock(&self) -> Result<ClockColor> {
        match self {
            Ty::Const(Bits::Clock(color)) => Ok(*color),
            _ => bail!("Expected clock type, got {:?}", self),
        }
    }

    pub(crate) fn is_variable(&self) -> bool {
        matches!(self, Ty::Var(_))
    }
}

pub(super) fn ty_bool() -> Ty {
    Ty::Const(Bits::Unsigned(1))
}

pub(super) fn ty_empty() -> Ty {
    Ty::Const(Bits::Empty)
}

pub(super) fn ty_clock(color: ClockColor) -> Ty {
    Ty::Const(Bits::Clock(color))
}

pub(super) fn ty_bits(width: usize) -> Ty {
    Ty::Const(Bits::Unsigned(width))
}

pub(super) fn ty_signed(width: usize) -> Ty {
    Ty::Const(Bits::Signed(width))
}

pub(super) fn ty_array(t: Ty, len: usize) -> Ty {
    Ty::Array(vec![t; len])
}

pub(super) fn ty_tuple(args: Vec<Ty>) -> Ty {
    if args.is_empty() {
        return ty_empty();
    }
    Ty::Tuple(args)
}

pub(super) fn ty_signal(base: Ty, id: TypeId) -> Ty {
    Ty::Signal(Box::new(base), Box::new(ty_var(id.0 + 10_000)))
}

pub(super) fn ty_var(id: usize) -> Ty {
    Ty::Var(TypeId(id))
}

pub(super) fn ty_usize() -> Ty {
    Ty::Const(Bits::Usize)
}

pub(super) fn ty_integer() -> Ty {
    Ty::Integer
}

pub(super) fn ty_named_field(base: &Ty, field: &str) -> Result<Ty> {
    let Ty::Struct(struct_) = base else {
        bail!("Expected struct type, got {:?} for field {field}", base)
    };
    struct_
        .fields
        .get(field)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No field named {} in {:?}", field, base))
}

pub(super) fn ty_unnamed_field(base: &Ty, field: usize) -> Result<Ty> {
    // We can get an unnamed field from a tuple or a tuple struct
    match base {
        Ty::Tuple(fields_) => fields_
            .get(field)
            .cloned()
            .ok_or_else(|| anyhow!("Field {} not found", field)),
        Ty::Struct(struct_) => struct_
            .fields
            .get(&format!("{}", field))
            .cloned()
            .ok_or_else(|| anyhow!("Field {} not found in struct {}", field, struct_.name)),
        Ty::Array(elems) => elems
            .get(field)
            .cloned()
            .ok_or_else(|| anyhow!("Field {} not found in array", field)),
        _ => bail!("Type must be a tuple or tuple struct"),
    }
}

pub(super) fn ty_indexed_item(base: &Ty, index: usize) -> Result<Ty> {
    let Ty::Array(elems) = base else {
        bail!(format!("Type must be an array, got {:?}", base))
    };
    elems
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow!("Index {} out of bounds", index))
}

pub(super) fn ty_array_base(base: &Ty) -> Result<Ty> {
    let Ty::Array(elems) = base else {
        bail!("Type must be an array")
    };
    elems
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("Array must have at least one element"))
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Var(id) => write!(f, "V{}", id.0),
            Ty::Signal(ty, color) => write!(f, "{}{}", ty, color),
            Ty::Const(bits) => match bits {
                Bits::Usize => write!(f, "usize"),
                Bits::Signed(width) => write!(f, "s{}", width),
                Bits::Unsigned(width) => write!(f, "b{}", width),
                Bits::Empty => write!(f, "{{}}"),
                Bits::Clock(color) => write!(f, "@{}", color),
            },
            Ty::Struct(struct_) => {
                write!(f, "{} {{", struct_.name)?;
                for (i, (field, ty)) in struct_.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field, ty)?;
                }
                write!(f, "}}")
            }
            Ty::Enum(enum_) => {
                let struct_ = &enum_.payload;
                write!(f, "{} {{", struct_.name)?;
                for (i, (field, ty)) in struct_.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field, ty)?;
                }
                write!(f, "}}")
            }
            Ty::Tuple(fields) => {
                write!(f, "(")?;
                for (i, term) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", term)?;
                }
                write!(f, ")")
            }
            Ty::Array(elems) => {
                write!(f, "[")?;
                for (i, term) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", term)?;
                }
                write!(f, "]")
            }
            Ty::Integer => write!(f, "int"),
        }
    }
}

// Provide a conversion from Kind (which is a concrete type descriptor) to
// Ty (which can contain variables)
impl From<Kind> for Ty {
    fn from(value: Kind) -> Self {
        match value.clone() {
            Kind::Bits(width) => ty_bits(width),
            Kind::Signed(width) => ty_signed(width),
            Kind::Empty => ty_empty(),
            Kind::Struct(struct_) => Ty::Struct(TyMap {
                name: struct_.name,
                fields: struct_
                    .fields
                    .into_iter()
                    .map(|field| (field.name, field.kind.into()))
                    .collect(),
                kind: value,
            }),
            Kind::Tuple(fields) => Ty::Tuple(
                fields
                    .elements
                    .into_iter()
                    .map(|field| field.into())
                    .collect(),
            ),
            Kind::Enum(enum_) => Ty::Enum(TyEnum {
                payload: TyMap {
                    name: enum_.name,
                    fields: enum_
                        .variants
                        .into_iter()
                        .map(|variant| (variant.name, variant.kind.into()))
                        .collect(),
                    kind: value,
                },
                discriminant: Box::new(enum_.discriminant_layout.into()),
            }),
            Kind::Array(array) => ty_array((*array.base).into(), array.size),
            Kind::Signal(base, color) => {
                let base: Ty = (*base).into();
                Ty::Signal(Box::new(base), Box::new(ty_clock(color)))
            }
        }
    }
}

impl From<DiscriminantLayout> for Ty {
    fn from(x: DiscriminantLayout) -> Self {
        match x.ty {
            DiscriminantType::Signed => ty_signed(x.width),
            DiscriminantType::Unsigned => ty_bits(x.width),
        }
    }
}

// Going from a Ty to a Kind is fallible, since a Ty can contain variables.
impl TryFrom<Ty> for Kind {
    type Error = anyhow::Error;

    fn try_from(value: Ty) -> std::result::Result<Self, Self::Error> {
        match value {
            Ty::Var(_) => bail!("Cannot convert Ty::Var to Kind"),
            Ty::Const(bits) => match bits {
                Bits::Usize => Ok(Kind::Bits(std::mem::size_of::<usize>() * 8)),
                Bits::Signed(width) => Ok(Kind::Signed(width)),
                Bits::Unsigned(width) => Ok(Kind::Bits(width)),
                Bits::Empty => Ok(Kind::Empty),
                Bits::Clock(_) => bail!("Cannot convert Ty::Clock to Kind"),
            },
            Ty::Struct(struct_) => Ok(struct_.kind),
            Ty::Tuple(fields) => Ok(Kind::make_tuple(
                fields
                    .into_iter()
                    .map(|field| field.try_into())
                    .collect::<Result<_>>()?,
            )),
            Ty::Array(elems) => Ok(Kind::make_array(
                elems
                    .first()
                    .ok_or(anyhow!("Array must have at least one element"))?
                    .clone()
                    .try_into()?,
                elems.len(),
            )),
            Ty::Enum(enum_) => Ok(enum_.payload.kind),
            Ty::Signal(base, color) => {
                let base: Kind = (*base).try_into()?;
                Ok(Kind::Signal(Box::new(base), color.try_clock()?))
            }
            Ty::Integer => Ok(Kind::Signed(32)),
        }
    }
}
