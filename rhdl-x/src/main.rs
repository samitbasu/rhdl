use anyhow::bail;
use anyhow::Result;
use std::vec;
use std::{collections::HashMap, fmt::Display};

// First we define a type id - this is equivalent to the type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Int,
    Bool,
}

// These are type functions.  They can be applied to a
// type to produce a new type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Functions {
    // The pointer function
    Ref,
    // The tuple function (makes a type tuple)
    Tuple,
    // The function that gets a field from a struct
    GetField(String),
}

impl Display for Functions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Functions::Ref => write!(f, "ref"),
            Functions::Tuple => write!(f, "tuple"),
            Functions::GetField(s) => write!(f, "get_field<{}>", s),
        }
    }
}

// Start simple, modelling as in the Eli Bendersky example.
// Now, consider the case of a reference, i.e., a = &b.
// This is equivalent to saying that a = ptr(b), where ptr
// is some function.  Also, when we say that a = &b.foo,
// that could be considered the application of a function
// a = get_field(b, "foo").
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum Term {
    Var(TypeId),
    Const(Kind),
    App { func: Functions, args: Vec<Term> },
}

fn tuple(args: Vec<Term>) -> Term {
    Term::App {
        func: Functions::Tuple,
        args,
    }
}

fn as_ref(t: Term) -> Term {
    Term::App {
        func: Functions::Ref,
        args: vec![t],
    }
}

fn get_field(t: Term, field: &str) -> Term {
    Term::App {
        func: Functions::GetField(field.to_string()),
        args: vec![t],
    }
}

fn var(id: usize) -> Term {
    Term::Var(TypeId(id))
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(id) => write!(f, "V{}", id.0),
            Term::Const(kind) => write!(f, "Kind<{:?}>", kind),
            Term::App { func, args } => {
                write!(f, "{} (", func)?;
                for (i, term) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", term)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Subst {
    map: HashMap<TypeId, Term>,
    extras: Vec<(Term, Term)>,
}

fn show_table(subst: &Subst) {
    subst
        .map
        .iter()
        .for_each(|(k, v)| println!("V{} -> {}", k.0, v));
    subst
        .extras
        .iter()
        .for_each(|(k, v)| println!("{} <-> {}", k, v));
}

pub fn unify(x: Term, y: Term, mut subst: Subst) -> Result<Subst> {
    if x == y {
        return Ok(subst);
    }
    if let Term::Var(x) = x {
        unify_variable(x, y, subst)
    } else if let Term::Var(y) = y {
        unify_variable(y, x, subst)
    } else if let (
        Term::App {
            func: x_func,
            args: x_args,
        },
        Term::App {
            func: y_func,
            args: y_args,
        },
    ) = (x.clone(), y.clone())
    {
        if x_func != y_func {
            bail!("Cannot unify {:?} and {:?}", x, y)
        }
        if x_args.len() != y_args.len() {
            bail!("Cannot unify tuples of different lengths")
        } else {
            for (x, y) in x_args.iter().zip(y_args.iter()) {
                subst = unify(x.clone(), y.clone(), subst)?;
            }
            Ok(subst)
        }
    } else {
        subst.extras.push((x, y));
        Ok(subst)
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
    } else if let Term::App { func: _, args } = term {
        return args.iter().any(|x| occurs_check(v, x, subst));
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
#[test]
fn test_case_7() {
    let subst = Subst::default();
    let a = var(0);
    let b = var(1);
    let c = var(2);
    let d = var(3);
    let ty_bool = Term::Const(Kind::Bool);
    let subst = unify(b.clone(), get_field(a.clone(), "foo"), subst).unwrap();
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
    let subst = unify(get_field(a.clone(), "1"), ty_bool.clone(), subst).unwrap();
    let subst = unify(b.clone(), as_ref(a.clone()), subst).unwrap();
    let subst = unify(c.clone(), get_field(b.clone(), "2"), subst).unwrap();
    let subst = unify(d.clone(), ty_bool.clone(), subst).unwrap();
    let subst = unify(c.clone(), as_ref(d.clone()), subst).unwrap();
    show_table(&subst);
    assert_eq!(subst.map.get(&TypeId(0)).unwrap(), &tuple(vec![var(4), ty_bool.clone(), ty_bool.clone()]));
}


fn main() {
    println!("Hello, world!");
}
