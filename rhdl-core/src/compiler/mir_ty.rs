use id_arena::{Arena, Id};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Display, Formatter},
};

use crate::{
    types::kind::{Array, Enum, Struct},
    ClockColor, Kind,
};
use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VarNum(u32);

struct TypeDB {
    types: Arena<Type>,
    var: VarNum,
}

type TypeId = Id<Type>;

// These are types that are fundamental, i.e., not parameterized or
// generic over any other types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Const {
    Struct(Struct),
    Enum(Enum),
    Clock(ClockColor),
    Length(usize),
    Integer,
    Usize,
    Empty,
}

// These are types that are generic over one or more other types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppTypeKind {
    Bits,
    Signed,
    Signal,
    Tuple,
    Array,
}

// These are types that are generic over one or more other types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppType {
    kind: AppTypeKind,
    args: Vec<TypeId>,
}

// The type system is more expressive than the Kind system.  The Kind
// system defines types that are built at compile time, while the type
// system defines types that are built at run time.  The type system
// is necessarily more expressive, and as a result, can represent types
// that do not mean anything in the Kind system.  For example, inference
// variables are not part of the Kind system, but are part of the type
// system.  Furthermore, the type system enforces type rules at run time
// not at compile time.  So while a construct like Bits<Red> is not meaningful,
// it can be constructed at run time.  As a result of this, conversion from
// a Type -> Kind is fallible, while conversion from Kind -> Type is not.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Var(VarNum),
    Const(Const),
    App(AppType),
}

impl TypeDB {
    fn new() -> Self {
        let types: Arena<Type> = Arena::new();
        let var = VarNum(0);
        TypeDB { types, var }
    }

    fn ty_app(&mut self, kind: AppTypeKind, args: Vec<TypeId>) -> TypeId {
        self.types.alloc(Type::App(AppType { kind, args }))
    }

    fn ty_const(&mut self, const_ty: Const) -> TypeId {
        self.types.alloc(Type::Const(const_ty))
    }

    fn ty_const_len(&mut self, len: usize) -> TypeId {
        self.ty_const(Const::Length(len))
    }

    fn ty_bits(&mut self, len: TypeId) -> TypeId {
        self.ty_app(AppTypeKind::Bits, vec![len])
    }

    fn ty_signed(&mut self, len: TypeId) -> TypeId {
        self.ty_app(AppTypeKind::Signed, vec![len])
    }

    fn ty_var(&mut self) -> TypeId {
        let ty = self.types.alloc(Type::Var(self.var));
        self.var.0 += 1;
        ty
    }

    fn ty_array(&mut self, base: TypeId, len: TypeId) -> TypeId {
        self.ty_app(AppTypeKind::Array, vec![base, len])
    }

    fn ty_struct(&mut self, strukt: Struct) -> TypeId {
        self.ty_const(Const::Struct(strukt))
    }

    fn ty_enum(&mut self, enumerate: Enum) -> TypeId {
        self.ty_const(Const::Enum(enumerate))
    }

    fn ty_tuple(&mut self, fields: Vec<TypeId>) -> TypeId {
        self.ty_app(AppTypeKind::Tuple, fields)
    }

    fn ty_clock(&mut self, clock: ClockColor) -> TypeId {
        self.ty_const(Const::Clock(clock))
    }

    fn ty_signal(&mut self, data: TypeId, clock: TypeId) -> TypeId {
        self.ty_app(AppTypeKind::Signal, vec![data, clock])
    }

    fn ty_empty(&mut self) -> TypeId {
        self.ty_const(Const::Empty)
    }

    fn ty_integer(&mut self) -> TypeId {
        self.ty_const(Const::Integer)
    }

    fn ty_usize(&mut self) -> TypeId {
        self.ty_const(Const::Usize)
    }

    fn from_kind(&mut self, kind: Kind) -> TypeId {
        match kind {
            Kind::Bits(n) => {
                let n = self.ty_const_len(n);
                self.ty_bits(n)
            }
            Kind::Signed(n) => {
                let n = self.ty_const_len(n);
                self.ty_signed(n)
            }
            Kind::Empty => self.ty_empty(),
            Kind::Struct(strukt) => self.ty_struct(strukt),
            Kind::Tuple(fields) => {
                let arg = fields
                    .elements
                    .into_iter()
                    .map(|k| self.from_kind(k))
                    .collect();
                self.ty_tuple(arg)
            }
            Kind::Enum(enumerate) => self.ty_enum(enumerate),
            Kind::Array(array) => {
                let base = self.from_kind(*array.base);
                let len = self.ty_const_len(array.size);
                self.ty_array(base, len)
            }
            Kind::Signal(kind, clock) => {
                let kind = self.from_kind(*kind);
                let clock = self.ty_clock(clock);
                self.ty_signal(kind, clock)
            }
        }
    }

