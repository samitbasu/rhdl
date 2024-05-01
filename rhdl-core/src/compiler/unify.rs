use crate::compiler::ty::{
    ty_array_base, ty_named_field, ty_signal, ty_unnamed_field, ty_var, TyEnum,
};
use crate::compiler::ty::{Ty, TypeId};
use anyhow::bail;
use anyhow::Result;
use std::{collections::HashMap, fmt::Display};
type Term = crate::compiler::ty::Ty;
type TermMap = crate::compiler::ty::TyMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifyContext {
    map: HashMap<TypeId, Term>,
    // The unify context also records the relationship between type variables
    // that are unified.  So if have an expression path that resolves to a binding
    // we record that cross reference here.  While this is not strictly the concern
    // of type inference, it is helpful to avoid duplicating the work later.
    cross_reference: HashMap<TypeId, TypeId>,
}

impl Display for UnifyContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map: Vec<_> = self.map.iter().collect();
        map.sort_by_key(|x| x.0);
        for (k, v) in map {
            writeln!(f, "V{} -> {}", k.0, v)?;
        }
        Ok(())
    }
}

impl UnifyContext {
    pub fn bind(&mut self, parent: TypeId, child: TypeId) {
        self.cross_reference.insert(child, parent);
    }
    pub fn get_parent(&self, child: TypeId) -> Option<TypeId> {
        self.cross_reference.get(&child).cloned()
    }
    pub(super) fn apply(&self, typ: Term) -> Term {
        match typ {
            Term::Var(id) => {
                if let Some(t) = self.map.get(&id) {
                    self.apply(t.clone())
                } else {
                    Term::Var(id)
                }
            }
            Term::Const { .. } | Term::Integer => typ,
            Term::Signal(base, color) => {
                Term::Signal(Box::new(self.apply(*base)), Box::new(self.apply(*color)))
            }
            Term::Tuple(fields) => Term::Tuple(fields.into_iter().map(|x| self.apply(x)).collect()),
            Term::Array(elems) => Term::Array(elems.into_iter().map(|x| self.apply(x)).collect()),
            Term::Struct(struct_) => Term::Struct(TermMap {
                name: struct_.name,
                fields: struct_
                    .fields
                    .into_iter()
                    .map(|(k, v)| (k, self.apply(v)))
                    .collect(),
                kind: struct_.kind,
            }),
            Term::Enum(enum_) => Term::Enum(TyEnum {
                payload: TermMap {
                    name: enum_.payload.name,
                    fields: enum_
                        .payload
                        .fields
                        .into_iter()
                        .map(|(k, v)| (k, self.apply(v)))
                        .collect(),
                    kind: enum_.payload.kind,
                },
                discriminant: enum_.discriminant,
            }),
        }
    }
    fn unify_tuple_arrays(&mut self, x: Vec<Term>, y: Vec<Term>) -> Result<()> {
        if x.len() != y.len() {
            bail!("Cannot unify tuples/arrays of different lengths")
        } else {
            for (x, y) in x.iter().zip(y.iter()) {
                self.unify(x.clone(), y.clone())?;
            }
            Ok(())
        }
    }
    fn unify_maps(&mut self, x: TermMap, y: TermMap) -> Result<()> {
        if x.name != y.name {
            bail!("Cannot unify structs/enums of different names")
        } else {
            for (x_field, y_field) in x.fields.iter().zip(y.fields.iter()) {
                if x_field.0 != y_field.0 {
                    bail!("Cannot unify structs/enums with different fields/discriminants")
                }
                self.unify(x_field.1.clone(), y_field.1.clone())?;
            }
            Ok(())
        }
    }
    pub(super) fn unify(&mut self, x: Term, y: Term) -> Result<()> {
        if x == y {
            return Ok(());
        }
        match (x, y) {
            (Term::Var(x), y) => self.unify_variable(x, y),
            (x, Term::Var(y)) => self.unify_variable(y, x),
            (Term::Const(x), Term::Const(y)) => bail!("Cannot unify {:?} and {:?}", x, y),
            (Term::Tuple(x), Term::Tuple(y)) | (Term::Array(x), Term::Array(y)) => {
                self.unify_tuple_arrays(x, y)
            }
            (Term::Struct(x), Term::Struct(y)) => self.unify_maps(x, y),
            (Term::Signal(x, a), Term::Signal(y, b)) => {
                self.unify(*x, *y).and_then(|_| self.unify(*a, *b))
            }
            (Term::Enum(x), Term::Enum(y)) => self.unify_maps(x.payload, y.payload),
            (x, y) => bail!("Cannot unify {:?} and {:?}", x, y),
        }
    }
    fn unify_variable(&mut self, v: TypeId, x: Term) -> Result<()> {
        if let Some(t) = self.map.get(&v) {
            return self.unify(t.clone(), x);
        } else if let Term::Var(x_id) = x {
            if let Some(t) = self.map.get(&x_id) {
                return self.unify(Term::Var(v), t.clone());
            }
        }
        if self.occurs_check(v, &x) {
            bail!("Recursive unification")
        }
        self.map.insert(v, x);
        Ok(())
    }
    fn occurs_check(&self, v: TypeId, term: &Term) -> bool {
        if let Term::Var(x) = term {
            if *x == v {
                return true;
            }
            if let Some(t) = self.map.get(x) {
                return self.occurs_check(v, t);
            }
        }
        if let Term::Tuple(fields) = term {
            return fields.iter().any(|x| self.occurs_check(v, x));
        }
        if let Term::Array(elems) = term {
            return elems.iter().any(|x| self.occurs_check(v, x));
        }
        if let Term::Struct(struct_) = term {
            return struct_.fields.values().any(|x| self.occurs_check(v, x));
        }
        false
    }
    pub(super) fn get_named_field(&self, t: Ty, field: &str) -> Result<Ty> {
        eprintln!("Getting field {} of {:?}", field, t);
        let Ty::Var(id) = t else {
            bail!("Cannot get field of non-variable")
        };
        let Some(t) = self.map.get(&id) else {
            bail!("Type must be known at this point")
        };
        ty_named_field(t, field)
    }
    pub(super) fn get_unnamed_field(&self, t: Ty, field: usize) -> Result<Ty> {
        let Ty::Var(id) = t else {
            bail!("Cannot get field of non-variable")
        };
        let Some(t) = self.map.get(&id) else {
            bail!("Type must be known at this point")
        };
        ty_unnamed_field(t, field)
    }
    pub(super) fn get_array_base(&self, t: Ty) -> Result<Ty> {
        let Ty::Var(id) = t else {
            bail!("Cannot get field of non-variable")
        };
        let Some(t) = self.map.get(&id) else {
            bail!("Type must be known at this point")
        };
        ty_array_base(t)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::compiler::ty::ty_bits;
    use crate::compiler::ty::ty_tuple as tuple;
    use crate::compiler::ty::ty_var as var;

    #[test]
    fn test_case_1() {
        //  Let's start with the simplest possible
        // case.  A = 1.  In this case, we have 2 type variables,
        // one for A and one for the constant  `1`.
        // We also have a constraint that type of (1) = Kind::Int.
        let mut subst = UnifyContext::default();
        let a = Term::Var(TypeId(0));
        let one = Term::Var(TypeId(1));
        subst.unify(a, one.clone()).unwrap();
        subst.unify(one, ty_bits(4)).unwrap();
        println!("{}", subst);
    }

    #[test]
    fn test_case_2() {
        // Suppose we have a, b, and c, where the types are all equal
        // to each other.  We can unify them all to the same type.
        let mut subst = UnifyContext::default();
        let a = Term::Var(TypeId(0));
        let b = Term::Var(TypeId(1));
        let c = Term::Var(TypeId(2));
        subst.unify(a.clone(), b.clone()).unwrap();
        subst.unify(b.clone(), c.clone()).unwrap();
        subst.unify(c.clone(), a.clone()).unwrap();
        println!("{}", subst);
    }

    #[test]
    fn test_case_3() {
        // Test a failure.  Try to unify a with b and a with c, where b and c are
        // different types.
        let mut subst = UnifyContext::default();
        let a = Term::Var(TypeId(0));
        let b = Term::Var(TypeId(1));
        let c = Term::Var(TypeId(2));
        subst.unify(a.clone(), b.clone()).unwrap();
        subst.unify(a.clone(), c.clone()).unwrap();
        subst.unify(b.clone(), ty_bits(8)).unwrap();
        assert!(subst.unify(c.clone(), ty_bits(1)).is_err());
    }

    #[test]
    fn test_case_4() {
        // The most difficult case.  let q be a tuple of 3 types (a, b, c)
        // and let p be a tuple of 3 types as well (d, e, f).
        // Then unify b with an int, d and f with bools, and check that all types are unified
        // correctly.
        let mut subst = UnifyContext::default();
        let a = Term::Var(TypeId(0));
        let b = Term::Var(TypeId(1));
        let c = Term::Var(TypeId(2));
        let d = Term::Var(TypeId(3));
        let e = Term::Var(TypeId(4));
        let f = Term::Var(TypeId(5));
        let q = Term::Var(TypeId(6));
        let p = Term::Var(TypeId(7));
        let ty_int = ty_bits(8);
        let ty_bool = ty_bits(1);
        subst.unify(b.clone(), ty_int.clone()).unwrap();
        subst.unify(d.clone(), ty_bool.clone()).unwrap();
        subst.unify(f.clone(), ty_bool.clone()).unwrap();
        subst.unify(q.clone(), p.clone()).unwrap();
        subst.unify(q, tuple(vec![a, b, c])).unwrap();
        subst.unify(p, tuple(vec![d, e, f])).unwrap();
        println!("{}", subst);
    }

    #[test]
    fn test_case_5() {
        // Check for self-referentialness
        let mut subst = UnifyContext::default();
        let a = Term::Var(TypeId(0));
        let b = Term::Var(TypeId(1));
        subst.unify(a.clone(), b.clone()).unwrap();
        assert!(subst.unify(b.clone(), tuple(vec![a])).is_err());
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
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let c = var(2);
        let ty_bool = ty_bits(1);
        let ty_foo_struct = Ty::Struct(TermMap {
            name: "FooStruct".into(),
            fields: {
                let mut map = BTreeMap::default();
                map.insert("foo".into(), ty_bool.clone());
                map
            },
            kind: crate::Kind::Empty, // Not correct, but not needed for this test case
        });
        subst.unify(a.clone(), ty_foo_struct.clone()).unwrap();
        subst
            .unify(b.clone(), subst.get_named_field(a.clone(), "foo").unwrap())
            .unwrap();
        subst.unify(c.clone(), ty_bool.clone()).unwrap();
        //subst.unify(d.clone(), as_ref(b.clone())).unwrap();
        //subst.unify(d.clone(), as_ref(c.clone())).unwrap();
        println!("{}", subst);
    }
}
