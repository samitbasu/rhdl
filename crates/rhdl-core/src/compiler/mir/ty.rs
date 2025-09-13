use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    iter::once,
};

use crate::{
    RHDLError,
    {
        Color, DiscriminantAlignment, DiscriminantType, Kind,
        ast::source::source_location::SourceLocation,
        rhif::spec::Member,
        types::kind::{DiscriminantLayout, Enum, Field, Struct},
    },
};
use internment::Intern;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VarNum(u32);

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
    String(Intern<String>),
    Unclocked,
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

#[derive(Error, Debug, Diagnostic)]
pub enum UnifyError {
    #[error("Expected a struct, found {kind:?}")]
    ExpectedStructTypeButFound { kind: TypeKind },
    #[error("Expected an enum, found {kind:?}")]
    ExpectedEnumTypeButFound { kind: TypeKind },
    #[error("Expected a variable, found {kind:?}")]
    ExpectedVariableButFound { kind: TypeKind },
    #[error("Expected a sign flag, found {kind:?}")]
    ExpectedSignFlagButFound { kind: TypeKind },
    #[error("Expected a length, found {kind:?}")]
    ExpectedLengthButFound { kind: TypeKind },
    #[error("Expected a clock, found {kind:?}")]
    ExpectedClockButFound { kind: TypeKind },
    #[error("Unbound variable {kind:?}")]
    UnboundVariable { kind: TypeKind },
    #[error("Expected constant {kind:?}")]
    ExpectedConstant { kind: TypeKind },
    #[error("Cannot unify {x_kind:?} and {y_kind:?}")]
    CannotUnifyKinds { x_kind: TypeKind, y_kind: TypeKind },
    #[error("Recursive unification encountered {v:?}")]
    RecursiveUnification { v: TypeKind },
    #[error("Cannot unify tuples of different length {x:?} and {y:?}")]
    CannotUnifyDifferentSizeTuples { x: AppTuple, y: AppTuple },
    #[error("Cannot unify structs {x:?} and {y:?}")]
    CannotUnifyStructs { x: AppStruct, y: AppStruct },
    #[error("Cannot unify fields with names {x} and {y}")]
    CannotUnifyFieldsWithNames {
        x: Intern<String>,
        y: Intern<String>,
    },
    #[error("Cannot unify enums with different variants {x:?} and {y:?}")]
    CannotUnifyEnums { x: AppEnum, y: AppEnum },
    #[error("Cannot unify variants of tags {x:?} and {y:?}")]
    CannotUnifyVariants { x: VariantTag, y: VariantTag },
    #[error("Expected app type instead of {x:?}")]
    ExpectedAppType { x: TypeKind },
    #[error("Cannot unify applicative types {x:?} and {y:?}")]
    CannotUnifyApplicativeTypes { x: AppType, y: AppType },
    #[error("Cannot unify {x:?} and unclocked")]
    CannotUnifyUnclocked { x: AppType },
    #[error("Expected a string, but found {x:?}")]
    ExpectedStringNot { x: TypeKind },
    #[error("Field {field} missing in struct definition")]
    MissingFieldInStructDefinition { field: String },
    #[error("Index {index} out of bounds")]
    IndexOutOfBounds { index: usize },
    #[error("Expected an array, tuple or struct, found {kind:?}")]
    ExpectedArrayTupleOrStruct { kind: AppType },
    #[error("Variant {tag:?} not found")]
    VariantNotFound { tag: String },
    #[error("Variant with discriminant {tag:?} not found")]
    VariantWithDiscriminantNotFound { tag: i64 },
}

