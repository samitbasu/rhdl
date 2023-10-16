use anyhow::Result;
use anyhow::{anyhow, bail};
use std::vec;
use std::{collections::HashMap, fmt::Display};

// First we define a type id - this is equivalent to the type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Int,
    Bool,
    Struct { fields: HashMap<String, Kind> },
}

// Start simple, modelling as in the Eli Bendersky example.
// Now, consider the case of a reference, i.e., a = &b.
// This is equivalent to saying that a = ptr(b), where ptr
// is some function.  Also, when we say that a = &b.foo,
// that could be considered the application of a function
// a = get_field(b, "foo").
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum Term {
    Var(TypeId),
    Const(Kind),
    Ref(Box<Term>),
    Tuple { fields: Vec<Term> },
}

fn tuple(args: Vec<Term>) -> Term {
    Term::Tuple { fields: args }
}

fn as_ref(t: Term) -> Term {
    Term::Ref(Box::new(t))
}

fn get_named_field(t: Term, field: &str, subs: &Subst) -> Result<Term> {
    let Term::Var(id) = t else {
        bail!("Cannot get field of non-variable")
    };
    let Some(t) = subs.map.get(&id) else {
        bail!("Type must be known at this point")
    };
    let Term::Const(k) = t else {
        bail!("Type must be known at this point")
    };
    let Kind::Struct { fields } = k else {
        bail!("Type must be a struct")
    };
    let Some(ty) = fields.get(field) else {
        bail!("Field {} not found", field)
    };
    Ok(Term::Const(ty.clone()))
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
    let Term::Tuple { fields } = t else {
        bail!("Type must be a tuple")
    };
    fields
        .get(field)
        .cloned()
        .ok_or_else(|| anyhow!("Field {} not found", field))
}

fn var(id: usize) -> Term {
    Term::Var(TypeId(id))
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(id) => write!(f, "V{}", id.0),
            Term::Const(kind) => write!(f, "Kind<{:?}>", kind),
            Term::Tuple { fields } => {
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

pub fn unify(x: Term, y: Term, mut subst: Subst) -> Result<Subst> {
    if x == y {
        return Ok(subst);
    }
    if let Term::Var(x) = x {
        return unify_variable(x, y, subst);
    }
    if let Term::Var(y) = y {
        return unify_variable(y, x, subst);
    }
    // Neither is a variable.  Check the reference case
    if let (Term::Ref(x), Term::Ref(y)) = (x.clone(), y.clone()) {
        return unify(*x, *y, subst);
    }
    if let (Term::Tuple { fields: x }, Term::Tuple { fields: y }) = (x.clone(), y.clone()) {
        if x.len() != y.len() {
            bail!("Cannot unify tuples of different lengths")
        } else {
            for (x, y) in x.iter().zip(y.iter()) {
                subst = unify(x.clone(), y.clone(), subst)?;
            }
            return Ok(subst);
        }
    }
    bail!("Cannot unify {:?} and {:?}", x, y)
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
    if let Term::Tuple { fields } = term {
        return fields.iter().any(|x| occurs_check(v, x, subst));
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
    let subst = unify(one, Term::Const(Kind::Int), subst).unwrap();
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
    let subst = unify(b.clone(), Term::Const(Kind::Int), subst).unwrap();
    let subst = unify(c.clone(), Term::Const(Kind::Bool), subst).unwrap();
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
    let ty_int = Term::Const(Kind::Int);
    let ty_bool = Term::Const(Kind::Bool);
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
    let ty_bool = Term::Const(Kind::Bool);
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
    let ty_foo_struct = Term::Const(Kind::Struct {
        fields: [("foo".to_string(), Kind::Bool)].into(),
    });
    let ty_bool = Term::Const(Kind::Bool);
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
    let ty_bool = Term::Const(Kind::Bool);
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
        subst.map.get(&TypeId(0)).unwrap(),
        &tuple(vec![var(4), ty_bool.clone(), ty_bool.clone()])
    );
}

fn main() {
    println!("Hello, world!");
}
