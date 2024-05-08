use id_arena::{Arena, Id};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Display, Formatter},
};

use crate::{
    types::kind::{Array, DiscriminantLayout, Enum, Struct},
    ClockColor, Kind,
};
use anyhow::{bail, ensure, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VarNum(u32);

type TypeId = Id<Type>;

// These are types that are fundamental, i.e., not parameterized or
// generic over any other types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Const {
    Clock(ClockColor),
    Length(usize),
    Integer,
    Usize,
    Empty,
}

// These are types that are generic over one or more other types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppTypeKind {
    Tuple,
    Array,
    Struct(StructType),
    Enum(EnumType),
}

// A struct is really just a tuple with named fields.
// So if a tuple is generic over it's fields, then
// so is a struct, really.  The only difference is that
// a tuple is characterized only by the list of types
// that make up it's fields.  While a struct also has
// both a name, and names for the fields.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructType {
    name: String,
    fields: Vec<String>,
}

// An enum is generic over the discriminant type and
// the variants themselves.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnumType {
    name: String,
    variants: Vec<VariantTag>,
    discriminant_layout: DiscriminantLayout,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariantTag {
    name: String,
    discriminant: i64,
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

#[derive(Clone, Debug)]
pub struct UnifyContext {
    substitution_map: HashMap<VarNum, TypeId>,
    types: Arena<Type>,
    var: VarNum,
}

impl UnifyContext {
    fn new() -> Self {
        let substitution_map = HashMap::new();
        let types: Arena<Type> = Arena::new();
        let var = VarNum(0);
        Self {
            substitution_map,
            types,
            var,
        }
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
        self.ty_app(
            AppTypeKind::Struct(StructType {
                name: "Bits".to_string(),
                fields: vec!["0".to_string()],
            }),
            vec![len],
        )
    }

    fn ty_signed(&mut self, len: TypeId) -> TypeId {
        self.ty_app(
            AppTypeKind::Struct(StructType {
                name: "Signed".to_string(),
                fields: vec!["0".to_string()],
            }),
            vec![len],
        )
    }

    fn ty_signal(&mut self, data: TypeId, clock: TypeId) -> TypeId {
        self.ty_app(
            AppTypeKind::Struct(StructType {
                name: "Signal".to_string(),
                fields: vec!["data".to_string(), "clock".to_string()],
            }),
            vec![data, clock],
        )
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
        let (names, tids): (Vec<String>, Vec<TypeId>) = strukt
            .fields
            .into_iter()
            .map(|field| {
                let name = field.name.clone();
                let ty = self.from_kind(field.kind);
                (name, ty)
            })
            .unzip();
        self.ty_app(
            AppTypeKind::Struct(StructType {
                name: strukt.name.clone(),
                fields: names,
            }),
            tids,
        )
    }

    fn ty_enum(&mut self, enumerate: Enum) -> TypeId {
        let (tags, tids): (Vec<VariantTag>, Vec<TypeId>) = enumerate
            .variants
            .into_iter()
            .map(|variant| {
                let name = variant.name.clone();
                let ty = self.from_kind(variant.kind);
                let tag = VariantTag {
                    name,
                    discriminant: variant.discriminant,
                };
                (tag, ty)
            })
            .unzip();
        self.ty_app(
            AppTypeKind::Enum(EnumType {
                name: enumerate.name.clone(),
                variants: tags,
                discriminant_layout: enumerate.discriminant_layout,
            }),
            tids,
        )
    }

    fn ty_tuple(&mut self, fields: Vec<TypeId>) -> TypeId {
        self.ty_app(AppTypeKind::Tuple, fields)
    }

    fn ty_clock(&mut self, clock: ClockColor) -> TypeId {
        self.ty_const(Const::Clock(clock))
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
                Const::Clock(c) => format!("clock {}", c),
                Const::Length(n) => format!("length {}", n),
                Const::Integer => "integer".to_string(),
                Const::Usize => "usize".to_string(),
                Const::Empty => "empty".to_string(),
            },
            Type::App(app) => match &app.kind {
                AppTypeKind::Struct(strukt) => strukt
                    .fields
                    .iter()
                    .zip(&app.args)
                    .fold(format!("struct {}<", strukt.name), |acc, (field, ty)| {
                        format!("{}{}:{},", acc, field, self.desc(*ty))
                    }),
                AppTypeKind::Enum(enumerate) => enumerate
                    .variants
                    .iter()
                    .zip(&app.args)
                    .fold(format!("enum {}<", enumerate.name), |acc, (variant, ty)| {
                        format!("{}{}:{},", acc, variant.name, self.desc(*ty))
                    }),
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

impl Display for UnifyContext {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (v, t) in &self.substitution_map {
            writeln!(f, "V{} -> {}", v.0, self.desc(*t))?;
        }
        Ok(())
    }
}

impl UnifyContext {
    fn add_subst(&mut self, v: TypeId, x: TypeId) -> Result<()> {
        let Type::Var(v) = self.types[v] else {
            bail!("Expected a variable, found {:?}", self.types[v]);
        };
        self.substitution_map.insert(v, x);
        Ok(())
    }
    fn subst(&self, ty: TypeId) -> anyhow::Result<Option<TypeId>> {
        let Type::Var(v) = self.types[ty] else {
            bail!("Expected a variable, found {:?}", self.types[ty]);
        };
        if let Some(t) = self.substitution_map.get(&v) {
            return Ok(Some(*t));
        }
        Ok(None)
    }
    fn apply(&mut self, ty: TypeId) -> TypeId {
        match self.types[ty].clone() {
            Type::Var(v) => {
                if let Some(t) = self.substitution_map.get(&v) {
                    self.apply(*t)
                } else {
                    ty
                }
            }
            Type::App(AppType { kind, args }) => {
                let args = args.iter().map(|t| self.apply(*t)).collect();
                self.ty_app(kind.clone(), args)
            }
            _ => ty,
        }
    }
    pub fn unify(&mut self, x: TypeId, y: TypeId) -> Result<()> {
        if self.types[x] == self.types[y] {
            return Ok(());
        }
        match (&self.types[x], &self.types[y]) {
            (Type::Var(_), _) => self.unify_variable(x, y),
            (_, Type::Var(_)) => self.unify_variable(y, x),
            (Type::Const(x), Type::Const(y)) if x == y => Ok(()),
            (Type::App(_), Type::App(_)) => self.unify_app(x, y),
            _ => bail!("Cannot unify {} and {}", self.desc(x), self.desc(y)),
        }
    }
    // We want to declare v and x as equivalent.
    fn unify_variable(&mut self, v: TypeId, x: TypeId) -> Result<()> {
        ensure!(
            self.is_var(v),
            "Expected a variable, found {:?}",
            self.types[v]
        );
        // If v is already in the substitution map, then we want
        // to unify x with the value in the map.
        if let Some(t) = self.subst(v)? {
            return self.unify(t, x);
        // There is no substitution for v in the map.  Check to
        // see if x is a variable.
        } else if self.is_var(x) {
            // if x is a variable, and it has a substitution in the
            // map, then unify v with the value in the map.
            if let Some(t) = self.subst(x)? {
                return self.unify(v, t);
            }
        }
        // To get to this point, we must have v as an unbound (no substitution)
        // variable, and x is either not a variable or it is
        // a variable that is not in the substitution map.
        // Check to make sure that if v -> x, we do not create a
        // recursive unification.
        if self.occurs(v, x) {
            bail!("Recursive unification encountered");
        }
        // All is good, so add the substitution to the map.
        self.add_subst(v, x)
    }
    fn unify_app(&mut self, x: TypeId, y: TypeId) -> Result<()> {
        let Type::App(AppType { kind: k1, args: a1 }) = &self.types[x] else {
            bail!("Expected app type instead of {:?}", self.types[x]);
        };
        let Type::App(AppType { kind: k2, args: a2 }) = &self.types[y] else {
            bail!("Expected app type instead of {:?}", self.types[y]);
        };
        if k1 != k2 {
            bail!("Cannot unify {:?} and {:?}", k1, k2);
        }
        let a1 = a1.clone();
        let a2 = a2.clone();
        if a1.len() != a2.len() {
            bail!("Cannot unify {:?} and {:?}", a1, a2);
        }
        for (a, b) in a1.iter().zip(a2.iter()) {
            self.unify(*a, *b)?;
        }
        Ok(())
    }
    fn occurs(&self, v: TypeId, term: TypeId) -> bool {
        // Check for the case that term is a variable
        if self.is_var(term) {
            // Unifying with itself is not allowed
            if self.types[term] == self.types[v] {
                return true;
            }
            // We know that term is a variable, so check to see
            // if it is in the substitution map.
            // If term is in the substitution map, then check
            // to see if v occurs in the substitution.
            if let Some(t) = self.subst(term).unwrap() {
                return self.occurs(v, t);
            }
        }
        // If term is an application type, then we need to check
        // each of the arguments to see if v occurs in any of them.
        if let Type::App(AppType { args, .. }) = &self.types[term] {
            return args.iter().any(|t| self.occurs(v, *t));
        }
        false
    }
}

/*
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
*/
