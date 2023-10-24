// This module provides the type system for RHDL.
use anyhow::Result;
use anyhow::{anyhow, bail};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;

// First we define a type id - this is equivalent to the type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bits {
    Signed(usize),
    Unsigned(usize),
    Empty,
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct TyMap {
    pub name: String,
    pub fields: BTreeMap<String, Ty>,
}

// A simple macro rules style macro that allows you to construct a TermMap
// using a struct-like syntax, like
// term_map! {
//     name: "Foo",
//     fields: {
//         "foo" => bits(8),
//         "bar" => bits(1),
//     }
// }
#[macro_export]
macro_rules! ty_struct {
    (name: $name:expr, fields: { $($field:expr => $ty:expr),* $(,)? }) => {
        Ty::Struct($crate::ty::TyMap {
            name: $name.into(),
            fields: {
                let mut map = std::collections::BTreeMap::new();
                $(
                    map.insert($field.into(), $ty);
                )*
                map
            }
        })
    };
}

pub(crate) use ty_struct;

macro_rules! ty_enum {
    (name: $name:expr, fields: { $($field:expr => $ty:expr),* $(,)? }) => {
        Ty::Enum($crate::ty::TyMap {
            name: $name.into(),
            fields: {
                let mut map = std::collections::BTreeMap::new();
                $(
                    map.insert($field.into(), $ty);
                )*
                map
            }
        })
    };
}

pub(crate) use ty_enum;

// Start simple, modelling as in the Eli Bendersky example.
// https://eli.thegreenplace.net/2018/type-inference/
// Support for function types as a target for inference
// has been dropped for now.
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum Ty {
    Var(TypeId),
    Const(Bits),
    Ref(Box<Ty>),
    Tuple(Vec<Ty>),
    Array(Vec<Ty>),
    Struct(TyMap),
    Enum(TyMap),
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

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Var(id) => write!(f, "V{}", id.0),
            Ty::Const(bits) => match bits {
                Bits::Signed(width) => write!(f, "s{}", width),
                Bits::Unsigned(width) => write!(f, "b{}", width),
                Bits::Empty => write!(f, "{{}}"),
            },
            Ty::Struct(struct_) | Ty::Enum(struct_) => {
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
        }
    }
}