pub trait AppTypeKind {
    fn sub_types(&self) -> Vec<TypeId>;
    fn apply(self, context: &mut UnifyContext) -> Self;
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError>;
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
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError> {
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
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError> {
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
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError> {
        let data = context.into_kind(self.data)?;
        let clock = context.cast_ty_as_clock(self.clock)?;
        Ok(Kind::Signal(internment::Intern::new(data), clock))
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
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError> {
        let base = context.into_kind(self.base)?;
        let size = context.cast_ty_as_bit_length(self.len)?;
        Ok(Kind::make_array(base, size))
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
    name: Intern<String>,
    fields: Vec<(Intern<String>, TypeId)>,
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
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError> {
        let fields = self
            .fields
            .into_iter()
            .map(|(name, t)| {
                let kind = context.into_kind(t)?;
                Ok(Field {
                    name: name.into(),
                    kind,
                })
            })
            .collect::<Result<_, RHDLError>>()?;
        Ok(Kind::make_struct(&self.name, fields))
    }
}

// An enum is generic over the discriminant type and
// the variants themselves.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppEnum {
    name: TypeId,
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
            name: context.apply(self.name),
            discriminant: context.apply(self.discriminant),
            variants: self
                .variants
                .into_iter()
                .map(|(v, t)| (v, context.apply(t)))
                .collect(),
            ..self
        }
    }
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError> {
        let name = context.apply_string(self.name)?.to_owned();
        let variants = self
            .variants
            .into_iter()
            .map(|(v, t)| {
                let kind = context.into_kind(t)?;
                Ok(crate::types::kind::Variant {
                    name: v.name.into(),
                    kind,
                    discriminant: v.discriminant,
                })
            })
            .collect::<Result<Vec<_>, RHDLError>>()?;
        let discriminant_kind = context.into_kind(self.discriminant)?;
        Ok(Kind::make_enum(
            &name,
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
    fn into_kind(self, context: &mut UnifyContext) -> Result<Kind, RHDLError> {
        let elements = self
            .elements
            .into_iter()
            .map(|t| context.into_kind(t))
            .collect::<Result<Vec<_>, RHDLError>>()?;
        Ok(Kind::make_tuple(elements))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariantTag {
    name: Intern<String>,
    discriminant: i64,
}

pub fn make_variant_tag(name: &str, discriminant: i64) -> VariantTag {
    VariantTag {
        name: name.to_string().into(),
        discriminant,
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
    pub kind: Intern<TypeKind>,
    pub loc: SourceLocation,
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
}

pub struct UnifyContext {
    substitution_map: HashMap<VarNum, TypeId>,
    var: VarNum,
    update_count: usize,
}

impl Default for UnifyContext {
    fn default() -> Self {
        let substitution_map = HashMap::new();
        let var = VarNum(0);
        Self {
            substitution_map,
            var,
            update_count: 0,
        }
    }
}

impl UnifyContext {
    pub fn modification_state(&self) -> ModificationState {
        ModificationState {
            update_count: self.update_count,
        }
    }

    fn ty_app(&mut self, loc: SourceLocation, kind: AppType) -> TypeId {
        TypeId {
            loc,
            kind: Intern::new(TypeKind::App(kind)),
        }
    }

    pub fn ty_const(&mut self, loc: SourceLocation, const_ty: Const) -> TypeId {
        TypeId {
            loc,
            kind: Intern::new(TypeKind::Const(const_ty)),
        }
    }

    pub fn ty_const_len(&mut self, loc: SourceLocation, len: usize) -> TypeId {
        self.ty_const(loc, Const::Length(len))
    }

    pub fn ty_bool(&mut self, loc: SourceLocation) -> TypeId {
        let n = self.ty_const_len(loc, 1);
        self.ty_bits(loc, n)
    }

    pub fn ty_usize(&mut self, loc: SourceLocation) -> TypeId {
        let n = self.ty_const_len(loc, 64);
        self.ty_bits(loc, n)
    }

    pub fn ty_sign_flag(&mut self, loc: SourceLocation, sign_flag: SignFlag) -> TypeId {
        self.ty_const(loc, Const::Signed(sign_flag))
    }

    pub fn ty_bits(&mut self, loc: SourceLocation, len: TypeId) -> TypeId {
        let sign_flag = self.ty_sign_flag(loc, SignFlag::Unsigned);
        self.ty_app(loc, AppType::Bits(AppBits { sign_flag, len }))
    }

    pub fn ty_signed(&mut self, loc: SourceLocation, len: TypeId) -> TypeId {
        let sign_flag = self.ty_sign_flag(loc, SignFlag::Signed);
        self.ty_app(loc, AppType::Bits(AppBits { sign_flag, len }))
    }

    pub fn ty_signal(&mut self, loc: SourceLocation, data: TypeId, clock: TypeId) -> TypeId {
        self.ty_app(loc, AppType::Signal(AppSignal { data, clock }))
    }

    pub fn ty_result(&mut self, loc: SourceLocation, ok_ty: TypeId, err_ty: TypeId) -> TypeId {
        let discriminant = self.ty_bool(loc);
        let name = self.ty_var(loc);
        let err_ty = self.ty_tuple(loc, vec![err_ty]);
        let ok_ty = self.ty_tuple(loc, vec![ok_ty]);
        self.ty_app(
            loc,
            AppType::Enum(AppEnum {
                name,
                discriminant,
                alignment: DiscriminantAlignment::Msb,
                variants: vec![
                    (make_variant_tag("Err", 0), err_ty),
                    (make_variant_tag("Ok", 1), ok_ty),
                ],
            }),
        )
    }

    pub fn ty_option(&mut self, loc: SourceLocation, some_ty: TypeId) -> TypeId {
        let discriminant = self.ty_bool(loc);
        let none_ty = self.ty_empty(loc);
        let name = self.ty_var(loc);
        let some_ty = self.ty_tuple(loc, vec![some_ty]);
        self.ty_app(
            loc,
            AppType::Enum(AppEnum {
                name,
                discriminant,
                alignment: DiscriminantAlignment::Msb,
                variants: vec![
                    (make_variant_tag("None", 0), none_ty),
                    (make_variant_tag("Some", 1), some_ty),
                ],
            }),
        )
    }

    pub fn ty_with_sign_and_len(
        &mut self,
        loc: SourceLocation,
        sign_flag: TypeId,
        len: TypeId,
    ) -> TypeId {
        self.ty_app(loc, AppType::Bits(AppBits { sign_flag, len }))
    }

    pub fn ty_maybe_signed(&mut self, loc: SourceLocation, len: TypeId) -> TypeId {
        let sign_flag = self.ty_var(loc);
        self.ty_app(loc, AppType::Bits(AppBits { sign_flag, len }))
    }

    pub fn ty_var(&mut self, loc: SourceLocation) -> TypeId {
        let ty = TypeId {
            loc,
            kind: Intern::new(TypeKind::Var(self.var)),
        };
        self.var.0 += 1;
        ty
    }

    pub fn ty_array(&mut self, loc: SourceLocation, base: TypeId, len: TypeId) -> TypeId {
        self.ty_app(loc, AppType::Array(AppArray { base, len }))
    }

    pub fn ty_struct(&mut self, loc: SourceLocation, strukt: &Struct) -> TypeId {
        let fields: Vec<(Intern<String>, TypeId)> = strukt
            .fields
            .iter()
            .map(|field| {
                let name = field.name;
                let ty = self.from_kind(loc, field.kind);
                (name, ty)
            })
            .collect();
        self.ty_app(
            loc,
            AppType::Struct(AppStruct {
                name: strukt.name,
                fields,
            }),
        )
    }

    pub fn ty_dyn_struct(
        &mut self,
        loc: SourceLocation,
        name: Intern<String>,
        fields: Vec<(Intern<String>, TypeId)>,
    ) -> TypeId {
        self.ty_app(loc, AppType::Struct(AppStruct { name, fields }))
    }

    fn ty_discriminant(&mut self, loc: SourceLocation, layout: DiscriminantLayout) -> TypeId {
        let len = self.ty_const_len(loc, layout.width);
        match layout.ty {
            DiscriminantType::Unsigned => self.ty_bits(loc, len),
            DiscriminantType::Signed => self.ty_signed(loc, len),
        }
    }

    pub fn ty_dyn_enum(
        &mut self,
        loc: SourceLocation,
        name: Intern<String>,
        discriminant: TypeId,
        alignment: DiscriminantAlignment,
        variants: Vec<(VariantTag, TypeId)>,
    ) -> TypeId {
        let name = self.ty_const(loc, Const::String(name));
        self.ty_app(
            loc,
            AppType::Enum(AppEnum {
                name,
                variants,
                discriminant,
                alignment,
            }),
        )
    }

    pub fn ty_enum(&mut self, loc: SourceLocation, enumerate: &Enum) -> TypeId {
        let variants: Vec<(VariantTag, TypeId)> = enumerate
            .variants
            .iter()
            .map(|variant| {
                let name = variant.name.clone();
                let ty = self.from_kind(loc, variant.kind);
                let tag = VariantTag {
                    name,
                    discriminant: variant.discriminant,
                };
                (tag, ty)
            })
            .collect();
        let discriminant = self.ty_discriminant(loc, enumerate.discriminant_layout);
        let name = self.ty_const(loc, Const::String(enumerate.name.clone()));
        self.ty_app(
            loc,
            AppType::Enum(AppEnum {
                name,
                variants,
                discriminant,
                alignment: enumerate.discriminant_layout.alignment,
            }),
        )
    }

    pub fn ty_tuple(&mut self, loc: SourceLocation, fields: Vec<TypeId>) -> TypeId {
        if fields.is_empty() {
            self.ty_empty(loc)
        } else {
            self.ty_app(loc, AppType::Tuple(AppTuple { elements: fields }))
        }
    }

    fn apply_string(&mut self, ty: TypeId) -> Result<&str, RHDLError> {
        let x = self.apply(ty);
        if let TypeKind::Const(Const::String(s)) = x.kind.as_ref() {
            Ok(s)
        } else {
            Err(self.raise_type_error(UnifyError::ExpectedStringNot {
                x: x.kind.as_ref().clone(),
            }))
        }
    }

    pub fn ty_index(&mut self, base: TypeId, index: usize) -> Result<TypeId, RHDLError> {
        let TypeKind::App(kind) = base.kind.as_ref() else {
            return Err(self.raise_type_error(UnifyError::ExpectedAppType {
                x: base.kind.as_ref().clone(),
            }));
        };
        match kind {
            AppType::Array(array) => Ok(array.base),
            AppType::Tuple(tuple) => tuple
                .elements
                .get(index)
                .cloned()
                .ok_or_else(|| self.raise_type_error(UnifyError::IndexOutOfBounds { index })),
            AppType::Struct(strukt) => strukt
                .fields
                .get(index)
                .map(|(_, ty)| *ty)
                .ok_or_else(|| self.raise_type_error(UnifyError::IndexOutOfBounds { index })),
            AppType::Signal(signal) => Ok(self.ty_index(signal.data, index)?),
            _ => Err(self
                .raise_type_error(UnifyError::ExpectedArrayTupleOrStruct { kind: kind.clone() })),
        }
    }

    pub fn ty_variant(&mut self, base: TypeId, variant: &str) -> Result<TypeId, RHDLError> {
        let TypeKind::App(AppType::Enum(enumerate)) = base.kind.as_ref() else {
            return Err(self.raise_type_error(UnifyError::ExpectedEnumTypeButFound {
                kind: base.kind.as_ref().clone(),
            }));
        };
        enumerate
            .variants
            .iter()
            .find_map(|(v, t)| if *v.name == variant { Some(*t) } else { None })
            .ok_or_else(|| {
                self.raise_type_error(UnifyError::VariantNotFound {
                    tag: variant.to_string(),
                })
            })
    }

    pub(crate) fn ty_variant_by_value(
        &self,
        base: TypeId,
        value: i64,
    ) -> Result<TypeId, RHDLError> {
        let TypeKind::App(AppType::Enum(enumerate)) = base.kind.as_ref() else {
            return Err(self.raise_type_error(UnifyError::ExpectedEnumTypeButFound {
                kind: base.kind.as_ref().clone(),
            }));
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
            .ok_or_else(|| {
                self.raise_type_error(UnifyError::VariantWithDiscriminantNotFound { tag: value })
            })
    }

    pub fn ty_field(&mut self, base: TypeId, member: &str) -> Result<TypeId, RHDLError> {
        let TypeKind::App(AppType::Struct(strukt)) = base.kind.as_ref() else {
            return Err(
                self.raise_type_error(UnifyError::ExpectedStructTypeButFound {
                    kind: base.kind.as_ref().clone(),
                }),
            );
        };
        strukt
            .fields
            .iter()
            .find_map(|f| if *f.0 == member { Some(f.1) } else { None })
            .ok_or_else(|| {
                self.raise_type_error(UnifyError::MissingFieldInStructDefinition {
                    field: member.to_string(),
                })
            })
    }

    pub fn ty_member(&mut self, base: TypeId, member: &Member) -> Result<TypeId, RHDLError> {
        match member {
            Member::Named(name) => self.ty_field(base, name),
            Member::Unnamed(index) => self.ty_index(base, *index as usize),
        }
    }

    pub fn ty_enum_discriminant(&mut self, base: TypeId) -> TypeId {
        let TypeKind::App(AppType::Enum(enumerate)) = base.kind.as_ref() else {
            return base;
        };
        enumerate.discriminant
    }

    pub fn ty_unclocked(&mut self, loc: SourceLocation) -> TypeId {
        self.ty_const(loc, Const::Unclocked)
    }

    pub fn ty_clock(&mut self, loc: SourceLocation, clock: Color) -> TypeId {
        self.ty_const(loc, Const::Clock(clock))
    }

    pub fn ty_empty(&mut self, loc: SourceLocation) -> TypeId {
        self.ty_const(loc, Const::Empty)
    }

    pub fn ty_integer(&mut self, loc: SourceLocation) -> TypeId {
        let len = self.ty_var(loc);
        let sign_flag = self.ty_var(loc);
        self.ty_app(loc, AppType::Bits(AppBits { sign_flag, len }))
    }

    fn cast_ty_as_sign_flag(&mut self, ty: TypeId) -> Result<SignFlag, RHDLError> {
        let x = self.apply(ty);
        if let TypeKind::Const(Const::Signed(s)) = x.kind.as_ref() {
            Ok(*s)
        } else {
            Err(self.raise_type_error(UnifyError::ExpectedSignFlagButFound {
                kind: x.kind.as_ref().clone(),
            }))
        }
    }

    pub fn cast_ty_as_bit_length(&mut self, ty: TypeId) -> Result<usize, RHDLError> {
        let x = self.apply(ty);
        if let TypeKind::Const(Const::Length(n)) = x.kind.as_ref() {
            Ok(*n)
        } else {
            Err(self.raise_type_error(UnifyError::ExpectedLengthButFound {
                kind: x.kind.as_ref().clone(),
            }))
        }
    }

    pub fn cast_ty_as_clock(&mut self, ty: TypeId) -> Result<Color, RHDLError> {
        let x = self.apply(ty);
        if let TypeKind::Const(Const::Clock(c)) = x.kind.as_ref() {
            Ok(*c)
        } else {
            Err(self.raise_type_error(UnifyError::ExpectedClockButFound {
                kind: x.kind.as_ref().clone(),
            }))
        }
    }

    pub fn into_kind(&mut self, ty: TypeId) -> Result<Kind, RHDLError> {
        let x = self.apply(ty);
        match x.kind.as_ref() {
            TypeKind::Var(_) => {
                return Err(self.raise_type_error(UnifyError::UnboundVariable {
                    kind: x.kind.as_ref().clone(),
                }));
            }
            TypeKind::Const(c) => match c {
                Const::Empty => Ok(Kind::Empty),
                _ => {
                    return Err(self.raise_type_error(UnifyError::ExpectedConstant {
                        kind: x.kind.as_ref().clone(),
                    }));
                }
            },
            TypeKind::App(app) => app.clone().into_kind(self),
        }
    }

    pub fn from_kind(&mut self, loc: SourceLocation, kind: Kind) -> TypeId {
        match kind {
            Kind::Bits(n) => {
                let n = self.ty_const_len(loc, n);
                self.ty_bits(loc, n)
            }
            Kind::Signed(n) => {
                let n = self.ty_const_len(loc, n);
                self.ty_signed(loc, n)
            }
            Kind::Empty => self.ty_empty(loc),
            Kind::Struct(strukt) => self.ty_struct(loc, &strukt),
            Kind::Tuple(fields) => {
                let arg = fields
                    .elements
                    .iter()
                    .map(|k| self.from_kind(loc, *k))
                    .collect();
                self.ty_tuple(loc, arg)
            }
            Kind::Enum(enumerate) => self.ty_enum(loc, &enumerate),
            Kind::Array(array) => {
                let base = self.from_kind(loc, *array.base);
                let len = self.ty_const_len(loc, array.size);
                self.ty_array(loc, base, len)
            }
            Kind::Signal(kind, clock) => {
                let kind = self.from_kind(loc, *kind);
                let clock = self.ty_clock(loc, clock);
                self.ty_signal(loc, kind, clock)
            }
        }
    }

    fn is_var(&self, ty: TypeId) -> bool {
        matches!(ty.kind.as_ref(), TypeKind::Var(_))
    }

    pub fn desc(&self, ty: TypeId) -> String {
        match ty.kind.as_ref() {
            TypeKind::Var(v) => format!("V{}", v.0),
            TypeKind::Const(c) => match c {
                Const::Clock(c) => format!("{c:?}"),
                Const::Length(n) => format!("{n}"),
                Const::Signed(f) => {
                    if f.eq(&SignFlag::Signed) {
                        "s".to_string()
                    } else {
                        "b".to_string()
                    }
                }
                Const::String(s) => s.to_string(),
                Const::Empty => "()".to_string(),
                Const::Unclocked => "*".to_string(),
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
                        "({})",
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
                    self.desc(enumerate.name),
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
    fn raise_type_error(&self, cause: UnifyError) -> RHDLError {
        Box::new(cause).into()
    }
    fn add_subst(&mut self, v: TypeId, x: TypeId) -> Result<(), RHDLError> {
        let TypeKind::Var(v) = v.kind.as_ref() else {
            return Err(self.raise_type_error(UnifyError::ExpectedVariableButFound {
                kind: v.kind.as_ref().clone(),
            }));
        };
        self.substitution_map.insert(*v, x);
        Ok(())
    }
    fn subst(&self, ty: TypeId) -> Result<Option<TypeId>, RHDLError> {
        let TypeKind::Var(v) = ty.kind.as_ref() else {
            return Err(self.raise_type_error(UnifyError::ExpectedVariableButFound {
                kind: ty.kind.as_ref().clone(),
            }));
        };
        if let Some(t) = self.substitution_map.get(&v) {
            return Ok(Some(*t));
        }
        Ok(None)
    }
    pub fn equal(&mut self, x: TypeId, y: TypeId) -> bool {
        let x = self.apply(x);
        let y = self.apply(y);
        x.kind == y.kind
    }
    pub fn is_unresolved(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        self.is_var(ty)
    }
    pub fn is_unsized_integer(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Bits(bits)) = ty.kind.as_ref() {
            return self.is_var(bits.len);
        }
        false
    }
    pub fn is_generic_integer(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Bits(bits)) = ty.kind.as_ref() {
            return self.is_var(bits.sign_flag) && self.is_var(bits.len);
        }
        false
    }
    pub fn is_signal(&mut self, ty: TypeId) -> bool {
        let ty = self.apply(ty);
        matches!(ty.kind.as_ref(), TypeKind::App(AppType::Signal(_)))
    }
    pub fn project_signal_clock(&mut self, ty: TypeId) -> Option<TypeId> {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Signal(signal)) = ty.kind.as_ref() {
            Some(signal.clock)
        } else {
            None
        }
    }
    pub fn project_signal_value(&mut self, ty: TypeId) -> Option<TypeId> {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Signal(signal)) = ty.kind.as_ref() {
            Some(signal.data)
        } else {
            None
        }
    }
    pub fn project_signal_data(&mut self, ty: TypeId) -> TypeId {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Signal(signal)) = ty.kind.as_ref() {
            signal.data
        } else {
            ty
        }
    }
    pub fn project_bit_length(&mut self, ty: TypeId) -> Option<TypeId> {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Bits(bits)) = ty.kind.as_ref() {
            Some(bits.len)
        } else {
            None
        }
    }
    pub fn project_sign_flag(&mut self, ty: TypeId) -> Option<TypeId> {
        let ty = self.apply(ty);
        if let TypeKind::App(AppType::Bits(bits)) = ty.kind.as_ref() {
            Some(bits.sign_flag)
        } else {
            None
        }
    }
    pub fn apply(&mut self, ty: TypeId) -> TypeId {
        match (*ty.kind).clone() {
            TypeKind::Var(v) => {
                if let Some(t) = self.substitution_map.get(&v) {
                    self.apply(*t)
                } else {
                    ty
                }
            }
            TypeKind::App(app) => {
                let app = app.apply(self);
                self.ty_app(ty.loc, app)
            }
            _ => ty,
        }
    }
    pub fn unify(&mut self, x: TypeId, y: TypeId) -> Result<(), RHDLError> {
        if x.kind == y.kind {
            return Ok(());
        }
        match (x.kind.as_ref(), y.kind.as_ref()) {
            (TypeKind::Var(_), _) => self.unify_variable(x, y),
            (_, TypeKind::Var(_)) => self.unify_variable(y, x),
            (TypeKind::Const(Const::Unclocked), TypeKind::Const(Const::Clock(_)))
            | (TypeKind::Const(Const::Clock(_)), TypeKind::Const(Const::Unclocked)) => Ok(()),
            (TypeKind::App(_), TypeKind::Const(Const::Unclocked)) => self.unify_app_unclocked(x, y),
            (TypeKind::Const(Const::Unclocked), TypeKind::App(_)) => self.unify_app_unclocked(y, x),
            (TypeKind::Const(x), TypeKind::Const(y)) if x == y => Ok(()),
            (TypeKind::App(_), TypeKind::App(_)) => self.unify_app(x, y),
            _ => Err(self.raise_type_error(UnifyError::CannotUnifyKinds {
                x_kind: x.kind.as_ref().clone(),
                y_kind: y.kind.as_ref().clone(),
            })),
        }
    }
    // We want to declare v and x as equivalent.
    fn unify_variable(&mut self, v: TypeId, x: TypeId) -> Result<(), RHDLError> {
        if !self.is_var(v) {
            return Err(self.raise_type_error(UnifyError::ExpectedVariableButFound {
                kind: v.kind.as_ref().clone(),
            }));
        }
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
            return Err(self.raise_type_error(UnifyError::RecursiveUnification {
                v: v.kind.as_ref().clone(),
            }));
        }
        // All is good, so add the substitution to the map.
        self.add_subst(v, x)
    }
    fn unify_unclocked_tuple(&mut self, x: &AppTuple, y: TypeId) -> Result<(), RHDLError> {
        if x.elements.is_empty() {
            return Ok(());
        }
        for a in x.elements.iter() {
            self.unify(*a, y)?;
        }
        Ok(())
    }
    fn unify_tuple(&mut self, x: &AppTuple, y: &AppTuple) -> Result<(), RHDLError> {
        if x.elements.len() != y.elements.len() {
            return Err(
                self.raise_type_error(UnifyError::CannotUnifyDifferentSizeTuples {
                    x: x.clone(),
                    y: y.clone(),
                }),
            );
        }
        for (a, b) in x.elements.iter().zip(y.elements.iter()) {
            self.unify(*a, *b)?;
        }
        Ok(())
    }
    fn unify_unclocked_array(&mut self, x: &AppArray, y: TypeId) -> Result<(), RHDLError> {
        self.unify(x.base, y)
    }
    fn unify_array(&mut self, x: &AppArray, y: &AppArray) -> Result<(), RHDLError> {
        self.unify(x.base, y.base)?;
        self.unify(x.len, y.len)
    }
    fn unify_unclocked_struct(&mut self, x: &AppStruct, y: TypeId) -> Result<(), RHDLError> {
        for (_, t) in x.fields.iter() {
            self.unify(*t, y)?;
        }
        Ok(())
    }
    fn unify_struct(&mut self, x: &AppStruct, y: &AppStruct) -> Result<(), RHDLError> {
        if x.name != y.name {
            return Err(self.raise_type_error(UnifyError::CannotUnifyStructs {
                x: x.clone(),
                y: y.clone(),
            }));
        }
        if x.fields.len() != y.fields.len() {
            return Err(self.raise_type_error(UnifyError::CannotUnifyStructs {
                x: x.clone(),
                y: y.clone(),
            }));
        }
        for (a, b) in x.fields.iter().zip(y.fields.iter()) {
            if a.0 != b.0 {
                return Err(
                    self.raise_type_error(UnifyError::CannotUnifyFieldsWithNames {
                        x: a.0.clone(),
                        y: b.0.clone(),
                    }),
                );
            }
            self.unify(a.1, b.1)?;
        }
        Ok(())
    }
    fn unify_unclocked_enum(&mut self, x: &AppEnum, y: TypeId) -> Result<(), RHDLError> {
        self.unify(x.discriminant, y)?;
        for (_, t) in x.variants.iter() {
            self.unify(*t, y)?;
        }
        Ok(())
    }
    fn unify_enum(&mut self, x: &AppEnum, y: &AppEnum) -> Result<(), RHDLError> {
        self.unify(x.name, y.name)?;
        if x.variants.len() != y.variants.len() {
            return Err(self.raise_type_error(UnifyError::CannotUnifyEnums {
                x: x.clone(),
                y: y.clone(),
            }));
        }
        for (a, b) in x.variants.iter().zip(y.variants.iter()) {
            if a.0 != b.0 {
                return Err(self.raise_type_error(UnifyError::CannotUnifyVariants {
                    x: a.0.clone(),
                    y: b.0.clone(),
                }));
            }
            self.unify(a.1, b.1)?;
        }
        self.unify(x.discriminant, y.discriminant)
    }
    fn unify_bits(&mut self, x: &AppBits, y: &AppBits) -> Result<(), RHDLError> {
        self.unify(x.sign_flag, y.sign_flag)?;
        self.unify(x.len, y.len)
    }
    fn unify_signal(&mut self, x: &AppSignal, y: &AppSignal) -> Result<(), RHDLError> {
        self.unify(x.data, y.data)?;
        self.unify(x.clock, y.clock)
    }
    fn unify_app(&mut self, x: TypeId, y: TypeId) -> Result<(), RHDLError> {
        let TypeKind::App(app1) = (*x.kind).clone() else {
            return Err(self.raise_type_error(UnifyError::ExpectedAppType {
                x: x.kind.as_ref().clone(),
            }));
        };
        let TypeKind::App(app2) = (*y.kind).clone() else {
            return Err(self.raise_type_error(UnifyError::ExpectedAppType {
                x: y.kind.as_ref().clone(),
            }));
        };
        match (&app1, &app2) {
            (AppType::Tuple(t1), AppType::Tuple(t2)) => self.unify_tuple(t1, t2),
            (AppType::Array(a1), AppType::Array(a2)) => self.unify_array(a1, a2),
            (AppType::Struct(s1), AppType::Struct(s2)) => self.unify_struct(s1, s2),
            (AppType::Enum(e1), AppType::Enum(e2)) => self.unify_enum(e1, e2),
            (AppType::Bits(b1), AppType::Bits(b2)) => self.unify_bits(b1, b2),
            (AppType::Signal(s1), AppType::Signal(s2)) => self.unify_signal(s1, s2),
            _ => Err(
                self.raise_type_error(UnifyError::CannotUnifyApplicativeTypes {
                    x: app1.clone(),
                    y: app2.clone(),
                }),
            ),
        }
    }
    fn unify_app_unclocked(&mut self, x: TypeId, y: TypeId) -> Result<(), RHDLError> {
        let TypeKind::App(app) = (*x.kind).clone() else {
            return Err(self.raise_type_error(UnifyError::ExpectedAppType {
                x: x.kind.as_ref().clone(),
            }));
        };
        match &app {
            AppType::Tuple(t1) => self.unify_unclocked_tuple(t1, y),
            AppType::Array(a1) => self.unify_unclocked_array(a1, y),
            AppType::Struct(s1) => self.unify_unclocked_struct(s1, y),
            AppType::Enum(e1) => self.unify_unclocked_enum(e1, y),
            _ => Err(self.raise_type_error(UnifyError::CannotUnifyUnclocked { x: app.clone() })),
        }
    }
    fn occurs(&self, v: TypeId, term: TypeId) -> bool {
        // Check for the case that term is a variable
        if self.is_var(term) {
            // Unifying with itself is not allowed
            if term.kind == v.kind {
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
        if let TypeKind::App(app_type) = term.kind.as_ref() {
            return app_type.sub_types().iter().any(|t| self.occurs(v, *t));
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use log::debug;

    use crate::ast::ast_impl::NodeId;

    use super::*;

    #[test]
    fn test_case_1() {
        let mut ctx = UnifyContext::default();
        let id = SourceLocation {
            func: 0.into(),
            node: NodeId::new(0),
        };
        let x = ctx.ty_var(id);
        let y = ctx.ty_var(id);
        let z = ctx.ty_var(id);
        let t = ctx.ty_tuple(id, vec![x, y, z]);
        let a = ctx.ty_integer(id);
        let b = ctx.ty_usize(id);
        let c = ctx.from_kind(id, Kind::Bits(128));
        let u = ctx.ty_tuple(id, vec![a, b, c]);
        assert!(ctx.unify(t, u).is_ok());
        let x = ctx.apply(x);
        let y = ctx.apply(y);
        let z = ctx.apply(z);
        assert_eq!(x, a);
        assert_eq!(y, b);
        assert_eq!(z, c);
    }

    #[test]
    fn test_case_2() {
        let mut ctx = UnifyContext::default();
        let id = SourceLocation {
            func: 0.into(),
            node: NodeId::new(0),
        };
        let n = ctx.ty_const_len(id, 12);
        let x = ctx.ty_bits(id, n);
        let m = ctx.ty_var(id);
        let y = ctx.ty_bits(id, m);
        let z = ctx.ty_var(id);
        assert!(ctx.unify(x, y).is_ok());
        assert!(ctx.unify(x, z).is_ok());
        debug!("{}", ctx);
        let m = ctx.into_kind(z).unwrap();
        println!("{m:?}");
    }
}
