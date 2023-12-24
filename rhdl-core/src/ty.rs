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
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    pub fn len(self) -> usize {
        match self {
            Bits::Usize => std::mem::size_of::<usize>() * 8,
            Bits::I128 | Bits::U128 => 128,
            Bits::Signed(width) | Bits::Unsigned(width) => width,
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
use crate::Kind;

// Start simple, modelling as in the Eli Bendersky example.
// https://eli.thegreenplace.net/2018/type-inference/
// Support for function types as a target for inference
// has been dropped for now.
#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub enum Ty {
    Var(TypeId),
    Const(Bits),
    Ref(Box<Ty>),
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
    Ty::Tuple(args)
}

pub fn ty_as_ref(t: Ty) -> Ty {
    Ty::Ref(Box::new(t))
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
    if let Ty::Ref(base) = base {
        return ty_named_field(base, field).map(|x| Ty::Ref(Box::new(x)));
    }
    let Ty::Struct(struct_) = base else {
        bail!("Expected struct type, got {:?}", base)
    };
    struct_
        .fields
        .get(field)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No field named {} in {:?}", field, base))
}

pub fn ty_unnamed_field(base: &Ty, field: usize) -> Result<Ty> {
    if let Ty::Ref(base) = base {
        return ty_unnamed_field(base, field).map(|x| Ty::Ref(Box::new(x)));
    }
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
        _ => bail!("Type must be a tuple or tuple struct"),
    }
}

pub fn ty_indexed_item(base: &Ty, index: usize) -> Result<Ty> {
    if let Ty::Ref(base) = base {
        return ty_indexed_item(base, index).map(|x| Ty::Ref(Box::new(x)));
    }
    let Ty::Array(elems) = base else {
        bail!("Type must be an array")
    };
    elems
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow!("Index {} out of bounds", index))
}

pub fn ty_array_base(base: &Ty) -> Result<Ty> {
    if let Ty::Ref(base) = base {
        return ty_array_base(base).map(|x| Ty::Ref(Box::new(x)));
    }
    let Ty::Array(elems) = base else {
        bail!("Type must be an array")
    };
    elems
        .get(0)
        .cloned()
        .ok_or_else(|| anyhow!("Array must have at least one element"))
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
            Ty::Ref(t) => write!(f, "&{}", t),
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
            _ => unimplemented!("No type conversion for kind: {:?}", value),
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
