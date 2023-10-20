// This module provides the type system for RHDL.
use anyhow::Result;
use anyhow::{anyhow, bail};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;

// First we define a type id - this is equivalent to the type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bits {
    Signed(usize),
    Unsigned(usize),
    Empty,
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct TyMap {
    name: String,
    fields: BTreeMap<String, Ty>,
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
macro_rules! ty_struct {
    (name: $name:expr, fields: { $($field:expr => $ty:expr),* $(,)? }) => {
        Ty::Struct(TyMap {
            name: $name.into(),
            fields: {
                let mut map = BTreeMap::new();
                $(
                    map.insert($field.into(), $ty);
                )*
                map
            }
        })
    };
}

macro_rules! ty_enum {
    (name: $name:expr, fields: { $($field:expr => $ty:expr),* $(,)? }) => {
        Ty::Enum(TyMap {
            name: $name.into(),
            fields: {
                let mut map = BTreeMap::new();
                $(
                    map.insert($field.into(), $ty);
                )*
                map
            }
        })
    };
}

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

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Subst {
    map: HashMap<TypeId, Ty>,
}

fn show_table(subst: &Subst) {
    subst
        .map
        .iter()
        .for_each(|(k, v)| println!("V{} -> {}", k.0, v));
}

/*
fn array_vars(args: Vec<Ty>, mut subs: Subst) -> Result<(Ty, Subst)> {
    // put all of the terms into a single unified equivalence class
    for x in args.windows(2) {
        subs = unify(x[0].clone(), x[1].clone(), subs)?;
    }
    Ok((Ty::Array(args), subs))
}
*/

fn tuple(args: Vec<Ty>) -> Ty {
    Ty::Tuple(args)
}

fn as_ref(t: Ty) -> Ty {
    Ty::Ref(Box::new(t))
}

fn get_variant(t: Ty, variant: &str, subs: &Subst) -> Result<Ty> {
    let Ty::Var(id) = t else {
        bail!("Cannot get variant of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Ty::Ref(t) = t {
        return get_variant(*t.clone(), variant, subs).map(|x| Ty::Ref(Box::new(x)));
    }
    let Ty::Enum(enum_) = t else {
        bail!("Type must be an enum")
    };
    let Some(ty) = enum_.fields.get(variant) else {
        bail!("Variant {} not found", variant)
    };
    Ok(ty.clone())
}

fn get_named_field(t: Ty, field: &str, subs: &Subst) -> Result<Ty> {
    let Ty::Var(id) = t else {
        bail!("Cannot get field of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Ty::Ref(t) = t {
        return get_named_field(*t.clone(), field, subs).map(|x| Ty::Ref(Box::new(x)));
    }
    let Ty::Struct(struct_) = t else {
        bail!("Type must be a struct")
    };
    let Some(ty) = struct_.fields.get(field) else {
        bail!("Field {} not found", field)
    };
    Ok(ty.clone())
}

fn get_unnamed_field(t: Ty, field: usize, subs: &Subst) -> Result<Ty> {
    let Ty::Var(id) = t else {
        bail!("Cannot get field of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Ty::Ref(t) = t {
        return get_unnamed_field(*t.clone(), field, subs).map(|x| Ty::Ref(Box::new(x)));
    }
    let Ty::Tuple(fields_) = t else {
        bail!("Type must be a tuple")
    };
    fields_
        .get(field)
        .cloned()
        .ok_or_else(|| anyhow!("Field {} not found", field))
}

fn get_indexed_item(t: Ty, index: usize, subs: &Subst) -> Result<Ty> {
    let Ty::Var(id) = t else {
        bail!("Cannot get field of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Ty::Ref(t) = t {
        return get_indexed_item(*t.clone(), index, subs).map(|x| Ty::Ref(Box::new(x)));
    }
    let Ty::Array(elems) = t else {
        bail!("Type must be an array")
    };
    elems
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow!("Index {} out of bounds", index))
}

fn var(id: usize) -> Ty {
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
