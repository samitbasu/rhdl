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

impl From<TypeId> for crate::ast::NodeId {
    fn from(value: TypeId) -> Self {
        NodeId::new(value.0 as u32)
    }
}

impl From<crate::ast::NodeId> for TypeId {
    fn from(value: crate::ast::NodeId) -> Self {
        TypeId(value.as_u32() as usize)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Bits {
    Signed(usize),
    Unsigned(usize),
    I128,
    U128,
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
            Bits::I128 | Bits::U128 => 128,
            Bits::Signed(width) | Bits::Unsigned(width) => *width,
            Bits::Empty => 0,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub struct TyMap {
    pub name: String,
    pub fields: BTreeMap<String, Ty>,
    pub kind: Kind,
}

#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub struct TyEnum {
    pub payload: TyMap,
    pub discriminant: Box<Ty>,
}

use crate::ast::NodeId;
use crate::kind::DiscriminantLayout;
use crate::kind::DiscriminantType;
use crate::path::PathElement;
use crate::Kind;

// Start simple, modelling as in the Eli Bendersky example.
// https://eli.thegreenplace.net/2018/type-inference/
// Support for function types as a target for inference
// has been dropped for now.
#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub enum Ty {
    Var(TypeId),
    Const(Bits),
    Tuple(Vec<Ty>),
    Array(Vec<Ty>),
    Struct(TyMap),
    Enum(TyEnum),
    Integer,
}
impl Ty {
    pub fn is_empty(&self) -> bool {
        self == &ty_empty()
    }
    pub fn is_unsigned(&self) -> bool {
        matches!(
            self,
            Ty::Const(Bits::Unsigned(_)) | Ty::Const(Bits::U128) | Ty::Const(Bits::Usize)
        )
    }
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Ty::Const(Bits::Signed(_)) | Ty::Const(Bits::I128) | Ty::Integer
        )
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Ty::Const(Bits::Unsigned(1)))
    }
    pub fn unsigned_bits(&self) -> Result<usize> {
        match self {
            Ty::Const(Bits::Unsigned(width)) => Ok(*width),
            Ty::Const(Bits::U128) => Ok(128),
            Ty::Const(Bits::Usize) => Ok(std::mem::size_of::<usize>() * 8),
            _ => bail!("Ty - Expected unsigned type, got {:?}", self),
        }
    }
    pub fn signed_bits(&self) -> Result<usize> {
        match self {
            Ty::Const(Bits::Signed(width)) => Ok(*width),
            Ty::Const(Bits::I128) => Ok(128),
            Ty::Integer => Ok(32),
            _ => bail!("Ty - Expected signed type, got {:?}", self),
        }
    }
    pub fn bits(&self) -> usize {
        match self {
            Ty::Const(bits) => bits.len(),
            Ty::Tuple(fields) => fields.iter().map(|f| f.bits()).sum(),
            Ty::Array(elems) => elems.iter().map(|e| e.bits()).sum(),
            Ty::Struct(struct_) => struct_.kind.bits(),
            Ty::Enum(enum_) => enum_.payload.kind.bits(),
            Ty::Var(_) => 0,
            Ty::Integer => 32,
        }
    }
}

pub fn ty_bool() -> Ty {
    Ty::Const(Bits::Unsigned(1))
}

pub fn ty_empty() -> Ty {
    Ty::Const(Bits::Empty)
}

pub fn ty_bits(width: usize) -> Ty {
    Ty::Const(Bits::Unsigned(width))
}

pub fn ty_signed(width: usize) -> Ty {
    Ty::Const(Bits::Signed(width))
}

pub fn ty_array(t: Ty, len: usize) -> Ty {
    Ty::Array(vec![t; len])
}

pub fn ty_tuple(args: Vec<Ty>) -> Ty {
    if args.is_empty() {
        return ty_empty();
    }
    Ty::Tuple(args)
}

pub fn ty_var(id: usize) -> Ty {
    Ty::Var(TypeId(id))
}

pub fn ty_int() -> Ty {
    Ty::Const(Bits::I128)
}

pub fn ty_uint() -> Ty {
    Ty::Const(Bits::U128)
}

pub fn ty_usize() -> Ty {
    Ty::Const(Bits::Usize)
}

pub fn ty_integer() -> Ty {
    Ty::Integer
}

pub fn ty_named_field(base: &Ty, field: &str) -> Result<Ty> {
    let Ty::Struct(struct_) = base else {
        bail!("Expected struct type, got {:?} for field {field}", base)
    };
    struct_
        .fields
        .get(field)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No field named {} in {:?}", field, base))
}

pub fn ty_unnamed_field(base: &Ty, field: usize) -> Result<Ty> {
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

pub fn ty_indexed_item(base: &Ty, index: usize) -> Result<Ty> {
    let Ty::Array(elems) = base else {
        bail!(format!("Type must be an array, got {:?}", base))
    };
    elems
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow!("Index {} out of bounds", index))
}

pub fn ty_array_base(base: &Ty) -> Result<Ty> {
    let Ty::Array(elems) = base else {
        bail!("Type must be an array")
    };
    elems
        .get(0)
        .cloned()
        .ok_or_else(|| anyhow!("Array must have at least one element"))
}

pub fn ty_path(mut base: Ty, path: &crate::path::Path) -> Result<Ty> {
    for segment in &path.elements {
        match segment {
            PathElement::All => (),
            PathElement::Index(i) => base = ty_unnamed_field(&base, *i)?,
            PathElement::Field(field) => base = ty_named_field(&base, field)?,
            PathElement::EnumDiscriminant => {
                if let Ty::Enum(enum_) = base {
                    base = *enum_.discriminant.clone();
                } else {
                    bail!("Expected enum type, got {:?}", base)
                }
            }
            PathElement::EnumPayload(_string) => {
                if let Ty::Enum(enum_) = base {
                    base = Ty::Struct(enum_.payload.clone());
                } else {
                    bail!("Expected enum type, got {:?}", base)
                }
            }
            PathElement::EnumPayloadByValue(value) => {
                if let Ty::Enum(enum_) = base {
                    let kind = enum_.payload.kind.lookup_variant(*value)?;
                    base = kind.into();
                } else {
                    bail!("Expected enum type, got {:?}", base)
                }
            }
            PathElement::DynamicIndex(_index) => {
                base = ty_array_base(&base)?;
            }
        }
    }
    Ok(base)
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Var(id) => write!(f, "V{}", id.0),
            Ty::Const(bits) => match bits {
                Bits::I128 => write!(f, "i128"),
                Bits::U128 => write!(f, "u128"),
                Bits::Usize => write!(f, "usize"),
                Bits::Signed(width) => write!(f, "s{}", width),
                Bits::Unsigned(width) => write!(f, "b{}", width),
                Bits::Empty => write!(f, "{{}}"),
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
            Kind::I128 => Ty::Const(Bits::I128),
            Kind::U128 => Ty::Const(Bits::U128),
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
                Bits::I128 => Ok(Kind::I128),
                Bits::U128 => Ok(Kind::U128),
                Bits::Usize => Ok(Kind::Bits(std::mem::size_of::<usize>() * 8)),
                Bits::Signed(width) => Ok(Kind::Signed(width)),
                Bits::Unsigned(width) => Ok(Kind::Bits(width)),
                Bits::Empty => Ok(Kind::Empty),
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
            Ty::Integer => Ok(Kind::Signed(32)),
        }
    }
}
