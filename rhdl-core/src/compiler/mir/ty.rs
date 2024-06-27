use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    iter::once,
};

use crate::{
    ast::ast_impl::NodeId,
    types::kind::{Array, DiscriminantLayout, Enum, Field, Struct, Tuple},
    Color, DiscriminantAlignment, DiscriminantType, Kind, VariantType,
};
use anyhow::{anyhow, bail, ensure, Result};

use super::interner::{Intern, InternKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VarNum(u32);

pub type TypeKindId = InternKey<TypeKind>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignFlag {
    Unsigned,
    Signed,
}

// These are types that are fundamental, i.e., not parameterized or
// generic over any other types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Const {
    Clock(Color),
    Length(usize),
    Empty,
    Signed(SignFlag),
}

// These are types that are generic over one or more other types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppType {
    Tuple(AppTuple),
    Array(AppArray),
    Struct(AppStruct),
    Enum(AppEnum),
    Bits(AppBits),
    Signal(AppSignal),
}

pub trait AppTypeKind {
    fn sub_types(&self) -> Vec<TypeId>;
    fn apply(self, context: &mut UnifyContext) -> Self;
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind>;
}

impl AppTypeKind for AppType {
    fn sub_types(&self) -> Vec<TypeId> {
        match self {
            AppType::Tuple(tuple) => tuple.sub_types(),
            AppType::Array(array) => array.sub_types(),
            AppType::Struct(strukt) => strukt.sub_types(),
            AppType::Enum(enumerate) => enumerate.sub_types(),
            AppType::Bits(bits) => bits.sub_types(),
            AppType::Signal(signal) => signal.sub_types(),
        }
    }
    fn apply(self, context: &mut UnifyContext) -> Self {
        match self {
            AppType::Tuple(tuple) => AppType::Tuple(tuple.apply(context)),
            AppType::Array(array) => AppType::Array(array.apply(context)),
            AppType::Struct(strukt) => AppType::Struct(strukt.apply(context)),
            AppType::Enum(enumerate) => AppType::Enum(enumerate.apply(context)),
            AppType::Bits(bits) => AppType::Bits(bits.apply(context)),
            AppType::Signal(signal) => AppType::Signal(signal.apply(context)),
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind> {
        match self {
            AppType::Tuple(tuple) => tuple.into_kind(context),
            AppType::Array(array) => array.into_kind(context),
            AppType::Struct(strukt) => strukt.into_kind(context),
            AppType::Enum(enumerate) => enumerate.into_kind(context),
            AppType::Bits(bits) => bits.into_kind(context),
            AppType::Signal(signal) => signal.into_kind(context),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppBits {
    sign_flag: TypeId,
    len: TypeId,
}

impl AppTypeKind for AppBits {
    fn sub_types(&self) -> Vec<TypeId> {
        vec![self.sign_flag, self.len]
    }
    fn apply(self, context: &mut UnifyContext) -> Self {
        AppBits {
            sign_flag: context.apply(self.sign_flag),
            len: context.apply(self.len),
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind> {
        let sign_flag = context.cast_ty_as_sign_flag(self.sign_flag)?;
        let len = context.cast_ty_as_bit_length(self.len)?;
        match sign_flag {
            SignFlag::Signed => Ok(Kind::Signed(len)),
            SignFlag::Unsigned => Ok(Kind::Bits(len)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppSignal {
    data: TypeId,
    clock: TypeId,
}

impl AppTypeKind for AppSignal {
    fn sub_types(&self) -> Vec<TypeId> {
        vec![self.data, self.clock]
    }
    fn apply(self, context: &mut UnifyContext) -> Self {
        AppSignal {
            data: context.apply(self.data),
            clock: context.apply(self.clock),
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind> {
        let data = context.into_kind(self.data)?;
        let clock = context.cast_ty_as_clock(self.clock)?;
        Ok(Kind::Signal(Box::new(data), clock))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppArray {
    base: TypeId,
    len: TypeId,
}

impl AppTypeKind for AppArray {
    fn sub_types(&self) -> Vec<TypeId> {
        once(self.base).chain(once(self.len)).collect()
    }
    fn apply(self, context: &mut UnifyContext) -> Self {
        AppArray {
            base: context.apply(self.base),
            len: context.apply(self.len),
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind> {
        let base = Box::new(context.into_kind(self.base)?);
        let size = context.cast_ty_as_bit_length(self.len)?;
        Ok(Kind::Array(Array { base, size }))
    }
}

// A struct is really just a tuple with named fields.
// So if a tuple is generic over it's fields, then
// so is a struct, really.  The only difference is that
// a tuple is characterized only by the list of types
// that make up it's fields.  While a struct also has
// both a name, and names for the fields.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppStruct {
    name: String,
    fields: Vec<(String, TypeId)>,
}

impl AppTypeKind for AppStruct {
    fn sub_types(&self) -> Vec<TypeId> {
        self.fields.iter().map(|(_, t)| *t).collect()
    }
    fn apply(self, context: &mut UnifyContext) -> Self {
        AppStruct {
            fields: self
                .fields
                .iter()
                .map(|(f, t)| (f.clone(), context.apply(*t)))
                .collect(),
            ..self
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind> {
        let fields = self
            .fields
            .into_iter()
            .map(|(name, t)| {
                let kind = context.into_kind(t)?;
                Ok(Field { name, kind })
            })
            .collect::<Result<_>>()?;
        Ok(Kind::Struct(Struct {
            name: self.name,
            fields,
        }))
    }
}

// An enum is generic over the discriminant type and
// the variants themselves.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppEnum {
    name: String,
    variants: Vec<(VariantTag, TypeId)>,
    discriminant: TypeId,
    alignment: DiscriminantAlignment,
}

impl AppTypeKind for AppEnum {
    fn sub_types(&self) -> Vec<TypeId> {
        once(self.discriminant)
            .chain(self.variants.iter().map(|(_, t)| *t))
            .collect()
    }
    fn apply(self, context: &mut UnifyContext) -> Self {
        AppEnum {
            discriminant: context.apply(self.discriminant),
            variants: self
                .variants
                .into_iter()
                .map(|(v, t)| (v, context.apply(t)))
                .collect(),
            ..self
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind> {
        let variants = self
            .variants
            .into_iter()
            .map(|(v, t)| {
                let kind = context.into_kind(t)?;
                Ok(crate::types::kind::Variant {
                    name: v.name,
                    kind,
                    discriminant: v.discriminant,
                    ty: v.ty,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let discriminant_kind = context.into_kind(self.discriminant)?;
        Ok(Kind::make_enum(
            &self.name,
            variants,
            DiscriminantLayout {
                ty: if discriminant_kind.is_signed() {
                    DiscriminantType::Signed
                } else {
                    DiscriminantType::Unsigned
                },
                width: discriminant_kind.bits(),
                alignment: self.alignment,
            },
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppTuple {
    elements: Vec<TypeId>,
}

impl AppTypeKind for AppTuple {
    fn sub_types(&self) -> Vec<TypeId> {
        self.elements.clone()
    }
    fn apply(self, context: &mut UnifyContext) -> Self {
        AppTuple {
            elements: self.elements.iter().map(|t| context.apply(*t)).collect(),
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> anyhow::Result<Kind> {
        let elements = self
            .elements
            .into_iter()
            .map(|t| context.into_kind(t))
            .collect::<Result<Vec<_>>>()?;
        Ok(Kind::Tuple(Tuple { elements }))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariantTag {
    name: String,
    discriminant: i64,
    ty: VariantType,
}

pub fn make_variant_tag(name: &str, discriminant: i64, ty: VariantType) -> VariantTag {
    VariantTag {
        name: name.to_string(),
        discriminant,
        ty,
    }
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
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct TypeId {
    kind: TypeKindId,
    pub id: NodeId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Var(VarNum),
    Const(Const),
    App(AppType),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct ModificationState {
    update_count: usize,
    intern_size: usize,
}

pub struct UnifyContext {
    substitution_map: HashMap<VarNum, TypeId>,
    types: Intern<TypeKind>,
    var: VarNum,
    update_count: usize,
}

impl Default for UnifyContext {
    fn default() -> Self {
        let substitution_map = HashMap::new();
        let var = VarNum(0);
        Self {
            substitution_map,
            types: Default::default(),
            var,
            update_count: 0,
        }
    }
}

impl UnifyContext {
    pub fn ty(&self, ty: TypeId) -> &TypeKind {
        &self.types[ty.kind]
    }

    pub fn modification_state(&self) -> ModificationState {
        ModificationState {
            update_count: self.update_count,
            intern_size: self.types.count(),
        }
    }

    fn ty_app(&mut self, id: NodeId, kind: AppType) -> TypeId {
        TypeId {
            id,
            kind: self.types.intern(&TypeKind::App(kind)),
        }
    }

    pub fn ty_const(&mut self, id: NodeId, const_ty: Const) -> TypeId {
        TypeId {
            id,
            kind: self.types.intern(&TypeKind::Const(const_ty)),
        }
    }

    pub fn ty_const_len(&mut self, id: NodeId, len: usize) -> TypeId {
        self.ty_const(id, Const::Length(len))
    }

    pub fn ty_bool(&mut self, id: NodeId) -> TypeId {
        let n = self.ty_const_len(id, 1);
        self.ty_bits(id, n)
    }

    pub fn ty_usize(&mut self, id: NodeId) -> TypeId {
        let n = self.ty_const_len(id, 64);
        self.ty_bits(id, n)
    }

    pub fn ty_sign_flag(&mut self, id: NodeId, sign_flag: SignFlag) -> TypeId {
        self.ty_const(id, Const::Signed(sign_flag))
    }

    pub fn ty_bits(&mut self, id: NodeId, len: TypeId) -> TypeId {
        let sign_flag = self.ty_sign_flag(id, SignFlag::Unsigned);
        self.ty_app(id, AppType::Bits(AppBits { sign_flag, len }))
    }

    pub fn ty_signed(&mut self, id: NodeId, len: TypeId) -> TypeId {
        let sign_flag = self.ty_sign_flag(id, SignFlag::Signed);
        self.ty_app(id, AppType::Bits(AppBits { sign_flag, len }))
    }

    pub fn ty_signal(&mut self, id: NodeId, data: TypeId, clock: TypeId) -> TypeId {
        self.ty_app(id, AppType::Signal(AppSignal { data, clock }))
    }

    pub fn ty_maybe_signed(&mut self, id: NodeId, len: TypeId) -> TypeId {
        let sign_flag = self.ty_var(id);
        self.ty_app(id, AppType::Bits(AppBits { sign_flag, len }))
    }

    pub fn ty_var(&mut self, id: NodeId) -> TypeId {
        let ty = TypeId {
            id,
            kind: self.types.intern(&TypeKind::Var(self.var)),
        };
        self.var.0 += 1;
        ty
    }

    pub fn ty_array(&mut self, id: NodeId, base: TypeId, len: TypeId) -> TypeId {
        self.ty_app(id, AppType::Array(AppArray { base, len }))
    }

    pub fn ty_struct(&mut self, id: NodeId, strukt: &Struct) -> TypeId {
        let fields: Vec<(String, TypeId)> = strukt
            .fields
            .iter()
            .map(|field| {
                let name = field.name.clone();
                let ty = self.from_kind(id, &field.kind);
                (name, ty)
            })
            .collect();
        self.ty_app(
            id,
            AppType::Struct(AppStruct {
                name: strukt.name.clone(),
                fields,
            }),
        )
    }

    pub fn ty_dyn_struct(
        &mut self,
        id: NodeId,
        name: String,
        fields: Vec<(String, TypeId)>,
    ) -> TypeId {
        self.ty_app(id, AppType::Struct(AppStruct { name, fields }))
    }

    fn ty_discriminant(&mut self, id: NodeId, layout: DiscriminantLayout) -> TypeId {
        let len = self.ty_const_len(id, layout.width);
        match layout.ty {
            DiscriminantType::Unsigned => self.ty_bits(id, len),
            DiscriminantType::Signed => self.ty_signed(id, len),
        }
    }

    pub fn ty_dyn_enum(
        &mut self,
        id: NodeId,
        name: String,
        discriminant: TypeId,
        alignment: DiscriminantAlignment,
        variants: Vec<(VariantTag, TypeId)>,
    ) -> TypeId {
        self.ty_app(
            id,
            AppType::Enum(AppEnum {
                name,
                variants,
                discriminant,
                alignment,
            }),
        )
    }

    pub fn ty_enum(&mut self, id: NodeId, enumerate: &Enum) -> TypeId {
        let variants: Vec<(VariantTag, TypeId)> = enumerate
            .variants
            .iter()
            .map(|variant| {
                let name = variant.name.clone();
                let ty = self.from_kind(id, &variant.kind);
                let tag = VariantTag {
                    name,
                    discriminant: variant.discriminant,
                    ty: variant.ty,
                };
                (tag, ty)
            })
            .collect();
        let discriminant = self.ty_discriminant(id, enumerate.discriminant_layout);
        self.ty_app(
            id,
            AppType::Enum(AppEnum {
                name: enumerate.name.clone(),
                variants,
                discriminant,
                alignment: enumerate.discriminant_layout.alignment,
            }),
        )
    }

    pub fn ty_tuple(&mut self, id: NodeId, fields: Vec<TypeId>) -> TypeId {
        self.ty_app(id, AppType::Tuple(AppTuple { elements: fields }))
    }

    pub fn ty_index(&mut self, base: TypeId, index: usize) -> Result<TypeId> {
        let TypeKind::App(kind) = &self.types[base.kind] else {
            bail!(
                "Expected an application type, found {:?}",
                self.types[base.kind]
            );
        };
        match kind {
            AppType::Array(array) => Ok(array.base),
            AppType::Tuple(tuple) => tuple
                .elements
                .get(index)
                .cloned()
                .ok_or_else(|| anyhow!("Index out of bounds")),
            AppType::Struct(strukt) => strukt
                .fields
                .get(index)
                .map(|(_, ty)| *ty)
                .ok_or_else(|| anyhow!("Index out of bounds")),
            AppType::Signal(signal) => Ok(self.ty_index(signal.data, index)?),
            _ => bail!("Expected an array, tuple, or struct, found {:?}", kind),
        }
    }

    pub fn ty_variant(&mut self, base: TypeId, variant: &str) -> Result<TypeId> {
        let TypeKind::App(AppType::Enum(enumerate)) = &self.types[base.kind] else {
            bail!("Expected an enum type, found {:?}", self.types[base.kind]);
        };
        enumerate
            .variants
            .iter()
            .find_map(|(v, t)| if v.name == variant { Some(*t) } else { None })
            .ok_or_else(|| anyhow!("Variant not found"))
    }

    pub(crate) fn ty_variant_by_value(&self, base: TypeId, value: i64) -> Result<TypeId> {
        let TypeKind::App(AppType::Enum(enumerate)) = &self.types[base.kind] else {
            bail!("Expected an enum type, found {:?}", self.types[base.kind]);
        };
        enumerate
            .variants
            .iter()
            .find_map(|(v, t)| {
                if v.discriminant == value {
                    Some(*t)
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("Variant not found"))
    }

    pub fn ty_field(&mut self, base: TypeId, member: &str) -> Result<TypeId> {
        let TypeKind::App(AppType::Struct(strukt)) = &self.types[base.kind] else {
            bail!(
                "Expected an a struct type, found {:?}",
                self.types[base.kind]
            );
        };
        strukt
            .fields
            .iter()
            .find_map(|f| if f.0 == member { Some(f.1) } else { None })
            .ok_or_else(|| anyhow!("Field not found"))
    }

    pub fn ty_enum_discriminant(&mut self, base: TypeId) -> TypeId {
        let TypeKind::App(AppType::Enum(enumerate)) = &self.types[base.kind] else {
            return base;
        };
        enumerate.discriminant
    }

    pub fn ty_clock(&mut self, id: NodeId, clock: Color) -> TypeId {
        self.ty_const(id, Const::Clock(clock))
    }

    pub fn ty_empty(&mut self, id: NodeId) -> TypeId {
        self.ty_const(id, Const::Empty)
    }

    pub fn ty_integer(&mut self, id: NodeId) -> TypeId {
        let len = self.ty_var(id);
        let sign_flag = self.ty_var(id);
        self.ty_app(id, AppType::Bits(AppBits { sign_flag, len }))
    }

    fn cast_ty_as_sign_flag(&mut self, ty: TypeId) -> Result<SignFlag> {
        let x = self.apply(ty);
        if let TypeKind::Const(Const::Signed(s)) = &self.types[x.kind] {
            Ok(*s)
        } else {
            bail!("Expected a sign flag, found {:?}", self.types[x.kind]);
        }
    }

    fn cast_ty_as_bit_length(&mut self, ty: TypeId) -> Result<usize> {
        let x = self.apply(ty);
        if let TypeKind::Const(Const::Length(n)) = &self.types[x.kind] {
            Ok(*n)
        } else {
            bail!("Expected a length, found {:?}", self.types[x.kind]);
        }
    }

    pub fn cast_ty_as_clock(&mut self, ty: TypeId) -> Result<Color> {
        let x = self.apply(ty);
        if let TypeKind::Const(Const::Clock(c)) = &self.types[x.kind] {
            Ok(*c)
        } else {
            bail!("Expected a clock, found {:?}", self.types[x.kind]);
        }
    }

    pub fn into_kind(&mut self, ty: TypeId) -> Result<Kind> {
        let x = self.apply(ty);
        match self.types[x.kind].clone() {
            TypeKind::Var(x) => bail!("Unbound variable {:?}", x),
            TypeKind::Const(c) => match c {
                Const::Empty => Ok(Kind::Empty),
                _ => bail!("Expected a constant, found {:?}", c),
            },
            TypeKind::App(app) => app.into_kind(self),
        }
    }

    pub fn from_kind(&mut self, id: NodeId, kind: &Kind) -> TypeId {
        match kind {
            Kind::Bits(n) => {
                let n = self.ty_const_len(id, *n);
                self.ty_bits(id, n)
            }
            Kind::Signed(n) => {
                let n = self.ty_const_len(id, *n);
                self.ty_signed(id, n)
            }
            Kind::Empty => self.ty_empty(id),
            Kind::Struct(strukt) => self.ty_struct(id, strukt),
            Kind::Tuple(fields) => {
                let arg = fields
                    .elements
                    .iter()
                    .map(|k| self.from_kind(id, k))
                    .collect();
                self.ty_tuple(id, arg)
            }
            Kind::Enum(enumerate) => self.ty_enum(id, enumerate),
            Kind::Array(array) => {
                let base = self.from_kind(id, &array.base);
                let len = self.ty_const_len(id, array.size);
                self.ty_array(id, base, len)
            }
            Kind::Signal(kind, clock) => {
                let kind = self.from_kind(id, kind);
                let clock = self.ty_clock(id, *clock);
                self.ty_signal(id, kind, clock)
            }
        }
    }

    fn is_var(&self, ty: TypeId) -> bool {
        matches!(self.types[ty.kind], TypeKind::Var(_))
    }

    pub fn desc(&self, ty: TypeId) -> String {
        match &self.types[ty.kind] {
            TypeKind::Var(v) => format!("V{}", v.0),
            TypeKind::Const(c) => match c {
                Const::Clock(c) => format!("{:?}", c),
                Const::Length(n) => format!("{}", n),
                Const::Signed(f) => {
                    if f.eq(&SignFlag::Signed) {
                        "s".to_string()
                    } else {
                        "b".to_string()
                    }
                }
                Const::Empty => "()".to_string(),
            },
            TypeKind::App(app) => match app {
                AppType::Struct(strukt) => format!(
                    "{}<{}>",
                    strukt.name,
                    strukt
                        .fields
                        .iter()
                        .map(|(f, t)| format!("{}:{}", f, self.desc(*t)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                AppType::Tuple(tuple) => {
                    format!(
                        "{}",
                        tuple
                            .elements
                            .iter()
                            .map(|t| self.desc(*t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
                AppType::Enum(enumerate) => format!(
                    "enum {}<{}>",
                    enumerate.name,
                    enumerate
                        .variants
                        .iter()
                        .map(|(v, t)| format!("{}:{}", v.name, self.desc(*t)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                AppType::Bits(bits) => {
                    format!("{}_{}", self.desc(bits.sign_flag), self.desc(bits.len))
                }
                AppType::Signal(signal) => format!(
                    "signal<{}, {}>",
                    self.desc(signal.data),
                    self.desc(signal.clock)
                ),
                AppType::Array(array) => {
                    format!("[{}; {}]", self.desc(array.base), self.desc(array.len))
                }
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
        let TypeKind::Var(v) = self.types[v.kind] else {
            bail!("Expected a variable, found {:?}", self.types[v.kind]);
        };
        self.substitution_map.insert(v, x);
        Ok(())
    }
    fn subst(&self, ty: TypeId) -> anyhow::Result<Option<TypeId>> {
        let TypeKind::Var(v) = self.types[ty.kind] else {
            bail!("Expected a variable, found {:?}", self.types[ty.kind]);
        };
        if let Some(t) = self.substitution_map.get(&v) {
            return Ok(Some(*t));
        }
        Ok(None)
    }
    pub fn equal(&mut self, x: TypeId, y: TypeId) -> bool {
        let x = self.apply(x);
        let y = self.apply(y);
        self.types[x.kind] == self.types[y.kind]
    }
    pub fn is_unresolved(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        self.is_var(ty)
    }
    pub fn is_unsized_integer(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Bits(bits)) = &self.types[ty.kind] {
            return self.is_var(bits.len);
        }
        false
    }
    pub fn is_generic_integer(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Bits(bits)) = &self.types[ty.kind] {
            return self.is_var(bits.sign_flag) && self.is_var(bits.len);
        }
        false
    }
    pub fn is_signal(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        matches!(&self.types[ty.kind], TypeKind::App(AppType::Signal(_)))
    }
    pub fn project_signal_clock(&mut self, ty: TypeId) -> Option<TypeId> {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Signal(signal)) = &self.types[ty.kind] {
            Some(signal.clock)
        } else {
            None
        }
    }
    pub fn project_signal_value(&mut self, ty: TypeId) -> Option<TypeId> {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Signal(signal)) = &self.types[ty.kind] {
            Some(signal.data)
        } else {
            None
        }
    }
    pub fn project_sign_flag(&mut self, ty: TypeId) -> Option<TypeId> {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Bits(bits)) = &self.types[ty.kind] {
            Some(bits.sign_flag)
        } else {
            None
        }
    }
    pub fn apply(&mut self, ty: TypeId) -> TypeId {
        match self.types[ty.kind].clone() {
            TypeKind::Var(v) => {
                if let Some(t) = self.substitution_map.get(&v) {
                    self.apply(*t)
                } else {
                    ty
                }
            }
            TypeKind::App(app) => {
                let app = app.apply(self);
                self.ty_app(ty.id, app)
            }
            _ => ty,
        }
    }
    pub fn unify(&mut self, x: TypeId, y: TypeId) -> Result<()> {
        if self.types[x.kind] == self.types[y.kind] {
            return Ok(());
        }
        match (&self.types[x.kind], &self.types[y.kind]) {
            (TypeKind::Var(_), _) => self.unify_variable(x, y),
            (_, TypeKind::Var(_)) => self.unify_variable(y, x),
            (TypeKind::Const(x), TypeKind::Const(y)) if x == y => Ok(()),
            (TypeKind::App(_), TypeKind::App(_)) => self.unify_app(x, y),
            _ => bail!("Cannot unify {} and {}", self.desc(x), self.desc(y)),
        }
    }
    // We want to declare v and x as equivalent.
    fn unify_variable(&mut self, v: TypeId, x: TypeId) -> Result<()> {
        ensure!(
            self.is_var(v),
            "Expected a variable, found {:?}",
            self.types[v.kind]
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
    fn unify_tuple(&mut self, x: &AppTuple, y: &AppTuple) -> Result<()> {
        if x.elements.len() != y.elements.len() {
            bail!("Cannot unify {:?} and {:?}", x, y);
        }
        for (a, b) in x.elements.iter().zip(y.elements.iter()) {
            self.unify(*a, *b)?;
        }
        Ok(())
    }
    fn unify_array(&mut self, x: &AppArray, y: &AppArray) -> Result<()> {
        self.unify(x.base, y.base)?;
        self.unify(x.len, y.len)
    }
    fn unify_struct(&mut self, x: &AppStruct, y: &AppStruct) -> Result<()> {
        if x.name != y.name {
            bail!("Cannot unify {:?} and {:?}", x, y);
        }
        if x.fields.len() != y.fields.len() {
            bail!("Cannot unify {:?} and {:?}", x, y);
        }
        for (a, b) in x.fields.iter().zip(y.fields.iter()) {
            if a.0 != b.0 {
                bail!("Cannot unify {:?} and {:?}", a, b);
            }
            self.unify(a.1, b.1)?;
        }
        Ok(())
    }
    fn unify_enum(&mut self, x: &AppEnum, y: &AppEnum) -> Result<()> {
        if x.name != y.name {
            bail!("Cannot unify {:?} and {:?}", x, y);
        }
        if x.variants.len() != y.variants.len() {
            bail!("Cannot unify {:?} and {:?}", x, y);
        }
        for (a, b) in x.variants.iter().zip(y.variants.iter()) {
            if a.0 != b.0 {
                bail!("Cannot unify {:?} and {:?}", a, b);
            }
            self.unify(a.1, b.1)?;
        }
        self.unify(x.discriminant, y.discriminant)
    }
    fn unify_bits(&mut self, x: &AppBits, y: &AppBits) -> Result<()> {
        self.unify(x.sign_flag, y.sign_flag)?;
        self.unify(x.len, y.len)
    }
    fn unify_signal(&mut self, x: &AppSignal, y: &AppSignal) -> Result<()> {
        self.unify(x.data, y.data)?;
        self.unify(x.clock, y.clock)
    }
    fn unify_app(&mut self, x: TypeId, y: TypeId) -> Result<()> {
        let TypeKind::App(app1) = self.types[x.kind].clone() else {
            bail!("Expected app type instead of {:?}", self.types[x.kind]);
        };
        let TypeKind::App(app2) = self.types[y.kind].clone() else {
            bail!("Expected app type instead of {:?}", self.types[y.kind]);
        };
        match (&app1, &app2) {
            (AppType::Tuple(t1), AppType::Tuple(t2)) => self.unify_tuple(t1, t2),
            (AppType::Array(a1), AppType::Array(a2)) => self.unify_array(a1, a2),
            (AppType::Struct(s1), AppType::Struct(s2)) => self.unify_struct(s1, s2),
            (AppType::Enum(e1), AppType::Enum(e2)) => self.unify_enum(e1, e2),
            (AppType::Bits(b1), AppType::Bits(b2)) => self.unify_bits(b1, b2),
            (AppType::Signal(s1), AppType::Signal(s2)) => self.unify_signal(s1, s2),
            _ => bail!("Cannot unify {:?} and {:?}", app1, app2),
        }
    }
    fn occurs(&self, v: TypeId, term: TypeId) -> bool {
        // Check for the case that term is a variable
        if self.is_var(term) {
            // Unifying with itself is not allowed
            if self.types[term.kind] == self.types[v.kind] {
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
        if let TypeKind::App(app_type) = &self.types[term.kind] {
            return app_type.sub_types().iter().any(|t| self.occurs(v, *t));
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let mut ctx = UnifyContext::default();
        let id = NodeId::new(0);
        let x = ctx.ty_var(id);
        let y = ctx.ty_var(id);
        let z = ctx.ty_var(id);
        let t = ctx.ty_tuple(id, vec![x, y, z]);
        let a = ctx.ty_integer(id);
        let b = ctx.ty_usize(id);
        let c = ctx.from_kind(id, &Kind::Bits(128));
        let u = ctx.ty_tuple(id, vec![a, b, c]);
        assert!(ctx.unify(t, u).is_ok());
        let x = ctx.apply(x);
        let y = ctx.apply(y);
        let z = ctx.apply(z);
        assert_eq!(ctx.ty(x), ctx.ty(a));
        assert_eq!(ctx.ty(y), ctx.ty(b));
        assert_eq!(ctx.ty(z), ctx.ty(c));
    }

    #[test]
    fn test_case_2() {
        let mut ctx = UnifyContext::default();
        let id = NodeId::new(0);
        let n = ctx.ty_const_len(id, 12);
        let x = ctx.ty_bits(id, n);
        let m = ctx.ty_var(id);
        let y = ctx.ty_bits(id, m);
        let z = ctx.ty_var(id);
        assert!(ctx.unify(x, y).is_ok());
        assert!(ctx.unify(x, z).is_ok());
        eprintln!("{}", ctx);
        let m = ctx.into_kind(z).unwrap();
        println!("{:?}", m);
    }
}
