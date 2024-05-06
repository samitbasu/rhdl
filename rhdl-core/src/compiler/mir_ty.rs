use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use crate::{ClockColor, Kind};
use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VarNum(u32);

// TODO - worry about the clone cost
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Var(VarNum),
    Kind(Kind),
    Tuple(Vec<Type>),
    Bits(Box<Type>),
    Signed(Box<Type>),
    Signal(Box<Type>, Box<Type>),
    Clock(ClockColor),
    N(usize),
    Integer,
    Usize,
}

impl From<Kind> for Type {
    fn from(k: Kind) -> Self {
        Type::Kind(k)
    }
}

impl From<ClockColor> for Type {
    fn from(c: ClockColor) -> Self {
        Type::Clock(c)
    }
}

impl Type {
    pub fn is_empty(&self) -> bool {
        match self {
            Type::Kind(k) => k.is_empty(),
            _ => false,
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            Type::Kind(k) => k.is_bool(),
            _ => false,
        }
    }
    pub fn is_constant(&self) -> bool {
        matches!(self, Type::Kind(_))
    }
    pub fn is_variable(&self) -> bool {
        matches!(self, Type::Var(_))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Type::Var(v) => write!(f, "V{}", v.0),
            Type::Kind(k) => write!(f, "{}", k),
            Type::Tuple(t) => {
                write!(f, "(")?;
                for (i, t) in t.iter().enumerate() {
                    write!(f, "{},", t)?;
                }
                write!(f, ")")
            }
            Type::Integer => write!(f, "integer"),
            Type::Usize => write!(f, "usize"),
            Type::Bits(t) => write!(f, "b<{}>", t),
            Type::Signed(t) => write!(f, "s<{}>", t),
            Type::Signal(t1, t2) => write!(f, "signal<{}, {}>", t1, t2),
            Type::Clock(c) => write!(f, "{}", c),
            Type::N(n) => write!(f, "N{}", n),
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
