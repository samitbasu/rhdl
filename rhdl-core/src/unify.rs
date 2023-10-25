use crate::ty::{Ty, TypeId};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::{collections::HashMap, fmt::Display};
type Term = crate::ty::Ty;
type TermMap = crate::ty::TyMap;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct UnifyContext {
    map: HashMap<TypeId, Term>,
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
    pub fn apply(&self, typ: Term) -> Term {
        match typ {
            Term::Var(id) => {
                if let Some(t) = self.map.get(&id) {
                    self.apply(t.clone())
                } else {
                    Term::Var(id)
                }
            }
            Term::Const { .. } => typ,
            Term::Tuple(fields) => Term::Tuple(fields.into_iter().map(|x| self.apply(x)).collect()),
            Term::Ref(t) => Term::Ref(Box::new(self.apply(*t))),
            Term::Array(elems) => Term::Array(elems.into_iter().map(|x| self.apply(x)).collect()),
            Term::Struct(struct_) => Term::Struct(TermMap {
                name: struct_.name,
                fields: struct_
                    .fields
                    .into_iter()
                    .map(|(k, v)| (k, self.apply(v)))
                    .collect(),
            }),
            Term::Enum(enum_) => Term::Enum(TermMap {
                name: enum_.name,
                fields: enum_
                    .fields
                    .into_iter()
                    .map(|(k, v)| (k, self.apply(v)))
                    .collect(),
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
    pub fn unify(&mut self, x: Term, y: Term) -> Result<()> {
        if x == y {
            return Ok(());
        }
        match (x, y) {
            (Term::Var(x), y) => self.unify_variable(x, y),
            (x, Term::Var(y)) => self.unify_variable(y, x),
            (Term::Ref(x), Term::Ref(y)) => self.unify(*x, *y),
            (Term::Const(x), Term::Const(y)) => bail!("Cannot unify {:?} and {:?}", x, y),
            (Term::Tuple(x), Term::Tuple(y)) | (Term::Array(x), Term::Array(y)) => {
                self.unify_tuple_arrays(x, y)
            }
            (Term::Struct(x), Term::Struct(y)) | (Term::Enum(x), Term::Enum(y)) => {
                self.unify_maps(x, y)
            }
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
        if let Term::Ref(x) = term {
            return self.occurs_check(v, x);
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
    pub fn get_variant(&self, t: Ty, variant: &str) -> Result<Ty> {
        let Ty::Var(id) = t else {
            bail!("Cannot get variant of non-variable")
        };
        let Some(t) = self.map.get(&id) else {
            bail!("Type must be known at this point")
        };
        if let Ty::Ref(t) = t {
            return self
                .get_variant(*t.clone(), variant)
                .map(|x| Ty::Ref(Box::new(x)));
        }
        let Ty::Enum(enum_) = t else {
            bail!("Type must be an enum")
        };
        let Some(ty) = enum_.fields.get(variant) else {
            bail!("Variant {} not found", variant)
        };
        Ok(ty.clone())
    }
    pub fn get_named_field(&self, t: Ty, field: &str) -> Result<Ty> {
        let Ty::Var(id) = t else {
            bail!("Cannot get field of non-variable")
        };
        let Some(t) = self.map.get(&id) else {
            bail!("Type must be known at this point")
        };
        if let Ty::Ref(t) = t {
            return self
                .get_named_field(*t.clone(), field)
                .map(|x| Ty::Ref(Box::new(x)));
        }
        let Ty::Struct(struct_) = t else {
            bail!("Type must be a struct")
        };
        let Some(ty) = struct_.fields.get(field) else {
            bail!("Field {} not found", field)
        };
        Ok(ty.clone())
    }
    pub fn get_unnamed_field(&self, t: Ty, field: usize) -> Result<Ty> {
        let Ty::Var(id) = t else {
            bail!("Cannot get field of non-variable")
        };
        let Some(t) = self.map.get(&id) else {
            bail!("Type must be known at this point")
        };
        if let Ty::Ref(t) = t {
            return self
                .get_unnamed_field(*t.clone(), field)
                .map(|x| Ty::Ref(Box::new(x)));
        }
        let Ty::Tuple(fields_) = t else {
            bail!("Type must be a tuple")
        };
        fields_
            .get(field)
            .cloned()
            .ok_or_else(|| anyhow!("Field {} not found", field))
    }
    pub fn get_indexed_item(&self, t: Ty, index: usize) -> Result<Ty> {
        let Ty::Var(id) = t else {
            bail!("Cannot get field of non-variable")
        };
        let Some(t) = self.map.get(&id) else {
            bail!("Type must be known at this point")
        };
        if let Ty::Ref(t) = t {
            return self
                .get_indexed_item(*t.clone(), index)
                .map(|x| Ty::Ref(Box::new(x)));
        }
        let Ty::Array(elems) = t else {
            bail!("Type must be an array")
        };
        elems
            .get(index)
            .cloned()
            .ok_or_else(|| anyhow!("Index {} out of bounds", index))
    }
    pub fn array_vars(&mut self, args: Vec<Ty>) -> Result<Ty> {
        // put all of the terms into a single unified equivalence class
        for x in args.windows(2) {
            self.unify(x[0].clone(), x[1].clone())?;
        }
        Ok(Ty::Array(args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::ty_array;
    use crate::ty::ty_as_ref as as_ref;
    use crate::ty::ty_bits;
    use crate::ty::ty_enum;
    use crate::ty::ty_struct;
    use crate::ty::ty_tuple as tuple;
    use crate::ty::ty_var as var;
    use crate::ty::Bits;

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
        subst.unify(c.clone(), ty_bits(1)).unwrap();
        print!("{}", subst);
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
        subst.unify(b.clone(), tuple(vec![a])).unwrap();
    }

    // Test the reference case...
    // let a
    // let b = &a
    // let c = bool
    // let *b = c (i.e., assert that b = &bool)
    // --> a = bool
    #[test]
    fn test_case_6() {
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let c = var(2);
        let ty_bool = ty_bits(1);
        subst.unify(b.clone(), as_ref(a.clone())).unwrap();
        subst.unify(c.clone(), ty_bool.clone()).unwrap();
        subst.unify(b.clone(), as_ref(c.clone())).unwrap();
        println!("{}", subst);
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
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let c = var(2);
        let d = var(3);
        let ty_bool = ty_bits(1);
        let ty_foo_struct = ty_struct! {
            name: "FooStruct",
            fields: {
                "foo" => ty_bool.clone(),
            }
        };
        subst.unify(a.clone(), ty_foo_struct.clone()).unwrap();
        subst
            .unify(b.clone(), subst.get_named_field(a.clone(), "foo").unwrap())
            .unwrap();
        subst.unify(c.clone(), ty_bool.clone()).unwrap();
        subst.unify(d.clone(), as_ref(b.clone())).unwrap();
        subst.unify(d.clone(), as_ref(c.clone())).unwrap();
        println!("{}", subst);
    }

    // let a = <some struct>
    // let b = &a
    // let c = &b.foo
    // let d = bool
    // let *c = d
    #[test]
    fn test_ref_struct_field() {
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let c = var(2);
        let d = var(3);
        let ty_bool = ty_bits(1);
        let ty_foo_struct = ty_struct!(
        name: "FooStruct",
        fields: {
            "foo" => ty_bool.clone(),
        });
        subst.unify(a.clone(), ty_foo_struct.clone()).unwrap();
        subst.unify(b.clone(), as_ref(a.clone())).unwrap();
        subst
            .unify(c.clone(), subst.get_named_field(b.clone(), "foo").unwrap())
            .unwrap();
        subst.unify(d.clone(), ty_bool.clone()).unwrap();
        subst.unify(c.clone(), as_ref(d.clone())).unwrap();
        println!("{}", subst);
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
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let c = var(2);
        let d = var(3);
        let ty_bool = ty_bits(1);
        subst
            .unify(a.clone(), tuple(vec![var(4), var(5), var(6)]))
            .unwrap();
        subst
            .unify(
                subst.get_unnamed_field(a.clone(), 1).unwrap(),
                ty_bool.clone(),
            )
            .unwrap();
        subst.unify(b.clone(), as_ref(a.clone())).unwrap();
        subst
            .unify(c.clone(), subst.get_unnamed_field(b.clone(), 2).unwrap())
            .unwrap();
        subst.unify(d.clone(), ty_bool.clone()).unwrap();
        subst.unify(c.clone(), as_ref(d.clone())).unwrap();
        subst
            .unify(a.clone(), tuple(vec![var(4), var(5), var(6)]))
            .unwrap();
        println!("{}", subst);
        assert_eq!(
            subst.apply(a),
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
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let c = var(2);
        let d = var(3);
        let e = var(4);
        let ty_bool = ty_bits(1);
        subst.unify(b.clone(), ty_array(a.clone(), 4)).unwrap();
        subst.unify(c.clone(), as_ref(b.clone())).unwrap();
        subst
            .unify(d.clone(), subst.get_indexed_item(c.clone(), 2).unwrap())
            .unwrap();
        subst.unify(e.clone(), ty_bool.clone()).unwrap();
        subst.unify(d.clone(), as_ref(e.clone())).unwrap();
        println!("{}", subst);
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
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let c = var(2);
        let d = var(3);
        let e = var(4);
        let f = var(5);
        let ty_bool = ty_bits(1);
        let ar = subst
            .array_vars(vec![a.clone(), b.clone(), c.clone()])
            .unwrap();
        subst.unify(d.clone(), ar).unwrap();
        subst
            .unify(
                e.clone(),
                as_ref(subst.get_indexed_item(d.clone(), 2).unwrap()),
            )
            .unwrap();
        subst.unify(f.clone(), ty_bool.clone()).unwrap();
        subst.unify(e.clone(), as_ref(f.clone())).unwrap();
        println!("{}", subst);
        assert_eq!(
            subst.apply(d),
            Term::Array(vec![ty_bool.clone(), ty_bool.clone(), ty_bool.clone()]),
        );
    }

    // Test the case of an enum.  First, we construct
    // an enum with 2 variants.
    // The first has no type payload, and the second has
    // a Foo Struct.
    #[test]
    fn test_case_11() {
        let mut subst = UnifyContext::default();
        let a = var(0);
        let b = var(1);
        let ty_bool = ty_bits(1);
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
        subst.unify(a.clone(), ty_enum.clone()).unwrap();
        subst
            .unify(b.clone(), subst.get_variant(a.clone(), "B").unwrap())
            .unwrap();
        println!("{}", subst);
        assert_eq!(subst.apply(b), ty_foo_struct.clone(),);
    }
}
