use anyhow::Result;
use anyhow::{anyhow, bail};
use std::collections::BTreeMap;
use std::vec;
use std::{collections::HashMap, fmt::Display};

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
pub struct TermMap {
    name: String,
    fields: BTreeMap<String, Term>,
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
        Term::Struct(TermMap {
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
        Term::Enum(TermMap {
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
// Support for function types as a target for inference
// has been dropped for now.  I don't expect to support
// those at first, focusing instead on traditional type functions.
// Stil to do is enum support
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum Term {
    Var(TypeId),
    Const(Bits),
    Ref(Box<Term>),
    Tuple(Vec<Term>),
    Array(Vec<Term>),
    Struct(TermMap),
    Enum(TermMap),
}

fn bits(width: usize) -> Term {
    Term::Const(Bits::Unsigned(width))
}

fn signed(width: usize) -> Term {
    Term::Const(Bits::Signed(width))
}

fn array(t: Term, len: usize) -> Term {
    Term::Array(vec![t; len])
}

fn array_vars(args: Vec<Term>, mut subs: Subst) -> Result<(Term, Subst)> {
    // put all of the terms into a single unified equivalence class
    for x in args.windows(2) {
        subs = unify(x[0].clone(), x[1].clone(), subs)?;
    }
    Ok((Term::Array(args), subs))
}

fn tuple(args: Vec<Term>) -> Term {
    Term::Tuple(args)
}

fn as_ref(t: Term) -> Term {
    Term::Ref(Box::new(t))
}

fn get_variant(t: Term, variant: &str, subs: &Subst) -> Result<Term> {
    let Term::Var(id) = t else {
        bail!("Cannot get variant of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Term::Ref(t) = t {
        return get_variant(*t.clone(), variant, subs).map(|x| Term::Ref(Box::new(x)));
    }
    let Term::Enum(enum_) = t else {
        bail!("Type must be an enum")
    };
    let Some(ty) = enum_.fields.get(variant) else {
        bail!("Variant {} not found", variant)
    };
    Ok(ty.clone())
}

fn get_named_field(t: Term, field: &str, subs: &Subst) -> Result<Term> {
    let Term::Var(id) = t else {
        bail!("Cannot get field of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Term::Ref(t) = t {
        return get_named_field(*t.clone(), field, subs).map(|x| Term::Ref(Box::new(x)));
    }
    let Term::Struct(struct_) = t else {
        bail!("Type must be a struct")
    };
    let Some(ty) = struct_.fields.get(field) else {
        bail!("Field {} not found", field)
    };
    Ok(ty.clone())
}

fn get_unnamed_field(t: Term, field: usize, subs: &Subst) -> Result<Term> {
    let Term::Var(id) = t else {
        bail!("Cannot get field of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Term::Ref(t) = t {
        return get_unnamed_field(*t.clone(), field, subs).map(|x| Term::Ref(Box::new(x)));
    }
    let Term::Tuple(fields_) = t else {
        bail!("Type must be a tuple")
    };
    fields_
        .get(field)
        .cloned()
        .ok_or_else(|| anyhow!("Field {} not found", field))
}

fn get_indexed_item(t: Term, index: usize, subs: &Subst) -> Result<Term> {
    let Term::Var(id) = t else {
        bail!("Cannot get field of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    if let Term::Ref(t) = t {
        return get_indexed_item(*t.clone(), index, subs).map(|x| Term::Ref(Box::new(x)));
    }
    let Term::Array(elems) = t else {
        bail!("Type must be an array")
    };
    elems
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow!("Index {} out of bounds", index))
}

fn var(id: usize) -> Term {
    Term::Var(TypeId(id))
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(id) => write!(f, "V{}", id.0),
            Term::Const(bits) => match bits {
                Bits::Signed(width) => write!(f, "s{}", width),
                Bits::Unsigned(width) => write!(f, "b{}", width),
                Bits::Empty => write!(f, "{{}}"),
            },
            Term::Struct(struct_) | Term::Enum(struct_) => {
                write!(f, "{} {{", struct_.name)?;
                for (i, (field, ty)) in struct_.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field, ty)?;
                }
                write!(f, "}}")
            }
            Term::Tuple(fields) => {
                write!(f, "(")?;
                for (i, term) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", term)?;
                }
                write!(f, ")")
            }
            Term::Ref(t) => write!(f, "&{}", t),
            Term::Array(elems) => {
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

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Subst {
    map: HashMap<TypeId, Term>,
}

fn show_table(subst: &Subst) {
    subst
        .map
        .iter()
        .for_each(|(k, v)| println!("V{} -> {}", k.0, v));
}

pub fn apply_unifier(typ: Term, subst: &Subst) -> Term {
    match typ {
        Term::Var(id) => {
            if let Some(t) = subst.map.get(&id) {
                apply_unifier(t.clone(), subst)
            } else {
                Term::Var(id)
            }
        }
        Term::Const { .. } => typ,
        Term::Tuple(fields) => Term::Tuple(
            fields
                .into_iter()
                .map(|x| apply_unifier(x, subst))
                .collect(),
        ),
        Term::Ref(t) => Term::Ref(Box::new(apply_unifier(*t, subst))),
        Term::Array(elems) => {
            Term::Array(elems.into_iter().map(|x| apply_unifier(x, subst)).collect())
        }
        Term::Struct(struct_) => Term::Struct(TermMap {
            name: struct_.name,
            fields: struct_
                .fields
                .into_iter()
                .map(|(k, v)| (k, apply_unifier(v, subst)))
                .collect(),
        }),
        Term::Enum(enum_) => Term::Enum(TermMap {
            name: enum_.name,
            fields: enum_
                .fields
                .into_iter()
                .map(|(k, v)| (k, apply_unifier(v, subst)))
                .collect(),
        }),
    }
}

fn unify_tuple_arrays(x: Vec<Term>, y: Vec<Term>, mut subst: Subst) -> Result<Subst> {
    if x.len() != y.len() {
        bail!("Cannot unify tuples/arrays of different lengths")
    } else {
        for (x, y) in x.iter().zip(y.iter()) {
            subst = unify(x.clone(), y.clone(), subst)?;
        }
        Ok(subst)
    }
}

fn unify_maps(x: TermMap, y: TermMap, mut subst: Subst) -> Result<Subst> {
    if x.name != y.name {
        bail!("Cannot unify structs/enums of different names")
    } else {
        for (x_field, y_field) in x.fields.iter().zip(y.fields.iter()) {
            if x_field.0 != y_field.0 {
                bail!("Cannot unify structs/enums with different fields/discriminants")
            }
            subst = unify(x_field.1.clone(), y_field.1.clone(), subst)?;
        }
        Ok(subst)
    }
}

pub fn unify(x: Term, y: Term, mut subst: Subst) -> Result<Subst> {
    if x == y {
        return Ok(subst);
    }
    match (x, y) {
        (Term::Var(x), y) => unify_variable(x, y, subst),
        (x, Term::Var(y)) => unify_variable(y, x, subst),
        (Term::Ref(x), Term::Ref(y)) => unify(*x, *y, subst),
        (Term::Const(x), Term::Const(y)) => bail!("Cannot unify {:?} and {:?}", x, y),
        (Term::Tuple(x), Term::Tuple(y)) | (Term::Array(x), Term::Array(y)) => {
            unify_tuple_arrays(x, y, subst)
        }
        (Term::Struct(x), Term::Struct(y)) | (Term::Enum(x), Term::Enum(y)) => {
            unify_maps(x, y, subst)
        }
        (x, y) => bail!("Cannot unify {:?} and {:?}", x, y),
    }
}

fn unify_variable(v: TypeId, x: Term, mut subst: Subst) -> Result<Subst> {
    if let Some(t) = subst.map.get(&v) {
        return unify(t.clone(), x, subst);
    } else if let Term::Var(x_id) = x {
        if let Some(t) = subst.map.get(&x_id) {
            return unify(Term::Var(v), t.clone(), subst);
        }
    }
    if occurs_check(v, &x, &subst) {
        bail!("Recursive unification")
    }
    subst.map.insert(v, x);
    Ok(subst)
}

fn occurs_check(v: TypeId, term: &Term, subst: &Subst) -> bool {
    if let Term::Var(x) = term {
        if *x == v {
            return true;
        }
        if let Some(t) = subst.map.get(&x) {
            return occurs_check(v, t, subst);
        }
    }
    if let Term::Ref(x) = term {
        return occurs_check(v, x, subst);
    }
    if let Term::Tuple(fields) = term {
        return fields.iter().any(|x| occurs_check(v, x, subst));
    }
    if let Term::Array(elems) = term {
        return elems.iter().any(|x| occurs_check(v, x, subst));
    }
    if let Term::Struct(struct_) = term {
        return struct_.fields.values().any(|x| occurs_check(v, x, subst));
    }
    false
}

#[test]
fn test_case_1() {
    //  Let's start with the simplest possible
    // case.  A = 1.  In this case, we have 2 type variables,
    // one for A and one for the constant  `1`.
    // We also have a constraint that type of (1) = Kind::Int.
    let subst = Subst::default();
    let a = Term::Var(TypeId(0));
    let one = Term::Var(TypeId(1));
    let subst = unify(a, one.clone(), subst).unwrap();
    let subst = unify(one, bits(4), subst).unwrap();
    println!("{:?}", subst);
}

#[test]
fn test_case_2() {
    // Suppose we have a, b, and c, where the types are all equal
    // to each other.  We can unify them all to the same type.
    let subst = Subst::default();
    let a = Term::Var(TypeId(0));
    let b = Term::Var(TypeId(1));
    let c = Term::Var(TypeId(2));
    let subst = unify(a.clone(), b.clone(), subst).unwrap();
    let subst = unify(b.clone(), c.clone(), subst).unwrap();
    let subst = unify(c.clone(), a.clone(), subst).unwrap();
    println!("{:?}", subst);
}

#[test]
fn test_case_3() {
    // Test a failure.  Try to unify a with b and a with c, where b and c are
    // different types.
    let subst = Subst::default();
    let a = Term::Var(TypeId(0));
    let b = Term::Var(TypeId(1));
    let c = Term::Var(TypeId(2));
    let subst = unify(a.clone(), b.clone(), subst).unwrap();
    let subst = unify(a.clone(), c.clone(), subst).unwrap();
    let subst = unify(b.clone(), bits(8), subst).unwrap();
    let subst = unify(c.clone(), bits(1), subst).unwrap();
    print!("{:?}", subst);
}

#[test]
fn test_case_4() {
    // The most difficult case.  let q be a tuple of 3 types (a, b, c)
    // and let p be a tuple of 3 types as well (d, e, f).
    // Then unify b with an int, d and f with bools, and check that all types are unified
    // correctly.
    let subst = Subst::default();
    let a = Term::Var(TypeId(0));
    let b = Term::Var(TypeId(1));
    let c = Term::Var(TypeId(2));
    let d = Term::Var(TypeId(3));
    let e = Term::Var(TypeId(4));
    let f = Term::Var(TypeId(5));
    let q = Term::Var(TypeId(6));
    let p = Term::Var(TypeId(7));
    let ty_int = bits(8);
    let ty_bool = bits(1);
    let subst = unify(b.clone(), ty_int.clone(), subst).unwrap();
    let subst = unify(d.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(f.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(q.clone(), p.clone(), subst).unwrap();
    let subst = unify(q, tuple(vec![a, b, c]), subst).unwrap();
    let subst = unify(p, tuple(vec![d, e, f]), subst).unwrap();
    show_table(&subst);
}

#[test]
fn test_case_5() {
    // Check for self-referentialness
    let subst = Subst::default();
    let a = Term::Var(TypeId(0));
    let b = Term::Var(TypeId(1));
    let subst = unify(a.clone(), b.clone(), subst).unwrap();
    let subst = unify(b.clone(), tuple(vec![a]), subst).unwrap();
}

// Test the reference case...
// let a
// let b = &a
// let c = bool
// let *b = c (i.e., assert that b = &bool)
// --> a = bool
#[test]
fn test_case_6() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let c = var(2);
    let ty_bool = bits(1);
    let subst = unify(b.clone(), as_ref(a.clone()), subst).unwrap();
    let subst = unify(c.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(b.clone(), as_ref(c.clone()), subst).unwrap();
    show_table(&subst);
    assert_eq!(subst.map.get(&TypeId(0)).unwrap(), &ty_bool);
}

// What happens in this case?
// let a
// let b = a.foo
// let c = bool
// let d = &b
// let *d = c
// --> a = { foo: bool }
// Answer - Rust insists that a needs an explicit type!
// So we can't do this.
#[test]
fn test_case_7() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let c = var(2);
    let d = var(3);
    let ty_bool = bits(1);
    let ty_foo_struct = ty_struct! {
        name: "FooStruct",
        fields: {
            "foo" => ty_bool.clone(),
        }
    };
    let subst = unify(a.clone(), ty_foo_struct.clone(), subst).unwrap();
    let subst = unify(
        b.clone(),
        get_named_field(a.clone(), "foo", &subst).unwrap(),
        subst,
    )
    .unwrap();
    let subst = unify(c.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(d.clone(), as_ref(b.clone()), subst).unwrap();
    let subst = unify(d.clone(), as_ref(c.clone()), subst).unwrap();
    show_table(&subst);
}

// let a = <some struct>
// let b = &a
// let c = &b.foo
// let d = bool
// let *c = d
#[test]
fn test_ref_struct_field() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let c = var(2);
    let d = var(3);
    let ty_bool = bits(1);
    let ty_foo_struct = ty_struct!(
    name: "FooStruct",
    fields: {
        "foo" => ty_bool.clone(),
    });
    let subst = unify(a.clone(), ty_foo_struct.clone(), subst).unwrap();
    let subst = unify(b.clone(), as_ref(a.clone()), subst).unwrap();
    let subst = unify(
        c.clone(),
        get_named_field(b.clone(), "foo", &subst).unwrap(),
        subst,
    )
    .unwrap();
    let subst = unify(d.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(c.clone(), as_ref(d.clone()), subst).unwrap();
    show_table(&subst);
}

// Let's keep going.
// Suppose we have a tuple type
// let a = (x, y, z)
// and then we have
// y = bool
// And then we have
// let b = &a
// let c = &b.2
// let d = bool
// let *c = d
// Then do we have a = (x, bool, bool)?
#[test]
fn test_case_8() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let c = var(2);
    let d = var(3);
    let ty_bool = bits(1);
    let subst = unify(a.clone(), tuple(vec![var(4), var(5), var(6)]), subst).unwrap();
    let subst = unify(
        get_unnamed_field(a.clone(), 1, &subst).unwrap(),
        ty_bool.clone(),
        subst,
    )
    .unwrap();
    let subst = unify(b.clone(), as_ref(a.clone()), subst).unwrap();
    let subst = unify(
        c.clone(),
        get_unnamed_field(b.clone(), 2, &subst).unwrap(),
        subst,
    )
    .unwrap();
    let subst = unify(d.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(c.clone(), as_ref(d.clone()), subst).unwrap();
    let subst = unify(a.clone(), tuple(vec![var(4), var(5), var(6)]), subst).unwrap();
    show_table(&subst);
    assert_eq!(
        apply_unifier(a, &subst),
        tuple(vec![var(4), ty_bool.clone(), ty_bool.clone()])
    );
}

// Test array case:
// let a
// let b = [a; 4]
// let c = &b
// let d = &c[2]
// let e = bool
// let *d = e
#[test]
fn test_case_9() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let c = var(2);
    let d = var(3);
    let e = var(4);
    let ty_bool = bits(1);
    let subst = unify(b.clone(), array(a.clone(), 4), subst).unwrap();
    let subst = unify(c.clone(), as_ref(b.clone()), subst).unwrap();
    let subst = unify(
        d.clone(),
        get_indexed_item(c.clone(), 2, &subst).unwrap(),
        subst,
    )
    .unwrap();
    let subst = unify(e.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(d.clone(), as_ref(e.clone()), subst).unwrap();
    show_table(&subst);
    assert_eq!(subst.map.get(&TypeId(0)).unwrap(), &ty_bool.clone());
}

// Test array from multiple variables:
// let a
// let b
// let c
// let d = [a, b, c];
// let e = &d[2]
// let f = bool
// let *e = f
#[test]
fn test_case_10() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let c = var(2);
    let d = var(3);
    let e = var(4);
    let f = var(5);
    let ty_bool = bits(1);
    let (ar, subst) = array_vars(vec![a.clone(), b.clone(), c.clone()], subst).unwrap();
    let subst = unify(d.clone(), ar, subst).unwrap();
    let subst = unify(
        e.clone(),
        as_ref(get_indexed_item(d.clone(), 2, &subst).unwrap()),
        subst,
    )
    .unwrap();
    let subst = unify(f.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(e.clone(), as_ref(f.clone()), subst).unwrap();
    show_table(&subst);
    assert_eq!(
        apply_unifier(d, &subst),
        Term::Array(vec![ty_bool.clone(), ty_bool.clone(), ty_bool.clone()]),
    );
}

// Test the case of an enum.  First, we construct
// an enum with 2 variants.
// The first has no type payload, and the second has
// a Foo Struct.
#[test]
fn test_case_11() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let ty_bool = bits(1);
    let ty_foo_struct = ty_struct!(
    name: "FooStruct",
    fields: {
        "foo" => ty_bool.clone(),
    });
    let ty_enum = ty_enum!(
    name: "FooEnum",
    fields: {
        "A" => Term::Const(Bits::Empty),
        "B" => ty_foo_struct.clone(),
    });
    let subst = unify(a.clone(), ty_enum.clone(), subst).unwrap();
    let subst = unify(
        b.clone(),
        get_variant(a.clone(), "B", &subst).unwrap(),
        subst,
    )
    .unwrap();
    show_table(&subst);
    assert_eq!(apply_unifier(b, &subst), ty_foo_struct.clone(),);
}

// Need to sort out interaction of pre-defined arrays tuples, with dynamically generated
// arrays and tuples.
// For example, if a is struct that contains a field foo, which has type (bool, bool, bool),
// then if we assign
// a.foo = (b, c, d)
// We immediately know that the tuple of (b, c, d) are all bools.

fn main() {
    println!("Hello, world!");
}