    fn is_empty(&self, ty: TypeId) -> bool {
        matches!(self.types[ty], Type::Const(Const::Empty))
    }

    fn is_var(&self, ty: TypeId) -> bool {
        matches!(self.types[ty], Type::Var(_))
    }

    fn desc(&self, ty: TypeId) -> String {
        match &self.types[ty] {
            Type::Var(v) => format!("V{}", v.0),
            Type::Const(c) => match c {
                Const::Struct(s) => format!("struct {:?}", s),
                Const::Enum(e) => format!("enum {:?}", e),
                Const::Clock(c) => format!("clock {}", c),
                Const::Length(n) => format!("length {}", n),
                Const::Integer => "integer".to_string(),
                Const::Usize => "usize".to_string(),
                Const::Empty => "empty".to_string(),
            },
            Type::App(app) => match app.kind {
                AppTypeKind::Bits => format!("bits<{}>", self.desc(app.args[0])),
                AppTypeKind::Signed => format!("signed<{}>", self.desc(app.args[0])),
                AppTypeKind::Signal => format!(
                    "signal<{}, {}>",
                    self.desc(app.args[0]),
                    self.desc(app.args[1])
                ),
                AppTypeKind::Tuple => {
                    let fields = app
                        .args
                        .iter()
                        .map(|a| self.desc(*a))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("tuple<{}>", fields)
                }
                AppTypeKind::Array => format!(
                    "array<{}, {}>",
                    self.desc(app.args[0]),
                    self.desc(app.args[1])
                ),
            },
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct UnifyContext {
    substitution_map: HashMap<VarNum, Type>,
    var_counter: u32,
}

impl Display for UnifyContext {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (v, t) in &self.substitution_map {
            writeln!(f, "V{} -> {}", v.0, t)?;
        }
        Ok(())
    }
}

impl UnifyContext {
    pub fn var_signed(&mut self) -> Type {
        Type::Signed(Box::new(self.var()))
    }
    pub fn var_bits(&mut self) -> Type {
        Type::Bits(Box::new(self.var()))
    }
    pub fn var(&mut self) -> Type {
        let v = VarNum(self.var_counter);
        self.var_counter += 1;
        Type::Var(v)
    }
    fn apply(&self, ty: Type) -> Type {
        match ty {
            Type::Var(v) => {
                if let Some(t) = self.substitution_map.get(&v) {
                    self.apply(t.clone())
                } else {
                    Type::Var(v)
                }
            }
            Type::Tuple(fields) => Type::Tuple(fields.into_iter().map(|t| self.apply(t)).collect()),
            Type::Bits(t) => Type::Bits(Box::new(self.apply(*t))),
            Type::Signed(t) => Type::Signed(Box::new(self.apply(*t))),
            Type::Signal(t1, t2) => {
                Type::Signal(Box::new(self.apply(*t1)), Box::new(self.apply(*t2)))
            }
            t => t,
        }
    }
    pub fn normalize(&self, ty: Type) -> Type {
        let ty = self.apply(ty);
        if let Type::Bits(t) = &ty {
            if let Type::N(n) = **t {
                return Type::Kind(Kind::make_bits(n));
            }
        }
        if let Type::Signed(t) = &ty {
            if let Type::N(n) = **t {
                return Type::Kind(Kind::make_signed(n));
            }
        }
        ty
    }
    pub fn unify(&mut self, x: Type, y: Type) -> Result<()> {
        if x == y {
            return Ok(());
        }
        match (x, y) {
            (Type::Var(x), y) => self.unify_variable(x, y),
            (x, Type::Var(y)) => self.unify_variable(y, x),
            (Type::Kind(x), Type::Kind(y)) => bail!("Cannot unify {} and {}", x, y),
            (Type::Tuple(x), Type::Tuple(y)) => self.unify_tuples(x, y),
            (Type::Bits(x), Type::Bits(y)) => self.unify(*x, *y),
            (Type::Signed(x), Type::Signed(y)) => self.unify(*x, *y),
            (Type::Signal(x1, x2), Type::Signal(y1, y2)) => {
                self.unify(*x1, *y1)?;
                self.unify(*x2, *y2)
            }
            (Type::Signal(d, c), Type::Kind(Kind::Signal(data, clock)))
            | (Type::Kind(Kind::Signal(data, clock)), Type::Signal(d, c)) => {
                self.unify(*d, Type::Kind(*data))?;
                self.unify(*c, clock.into())
            }
            (Type::Bits(x), Type::Kind(Kind::Bits(k)))
            | (Type::Kind(Kind::Bits(k)), Type::Bits(x)) => self.unify(*x, Type::N(k)),
            (Type::Signed(x), Type::Kind(Kind::Signed(k)))
            | (Type::Kind(Kind::Signed(k)), Type::Signed(x)) => self.unify(*x, Type::N(k)),
            (x, y) => bail!("Cannot unify {} and {}", x, y),
        }
    }
    // We want to declare v and x as equivalent.
    fn unify_variable(&mut self, v: VarNum, x: Type) -> Result<()> {
        // If v is already in the subtitution map, then we want
        // to unify x with the value in the map.
        if let Some(t) = self.substitution_map.get(&v).cloned() {
            return self.unify(t, x);
        // There is no substitution for v in the map.  Check to
        // see if x is a variable.
        } else if let Type::Var(x_id) = x {
            // if x is a variable, and it has a substutition in the
            // map, then unify v with the value in the map.
            if let Some(t) = self.substitution_map.get(&x_id).cloned() {
                return self.unify(Type::Var(v), t);
            }
        }
        // To get to this point, we must have v as an unbound
        // variable, and x is either not a variable or it is
        // a variable that is not in the substitution map.
        // Check to make sure that do not create a recursive unification.
        if self.occurs(v, &x) {
            bail!("Recursive unification encountered");
        }
        // All is good, so add the substitution to the map.
        self.substitution_map.insert(v, x);
        Ok(())
    }
    fn occurs(&self, v: VarNum, term: &Type) -> bool {
        // Check for the case that term is a variable
        if let Type::Var(x) = term {
            // Unifying with itself is not allowed
            if *x == v {
                return true;
            }
            // If x is in the substitution map, then check
            // to see if v occurs in the substitution.
            if let Some(t) = self.substitution_map.get(&x) {
                return self.occurs(v, t);
            }
        }
        if let Type::Tuple(fields) = term {
            return fields.iter().any(|t| self.occurs(v, t));
        }
        false
    }
    fn unify_tuples(&mut self, x: Vec<Type>, y: Vec<Type>) -> Result<()> {
        if x.len() != y.len() {
            bail!("Cannot unify tuples of different lengths");
        }
        for (x, y) in x.into_iter().zip(y.into_iter()) {
            self.unify(x, y)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let mut ctx = UnifyContext::default();
        let x = ctx.var();
        let y = ctx.var();
        let z = ctx.var();
        let t = Type::Tuple(vec![x.clone(), y.clone(), z.clone()]);
        let u = Type::Tuple(vec![
            Type::Integer,
            Type::Usize,
            Kind::make_bits(128).into(),
        ]);
        assert!(ctx.unify(t, u).is_ok());
        assert_eq!(ctx.apply(x), Type::Integer);
        assert_eq!(ctx.apply(y), Type::Usize);
        assert_eq!(ctx.apply(z), Kind::make_bits(128).into());
    }

    #[test]
    fn test_case_2() {
        let mut ctx = UnifyContext::default();
        let x = ctx.var_bits();
        let y = Type::Bits(Box::new(Type::N(12)));
        let z = ctx.var();
        assert!(ctx.unify(x.clone(), y.clone()).is_ok());
        assert!(ctx.unify(x.clone(), z.clone()).is_ok());
        let w = Kind::make_bits(12).into();
        assert_eq!(ctx.normalize(z), w);
        assert_eq!(ctx.normalize(y), w);
        assert_eq!(ctx.normalize(x), w);
    }

    #[test]
    fn test_case_3() {
        let mut ctx = UnifyContext::default();
        let x = ctx.var_bits();
        let y: Type = Kind::make_bits(12).into();
        assert!(ctx.unify(x.clone(), y.clone()).is_ok());
        assert_eq!(ctx.normalize(x), Type::Kind(Kind::make_bits(12)));
    }
}
