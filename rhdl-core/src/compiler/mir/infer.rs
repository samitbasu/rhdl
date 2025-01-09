use log::{debug, trace};
use rhdl_bits::alias::{b128, s128};
use std::collections::BTreeMap;

use crate::{
    ast::{
        ast_impl::{ExprLit, WrapOp},
        source::source_location::SourceLocation,
    },
    compiler::mir::{
        error::{RHDLSyntaxError, RHDLTypeCheckError, Syntax},
        ty::SignFlag,
    },
    error::RHDLError,
    rhif::{
        object::LocatedOpCode,
        spec::{AluBinary, AluUnary, CaseArgument, OpCode, Slot},
        Object,
    },
    types::path::{sub_kind, Path, PathElement},
    Digital, Kind, TypedBits,
};

use super::{
    error::{RHDLCompileError, RHDLTypeError, TypeCheck, ICE},
    mir_impl::{Mir, TypeEquivalence},
    ty::{TypeId, UnifyContext},
};
#[derive(Debug, Clone)]
pub struct TypeUnaryOp {
    op: AluUnary,
    lhs: TypeId,
    arg1: TypeId,
}

#[derive(Debug, Clone)]
pub struct TypeBinOp {
    op: AluBinary,
    lhs: TypeId,
    arg1: TypeId,
    arg2: TypeId,
}

#[derive(Debug, Clone)]
pub struct TypeIndex {
    lhs: TypeId,
    arg: TypeId,
    path: Path,
}

#[derive(Debug, Clone)]
pub struct TypeSelect {
    true_value: TypeId,
    false_value: TypeId,
    lhs: TypeId,
}

#[derive(Debug, Clone)]
pub struct TypeOperation {
    loc: SourceLocation,
    kind: TypeOperationKind,
}

#[derive(Debug, Clone)]
pub enum TypeOperationKind {
    UnaryOp(TypeUnaryOp),
    BinOp(TypeBinOp),
    Index(TypeIndex),
    Select(TypeSelect),
}

pub struct MirTypeInference<'a> {
    ctx: UnifyContext,
    slot_map: BTreeMap<Slot, TypeId>,
    mir: &'a Mir,
    type_ops: Vec<TypeOperation>,
}

type Result<T> = std::result::Result<T, RHDLError>;

/*
 Some additional concerns:
 1. If you can strip clocking information from a signal, then you
    can form types that have no Kind representation.  For example,
    the array is assumed to be homogenous.  But we can do something like:
    let x = x.val(); // x <- Red
    let y = y.val(); // y <- Green
    let z = [x, y]; // z <- [Red, Green] ??
    Rust will not complain, as this is completely allowed.  But when we
    try to reconstruct the timing


*/

impl<'a> MirTypeInference<'a> {
    fn new(mir: &'a Mir) -> Self {
        Self {
            mir,
            ctx: UnifyContext::default(),
            slot_map: BTreeMap::default(),
            type_ops: Vec::new(),
        }
    }
    fn raise_ice(&self, cause: ICE, loc: SourceLocation) -> Box<RHDLCompileError> {
        let source_span = self.mir.symbols.source_set.span(loc);
        Box::new(RHDLCompileError {
            cause,
            src: self.mir.symbols.source(),
            err_span: source_span.into(),
        })
    }
    fn raise_type_error(&self, cause: TypeCheck, loc: SourceLocation) -> Box<RHDLTypeError> {
        let source_span = self.mir.symbols.source_set.span(loc);
        Box::new(RHDLTypeError {
            cause,
            src: self.mir.symbols.source(),
            err_span: source_span.into(),
        })
    }
    fn cast_literal_to_inferred_type(&mut self, t: ExprLit, ty: TypeId) -> Result<TypedBits> {
        let kind = self.ctx.into_kind(ty)?;
        Ok(match t {
            ExprLit::TypedBits(tb) => {
                if tb.value.kind != kind {
                    return Err(self
                        .raise_type_error(
                            TypeCheck::InferredLiteralTypeMismatch {
                                typ: tb.value.kind,
                                kind,
                            },
                            ty.loc,
                        )
                        .into());
                }
                tb.value
            }
            ExprLit::Int(x) => {
                if kind.is_unsigned() {
                    let x_as_u128 = if let Some(x) = x.strip_prefix("0b") {
                        u128::from_str_radix(x, 2)?
                    } else if let Some(x) = x.strip_prefix("0o") {
                        u128::from_str_radix(x, 8)?
                    } else if let Some(x) = x.strip_prefix("0x") {
                        u128::from_str_radix(x, 16)?
                    } else {
                        x.parse::<u128>()?
                    };
                    b128(x_as_u128).typed_bits().unsigned_cast(kind.bits())?
                } else {
                    let x_as_i128 = if let Some(x) = x.strip_prefix("0b") {
                        i128::from_str_radix(x, 2)?
                    } else if let Some(x) = x.strip_prefix("0o") {
                        i128::from_str_radix(x, 8)?
                    } else if let Some(x) = x.strip_prefix("0x") {
                        i128::from_str_radix(x, 16)?
                    } else {
                        x.parse::<i128>()?
                    };
                    s128(x_as_i128).typed_bits().signed_cast(kind.bits())?
                }
            }
            ExprLit::Bool(b) => b.typed_bits(),
        })
    }
    fn unify(&mut self, loc: SourceLocation, lhs: TypeId, rhs: TypeId) -> Result<()> {
        trace!("Unifying {} and {}", self.ctx.desc(lhs), self.ctx.desc(rhs));
        if self.ctx.unify(lhs, rhs).is_err() {
            let lhs_span = self.mir.symbols.source_set.span(lhs.loc);
            let rhs_span = self.mir.symbols.source_set.span(rhs.loc);
            let lhs = self.ctx.apply(lhs);
            let rhs = self.ctx.apply(rhs);
            let lhs_desc = self.ctx.desc(lhs);
            let rhs_desc = self.ctx.desc(rhs);
            let cause_span = self.mir.symbols.source_set.span(loc);
            let cause_description = "Because of this expression".to_owned();
            return Err(Box::new(RHDLTypeCheckError {
                src: self.mir.symbols.source(),
                lhs_type: lhs_desc,
                lhs_span: lhs_span.into(),
                rhs_type: rhs_desc,
                rhs_span: rhs_span.into(),
                cause_description,
                cause_span: cause_span.into(),
            })
            .into());
        }
        Ok(())
    }
    fn import_literals(&mut self) {
        for (slot, lit) in &self.mir.literals {
            let id = self.mir.symbols.slot_map[slot];
            let ty = match lit {
                ExprLit::TypedBits(tb) => self.ctx.from_kind(id, &tb.value.kind),
                ExprLit::Int(_) => self.ctx.ty_integer(id),
                ExprLit::Bool(_) => self.ctx.ty_bool(id),
            };
            self.slot_map.insert(*slot, ty);
        }
    }
    fn import_signature(&mut self) -> Result<()> {
        for slot in &self.mir.arguments {
            let id = self.mir.symbols.slot_map[slot];
            let kind = &self.mir.ty[slot];
            let ty = self.ctx.from_kind(id, kind);
            self.slot_map.insert(*slot, ty);
        }
        let id = self.mir.symbols.slot_map[&self.mir.return_slot];
        let return_kind = &self.mir.ty[&self.mir.return_slot];
        let return_ty = self.ctx.from_kind(id, return_kind);
        self.slot_map.insert(self.mir.return_slot, return_ty);
        Ok(())
    }
    fn import_type_declarations(&mut self) -> Result<()> {
        for (slot, ty) in &self.mir.ty {
            let id = self.mir.symbols.slot_map[slot];
            let ty = self.ctx.from_kind(id, ty);
            self.slot_map.insert(*slot, ty);
        }
        Ok(())
    }
    fn import_type_equality(&mut self) -> Result<()> {
        for TypeEquivalence { loc: id, lhs, rhs } in &self.mir.ty_equate {
            let lhs_ty = self.slot_ty(*lhs);
            let rhs_ty = self.slot_ty(*rhs);
            self.unify(*id, lhs_ty, rhs_ty)?;
        }
        Ok(())
    }
    fn slot_ty(&mut self, slot: Slot) -> TypeId {
        let id = self.mir.symbols.slot_map[&slot];
        if matches!(slot, Slot::Empty) {
            return self.ctx.ty_empty(id);
        }
        if let Some(ty) = self.slot_map.get(&slot) {
            *ty
        } else {
            let var = self.ctx.ty_var(id);
            self.slot_map.insert(slot, var);
            var
        }
    }
    fn slot_tys(&mut self, slots: &[Slot]) -> Vec<TypeId> {
        slots.iter().map(|slot| self.slot_ty(*slot)).collect()
    }
    fn all_slots_resolved(&mut self) -> bool {
        self.unresolved_slot_typeid().is_none()
    }
    fn unresolved_slot_typeid(&mut self) -> Option<TypeId> {
        for ty in self.slot_map.values() {
            if self.ctx.into_kind(*ty).is_err() {
                return Some(*ty);
            }
        }
        None
    }
    fn try_unary(&mut self, loc: SourceLocation, op: &TypeUnaryOp) -> Result<()> {
        let a1 = self.ctx.apply(op.arg1);
        match op.op {
            AluUnary::All | AluUnary::Any | AluUnary::Xor => {
                let bool_ty = self.ctx.ty_bool(loc);
                if self.ctx.is_signal(a1) {
                    let clock_ty = self.ctx.ty_var(loc);
                    let bool_sig = self.ctx.ty_signal(loc, bool_ty, clock_ty);
                    self.unify(loc, op.lhs, bool_sig)?;
                    if let Some(a1_clock) = self.ctx.project_signal_clock(a1) {
                        self.unify(loc, clock_ty, a1_clock)?;
                    }
                } else {
                    self.unify(loc, op.lhs, bool_ty)?;
                }
            }
            AluUnary::Pad => {
                let Some(a1_len) = self.ctx.project_bit_length(a1) else {
                    return Ok(());
                };
                let Ok(a1_len) = self.ctx.cast_ty_as_bit_length(a1_len) else {
                    return Ok(());
                };
                let Some(a1_sign) = self.ctx.project_sign_flag(a1) else {
                    return Ok(());
                };
                let len = self.ctx.ty_const_len(loc, a1_len + 1);
                let lhs_ty = self.ctx.ty_with_sign_and_len(loc, a1_sign, len);
                self.unify(loc, op.lhs, lhs_ty)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn try_binop(&mut self, loc: SourceLocation, op: &TypeBinOp) -> Result<()> {
        match &op.op {
            AluBinary::Add
            | AluBinary::BitAnd
            | AluBinary::BitOr
            | AluBinary::BitXor
            | AluBinary::Mul
            | AluBinary::Sub => {
                self.enforce_data_types_binary(loc, op.lhs, op.arg1, op.arg2)?;
            }
            AluBinary::XAdd => {
                self.try_xadd(loc, op.lhs, op.arg1, op.arg2)?;
            }
            AluBinary::XSub => {
                self.try_xsub(loc, op.lhs, op.arg1, op.arg2)?;
            }
            AluBinary::Eq
            | AluBinary::Lt
            | AluBinary::Le
            | AluBinary::Ne
            | AluBinary::Ge
            | AluBinary::Gt => {
                // LHS of a comparison is always a boolean
                let lhs_var = self.ctx.ty_bool(loc);
                self.unify(loc, op.lhs, lhs_var)?;
                let a1_is_signal = self.ctx.is_signal(op.arg1);
                let a2_is_signal = self.ctx.is_signal(op.arg2);

                if !a1_is_signal && !a2_is_signal {
                    self.ctx.unify(op.arg1, op.arg2)?;
                } else if let (Some(arg1_data), Some(arg2_data)) = (
                    self.ctx.project_signal_value(op.arg1),
                    self.ctx.project_signal_value(op.arg2),
                ) {
                    self.unify(loc, arg1_data, arg2_data)?;
                }
            }
            AluBinary::Shl | AluBinary::Shr => {
                self.unify(loc, op.lhs, op.arg1)?;
                if let Some(flag) = self.ctx.project_sign_flag(op.arg2) {
                    let unsigned_flag = self.ctx.ty_sign_flag(loc, SignFlag::Unsigned);
                    self.unify(loc, flag, unsigned_flag)?;
                }

                /*
                TODO - do I need this?
                if let Some(arg2) = self.ctx.project_signal_value(a2) {
                    debug!("Project signal value flag for {}", self.ctx.desc(a2));
                    if let Some(flag) = self.ctx.project_sign_flag(arg2) {
                        debug!("Project sign flag for {}", self.ctx.desc(a2));
                        let unsigned_flag = self.ctx.ty_sign_flag(id, SignFlag::Unsigned);
                        self.unify(id, flag, unsigned_flag)?;
                    }
                }
                if let (Some(lhs_data), Some(arg1_data)) = (
                    self.ctx.project_signal_value(op.lhs),
                    self.ctx.project_signal_value(op.arg1),
                ) {
                    self.unify(id, lhs_data, arg1_data)?;
                } else {
                    self.unify(id, op.lhs, op.arg1)?;
                }
                */
            }
        }
        Ok(())
    }

    fn ty_path_project(&mut self, arg: TypeId, path: &Path, loc: SourceLocation) -> Result<TypeId> {
        let mut arg = self.ctx.apply(arg);
        for element in path.elements.iter() {
            match element {
                PathElement::Index(ndx) => {
                    arg = self.ctx.ty_index(arg, *ndx)?;
                }
                PathElement::Field(member) => {
                    arg = self.ctx.ty_field(arg, member)?;
                }
                PathElement::EnumDiscriminant => {
                    arg = self.ctx.ty_enum_discriminant(arg);
                }
                PathElement::TupleIndex(ndx) => {
                    arg = self.ctx.ty_index(arg, *ndx)?;
                }
                PathElement::EnumPayload(member) => {
                    arg = self.ctx.ty_variant(arg, member)?;
                }
                PathElement::DynamicIndex(slot) => {
                    let index = self.slot_ty(*slot);
                    let usize_ty = self.ctx.ty_usize(loc);
                    if slot.is_literal() {
                        self.unify(loc, index, usize_ty)?;
                    } else {
                        let reg_ty = self.ctx.apply(index);
                        if self.ctx.is_generic_integer(reg_ty) {
                            // For more clearly defined types, it is someone else's problem
                            // to ensure that the index is properly typed.
                            self.unify(loc, reg_ty, usize_ty)?;
                        }
                    }
                    arg = self.ctx.ty_index(arg, 0)?;
                }
                PathElement::EnumPayloadByValue(value) => {
                    arg = self.ctx.ty_variant_by_value(arg, *value)?;
                }
                PathElement::SignalValue => {
                    arg = self
                        .ctx
                        .project_signal_value(arg)
                        .ok_or(self.raise_type_error(TypeCheck::ExpectedSignalValue, loc))?;
                }
            }
        }
        Ok(arg)
    }

    fn try_index(&mut self, loc: SourceLocation, op: &TypeIndex) -> Result<()> {
        trace!(
            "Try to apply index to {} with path {:?}",
            self.ctx.desc(op.arg),
            op.path
        );
        match self.ty_path_project(op.arg, &op.path, loc) {
            Ok(ty) => self.unify(loc, op.lhs, ty),
            Err(_) => Err(self
                .raise_type_error(TypeCheck::PathMismatchInTypeInference, loc)
                .into()),
        }
    }
    fn try_xsub(
        &mut self,
        loc: SourceLocation,
        lhs: TypeId,
        arg1: TypeId,
        arg2: TypeId,
    ) -> Result<()> {
        let size_computation = |a: usize, b: usize| a.max(b) + 1;
        let lhs_len = self.ctx.ty_var(loc);
        let Some(a1_sign) = self.ctx.project_sign_flag(arg1) else {
            return Ok(());
        };
        let Some(a2_sign) = self.ctx.project_sign_flag(arg2) else {
            return Ok(());
        };
        // Both arguments must have the same sign
        self.unify(loc, a1_sign, a2_sign)?;
        // Get the bit length of the operands
        let Some(a1_len) = self.ctx.project_bit_length(arg1) else {
            return Ok(());
        };
        let Some(a2_len) = self.ctx.project_bit_length(arg2) else {
            return Ok(());
        };
        let Ok(a1_len) = self.ctx.cast_ty_as_bit_length(a1_len) else {
            return Ok(());
        };
        let Ok(a2_len) = self.ctx.cast_ty_as_bit_length(a2_len) else {
            return Ok(());
        };
        let lhs_computed_len = self.ctx.ty_const_len(loc, size_computation(a1_len, a2_len));
        self.unify(loc, lhs_len, lhs_computed_len)?;
        let lhs_sign_flag = self.ctx.ty_sign_flag(loc, SignFlag::Signed);
        let lhs_data = self
            .ctx
            .ty_with_sign_and_len(loc, lhs_sign_flag, lhs_computed_len);
        return self.unify(loc, lhs, lhs_data);
    }

    fn try_xadd(
        &mut self,
        loc: SourceLocation,
        lhs: TypeId,
        arg1: TypeId,
        arg2: TypeId,
    ) -> Result<()> {
        let size_computation = |a: usize, b: usize| a.max(b) + 1;
        let lhs_sign_flag = self.ctx.ty_var(loc);
        let lhs_len = self.ctx.ty_var(loc);
        let Some(a1_sign) = self.ctx.project_sign_flag(arg1) else {
            return Ok(());
        };
        let Some(a2_sign) = self.ctx.project_sign_flag(arg2) else {
            return Ok(());
        };
        self.unify(loc, a1_sign, lhs_sign_flag)?;
        self.unify(loc, a2_sign, lhs_sign_flag)?;
        // Get the bit length of the operands
        let Some(a1_len) = self.ctx.project_bit_length(arg1) else {
            return Ok(());
        };
        let Some(a2_len) = self.ctx.project_bit_length(arg2) else {
            return Ok(());
        };
        let Ok(a1_len) = self.ctx.cast_ty_as_bit_length(a1_len) else {
            return Ok(());
        };
        let Ok(a2_len) = self.ctx.cast_ty_as_bit_length(a2_len) else {
            return Ok(());
        };
        let lhs_computed_len = self.ctx.ty_const_len(loc, size_computation(a1_len, a2_len));
        self.unify(loc, lhs_len, lhs_computed_len)?;
        let lhs_data = self
            .ctx
            .ty_with_sign_and_len(loc, lhs_sign_flag, lhs_computed_len);
        return self.unify(loc, lhs, lhs_data);
    }
    // Given Y <- A op B, ensure that the data types of
    // Y, A, an B are all compatible.
    // This means that either A and B are not signals (constants),
    // both are signals with the same clock domain, or
    // one of them is a signal and the other is a constant.
    // In all cases, the data type of Y must be the same as the data type
    // of A and B.
    fn enforce_data_types_binary(
        &mut self,
        loc: SourceLocation,
        lhs: TypeId,
        arg1: TypeId,
        arg2: TypeId,
    ) -> Result<()> {
        let a1_is_signal = self.ctx.is_signal(arg1);
        let a2_is_signal = self.ctx.is_signal(arg2);
        let a_data = self.ctx.project_signal_data(arg1);
        let b_data = self.ctx.project_signal_data(arg2);
        self.unify(loc, a_data, b_data)?;
        if a1_is_signal {
            self.unify(loc, lhs, arg1)?;
        }
        if a2_is_signal {
            self.unify(loc, lhs, arg2)?;
        }
        if !a1_is_signal && !a2_is_signal {
            self.unify(loc, lhs, arg1)?;
            self.unify(loc, lhs, arg2)?;
        }
        if let (Some(arg1_data), Some(arg2_data)) = (
            self.ctx.project_signal_value(arg1),
            self.ctx.project_signal_value(arg2),
        ) {
            self.unify(loc, arg1_data, arg2_data)?;
        }
        if let (Some(lhs_data), Some(arg1_data)) = (
            self.ctx.project_signal_value(lhs),
            self.ctx.project_signal_value(arg1),
        ) {
            self.unify(loc, lhs_data, arg1_data)?;
        }
        if let (Some(lhs_data), Some(arg2_data)) = (
            self.ctx.project_signal_value(lhs),
            self.ctx.project_signal_value(arg2),
        ) {
            self.unify(loc, lhs_data, arg2_data)?;
        }
        Ok(())
    }
    fn try_select(&mut self, loc: SourceLocation, op: &TypeSelect) -> Result<()> {
        self.enforce_data_types_binary(loc, op.lhs, op.true_value, op.false_value)?;
        Ok(())
    }
    fn try_type_op(&mut self, op: &TypeOperation) -> Result<()> {
        let id = op.loc;
        match &op.kind {
            TypeOperationKind::BinOp(binop) => self.try_binop(id, binop),
            TypeOperationKind::Index(index) => self.try_index(id, index),
            TypeOperationKind::UnaryOp(unary) => self.try_unary(id, unary),
            TypeOperationKind::Select(select) => self.try_select(id, select),
        }
    }
    fn try_type_ops(&mut self, iteration_count: usize, ops: &[TypeOperation]) -> Result<()> {
        for loop_count in 0..iteration_count {
            debug!("Iteration {}", loop_count);
            let mod_state = self.ctx.modification_state();
            for op in ops {
                trace!("Type op {:?}", op);
                self.try_type_op(op)?;
            }
            if self.ctx.modification_state() == mod_state {
                break;
            }
            if self.all_slots_resolved() {
                break;
            }
        }
        Ok(())
    }
    fn process_ops(&mut self) -> Result<()> {
        for op in &self.mir.ops {
            trace!("Processing op {:?}", op.op);
            let loc = op.loc;
            match &op.op {
                OpCode::Array(array) => {
                    let lhs = self.slot_ty(array.lhs);
                    let rhs = self.slot_tys(&array.elements);
                    let array_base = self.ctx.ty_var(loc);
                    let array_len = self.ctx.ty_const_len(loc, rhs.len());
                    let lhs_ty = self.ctx.ty_array(loc, array_base, array_len);
                    self.unify(loc, lhs, lhs_ty)?;
                    for element in rhs {
                        self.unify(loc, element, array_base)?;
                    }
                }
                OpCode::Assign(assign) => {
                    let lhs = self.slot_ty(assign.lhs);
                    let rhs = self.slot_ty(assign.rhs);
                    self.unify(loc, lhs, rhs)?;
                }
                OpCode::AsBits(as_bits) => {
                    let arg = self.slot_ty(as_bits.arg);
                    let lhs = self.slot_ty(as_bits.lhs);
                    let len = if let Some(len) = as_bits.len {
                        self.ctx.ty_const_len(loc, len)
                    } else {
                        self.ctx.ty_var(loc)
                    };
                    let lhs_ty = self.ctx.ty_bits(loc, len);
                    self.unify(loc, lhs, lhs_ty)?;
                    let len_128 = self.ctx.ty_const_len(loc, 128);
                    let arg_ty = self.ctx.ty_bits(loc, len_128);
                    self.unify(loc, arg, arg_ty)?;
                }
                OpCode::AsSigned(as_signed) => {
                    let arg = self.slot_ty(as_signed.arg);
                    let lhs = self.slot_ty(as_signed.lhs);
                    let len = if let Some(len) = as_signed.len {
                        self.ctx.ty_const_len(loc, len)
                    } else {
                        self.ctx.ty_var(loc)
                    };
                    let lhs_ty = self.ctx.ty_signed(loc, len);
                    self.unify(loc, lhs, lhs_ty)?;
                    let len_128 = self.ctx.ty_const_len(loc, 128);
                    let arg_ty = self.ctx.ty_signed(loc, len_128);
                    self.unify(loc, arg, arg_ty)?;
                }
                OpCode::Binary(binary) => {
                    let lhs = self.slot_ty(binary.lhs);
                    let arg1 = self.slot_ty(binary.arg1);
                    let arg2 = self.slot_ty(binary.arg2);
                    self.type_ops.push(TypeOperation {
                        loc,
                        kind: TypeOperationKind::BinOp(TypeBinOp {
                            op: binary.op,
                            lhs,
                            arg1,
                            arg2,
                        }),
                    });
                }
                OpCode::Case(case) => {
                    let lhs = self.slot_ty(case.lhs);
                    let disc = self.slot_ty(case.discriminant);
                    for (test, value) in case.table.iter() {
                        match test {
                            CaseArgument::Slot(slot) => {
                                let ty = self.slot_ty(*slot);
                                self.unify(loc, disc, ty)?;
                            }
                            CaseArgument::Wild => {}
                        }
                        let val_ty = self.slot_ty(*value);
                        self.unify(loc, lhs, val_ty)?;
                    }
                }
                OpCode::Enum(enumerate) => {
                    let lhs = self.slot_ty(enumerate.lhs);
                    let Kind::Enum(enum_k) = &enumerate.template.kind else {
                        return Err(self
                            .raise_ice(
                                ICE::ExpectedEnumTemplate {
                                    kind: enumerate.template.kind,
                                },
                                op.loc,
                            )
                            .into());
                    };
                    let lhs_ty = self.ctx.ty_enum(loc, enum_k);
                    self.unify(loc, lhs, lhs_ty)?;
                    let discriminant = enumerate.template.discriminant()?.as_i64()?;
                    for field in &enumerate.fields {
                        let path = match &field.member {
                            crate::rhif::spec::Member::Named(name) => {
                                Path::default().payload_by_value(discriminant).field(name)
                            }
                            crate::rhif::spec::Member::Unnamed(ndx) => Path::default()
                                .payload_by_value(discriminant)
                                .tuple_index(*ndx as usize),
                        };
                        let field_kind = sub_kind(enumerate.template.kind, &path)?;
                        let field_ty = self.ctx.from_kind(loc, &field_kind);
                        let field_slot = self.slot_ty(field.value);
                        self.unify(loc, field_ty, field_slot)?;
                    }
                }
                OpCode::Exec(exec) => {
                    let external_fn = &self.mir.stash[&exec.id];
                    let sub_args = external_fn.arguments.iter().map(|x| &external_fn.kind[x]);
                    for (arg_kind, arg_slot) in sub_args.zip(exec.args.iter()) {
                        let arg_ty = self.slot_ty(*arg_slot);
                        let arg_kind = self.ctx.from_kind(loc, arg_kind);
                        self.unify(loc, arg_ty, arg_kind)?;
                    }
                    let ret_ty = self.slot_ty(exec.lhs);
                    let ret_kind = self
                        .ctx
                        .from_kind(loc, &external_fn.kind(external_fn.return_slot));
                    self.unify(loc, ret_ty, ret_kind)?;
                }
                OpCode::Index(index) => {
                    let arg = self.slot_ty(index.arg);
                    let lhs = self.slot_ty(index.lhs);
                    let path = index.path.clone();
                    self.type_ops.push(TypeOperation {
                        loc,
                        kind: TypeOperationKind::Index(TypeIndex { lhs, arg, path }),
                    });
                }
                OpCode::Repeat(repeat) => {
                    let lhs = self.slot_ty(repeat.lhs);
                    let value = self.slot_ty(repeat.value);
                    let len = self.ctx.ty_const_len(loc, repeat.len as usize);
                    let lhs_ty = self.ctx.ty_array(loc, value, len);
                    self.unify(loc, lhs, lhs_ty)?;
                }
                OpCode::Resize(cast) => {
                    let arg = self.slot_ty(cast.arg);
                    let lhs = self.slot_ty(cast.lhs);
                    let len = if let Some(len) = cast.len {
                        self.ctx.ty_const_len(loc, len)
                    } else {
                        self.ctx.ty_var(loc)
                    };
                    let sign = self.ctx.ty_var(loc);
                    let lhs_ty = self.ctx.ty_with_sign_and_len(loc, sign, len);
                    self.unify(loc, lhs, lhs_ty)?;
                    let arg_len = self.ctx.ty_var(loc);
                    let arg_ty = self.ctx.ty_with_sign_and_len(loc, sign, arg_len);
                    self.unify(loc, arg, arg_ty)?;
                }
                OpCode::Retime(retime) => {
                    let lhs = self.slot_ty(retime.lhs);
                    let arg = self.slot_ty(retime.arg);
                    let color = retime.color;
                    let sig_ty = self.ctx.ty_var(loc);
                    let sig_clock_lhs = self.ctx.ty_var(loc);
                    let sig = self.ctx.ty_signal(loc, sig_ty, sig_clock_lhs);
                    self.unify(loc, lhs, sig)?;
                    self.unify(loc, arg, sig_ty)?;
                    if let Some(color) = color {
                        let clk = self.ctx.ty_clock(loc, color);
                        self.unify(loc, sig_clock_lhs, clk)?;
                    }
                }
                OpCode::Select(select) => {
                    let cond = self.slot_ty(select.cond);
                    let cond_ty = self.ctx.ty_bool(loc);
                    self.unify(loc, cond, cond_ty)?;
                    let lhs = self.slot_ty(select.lhs);
                    let true_value = self.slot_ty(select.true_value);
                    let false_value = self.slot_ty(select.false_value);
                    trace!(
                        "Queueing select operation lhs = {}, true = {}, false = {}",
                        self.ctx.desc(lhs),
                        self.ctx.desc(true_value),
                        self.ctx.desc(false_value)
                    );
                    self.type_ops.push(TypeOperation {
                        loc: op.loc,
                        kind: TypeOperationKind::Select(TypeSelect {
                            lhs,
                            true_value,
                            false_value,
                        }),
                    });
                }
                OpCode::Splice(splice) => {
                    let lhs = self.slot_ty(splice.lhs);
                    let orig = self.slot_ty(splice.orig);
                    let subst = self.slot_ty(splice.subst);
                    let path = &splice.path;
                    self.unify(loc, lhs, orig)?;
                    // Reflect the constraint that
                    // ty(subst) = ty(lhs[path])
                    self.type_ops.push(TypeOperation {
                        loc,
                        kind: TypeOperationKind::Index(TypeIndex {
                            lhs: subst,
                            arg: lhs,
                            path: path.clone(),
                        }),
                    });
                }
                OpCode::Struct(structure) => {
                    let lhs = self.slot_ty(structure.lhs);
                    let Kind::Struct(strukt) = &structure.template.kind else {
                        return Err(self
                            .raise_ice(
                                ICE::ExpectedStructTemplate {
                                    kind: structure.template.kind,
                                },
                                op.loc,
                            )
                            .into());
                    };
                    let lhs_ty = self.ctx.ty_struct(loc, strukt);
                    self.unify(loc, lhs, lhs_ty)?;
                    for field in &structure.fields {
                        let field_kind = strukt.get_field_kind(&field.member)?;
                        let field_ty = self.ctx.from_kind(loc, &field_kind);
                        let field_slot = self.slot_ty(field.value);
                        self.unify(loc, field_ty, field_slot)?;
                    }
                    if let Some(rest) = structure.rest {
                        let rest_ty = self.slot_ty(rest);
                        self.unify(loc, lhs_ty, rest_ty)?;
                    }
                    self.unify(loc, lhs, lhs_ty)?;
                }
                OpCode::Tuple(tuple) => {
                    let lhs = self.slot_ty(tuple.lhs);
                    let tys = tuple
                        .fields
                        .iter()
                        .map(|slot| self.slot_ty(*slot))
                        .collect();
                    let lhs_ty = self.ctx.ty_tuple(loc, tys);
                    self.unify(loc, lhs, lhs_ty)?;
                }
                OpCode::Unary(unary) => {
                    let lhs = self.slot_ty(unary.lhs);
                    let arg1 = self.slot_ty(unary.arg1);
                    match unary.op {
                        AluUnary::Not => {
                            self.unify(loc, lhs, arg1)?;
                        }
                        AluUnary::Neg => {
                            let len = self.ctx.ty_var(loc);
                            let signed_ty = self.ctx.ty_signed(loc, len);
                            if self.unify(loc, lhs, signed_ty).is_err()
                                || self.unify(loc, arg1, signed_ty).is_err()
                            {
                                let source_span = self.mir.symbols.source_set.span(loc);
                                return Err(Box::new(RHDLSyntaxError {
                                    src: self.mir.symbols.source(),
                                    cause: Syntax::RollYourOwnUnary { op: AluUnary::Neg },
                                    err_span: source_span.into(),
                                })
                                .into());
                            }
                        }
                        AluUnary::All | AluUnary::Any | AluUnary::Xor | AluUnary::Pad => {
                            self.type_ops.push(TypeOperation {
                                loc: op.loc,
                                kind: TypeOperationKind::UnaryOp(TypeUnaryOp {
                                    op: unary.op,
                                    lhs,
                                    arg1,
                                }),
                            });
                        }
                        AluUnary::Unsigned => {
                            let len = self.ctx.ty_var(loc);
                            let signed_ty = self.ctx.ty_signed(loc, len);
                            let unsigned_ty = self.ctx.ty_bits(loc, len);
                            if self.unify(loc, lhs, unsigned_ty).is_err()
                                || self.unify(loc, arg1, signed_ty).is_err()
                            {
                                let source_span = self.mir.symbols.source_set.span(loc);
                                return Err(Box::new(RHDLSyntaxError {
                                    src: self.mir.symbols.source(),
                                    cause: Syntax::RollYourOwnUnary {
                                        op: AluUnary::Unsigned,
                                    },
                                    err_span: source_span.into(),
                                })
                                .into());
                            }
                        }
                        AluUnary::Signed => {
                            let len = self.ctx.ty_var(loc);
                            let signed_ty = self.ctx.ty_signed(loc, len);
                            let unsigned_ty = self.ctx.ty_bits(loc, len);
                            if self.unify(loc, lhs, signed_ty).is_err()
                                || self.unify(loc, arg1, unsigned_ty).is_err()
                            {
                                let source_span = self.mir.symbols.source_set.span(loc);
                                return Err(Box::new(RHDLSyntaxError {
                                    src: self.mir.symbols.source(),
                                    cause: Syntax::RollYourOwnUnary {
                                        op: AluUnary::Signed,
                                    },
                                    err_span: source_span.into(),
                                })
                                .into());
                            }
                        }
                        AluUnary::Val => {
                            let sig_ty = self.ctx.ty_var(loc);
                            let sig_clock = self.ctx.ty_var(loc);
                            let sig = self.ctx.ty_signal(loc, sig_ty, sig_clock);
                            if self.unify(loc, lhs, sig_ty).is_err()
                                || self.unify(loc, arg1, sig).is_err()
                            {
                                let source_span = self.mir.symbols.source_set.span(loc);
                                return Err(Box::new(RHDLSyntaxError {
                                    src: self.mir.symbols.source(),
                                    cause: Syntax::RollYourOwnUnary { op: AluUnary::Val },
                                    err_span: source_span.into(),
                                })
                                .into());
                            }
                        }
                    }
                }
                OpCode::Wrap(wrap) => {
                    let arg_ty = self.slot_ty(wrap.arg);
                    let lhs_ty = match wrap.op {
                        WrapOp::Ok => {
                            let err_ty = self.ctx.ty_var(loc);
                            self.ctx.ty_result(loc, arg_ty, err_ty)
                        }
                        WrapOp::Err => {
                            let ok_ty = self.ctx.ty_var(loc);
                            self.ctx.ty_result(loc, ok_ty, arg_ty)
                        }
                        WrapOp::Some => self.ctx.ty_option(loc, arg_ty),
                        WrapOp::None => {
                            let some_ty = self.ctx.ty_var(loc);
                            self.ctx.ty_option(loc, some_ty)
                        }
                    };
                    let lhs = self.slot_ty(wrap.lhs);
                    self.unify(loc, lhs, lhs_ty)?;
                    if let Some(kind) = &wrap.kind {
                        let kind = self.ctx.from_kind(loc, kind);
                        self.unify(loc, lhs_ty, kind)?;
                    }
                }
                OpCode::Noop | OpCode::Comment(_) => {}
            }
        }
        Ok(())
    }
}

pub fn infer(mir: Mir) -> Result<Object> {
    let mut infer = MirTypeInference::new(&mir);
    infer.import_literals();
    infer.import_signature()?;
    infer.import_type_equality()?;
    infer.import_type_declarations()?;
    debug!("=================================");
    debug!("Before inference");
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        debug!("Slot {:?} -> type {}", slot, ty);
    }
    for op in mir.ops.iter() {
        debug!("{:?}", op.op);
    }
    debug!("=================================");
    if let Err(e) = infer.process_ops() {
        debug!("Error: {}", e);
        for (slot, ty) in &infer.slot_map {
            let ty = infer.ctx.apply(*ty);
            let ty = infer.ctx.desc(ty);
            debug!("Slot {:?} -> type {}", slot, ty);
        }
        return Err(e);
    }
    infer.process_ops()?;
    let type_ops = infer.type_ops.clone();
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        debug!("Slot {:?} -> type {}", slot, ty);
    }
    // TODO - remove fixed iteration count
    infer.try_type_ops(5, &type_ops)?;
    debug!("Try to replace generic literals with ?128");
    // Try to replace generic literals with (b/s)128
    if !infer.all_slots_resolved() {
        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if infer.ctx.is_unsized_integer(ty) {
                let i128_len = infer.ctx.ty_const_len(ty.loc, 128);
                let m128_ty = infer.ctx.ty_maybe_signed(ty.loc, i128_len);
                debug!(
                    "Literal {:?} -> {} U {}",
                    lit,
                    infer.ctx.desc(ty),
                    infer.ctx.desc(m128_ty)
                );
                infer.unify(ty.loc, ty, m128_ty)?;
            }
        }
    }
    debug!("Recheck delayed inference rools");
    infer.try_type_ops(5, &type_ops)?;

    debug!("Try to replace generic literals with i128");
    // Try to replace any generic literals with i128s
    if !infer.all_slots_resolved() {
        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if let Some(ty_sign) = infer.ctx.project_sign_flag(ty) {
                if infer.ctx.is_unresolved(ty_sign) {
                    let sign_flag = infer.ctx.ty_sign_flag(ty.loc, SignFlag::Signed);
                    infer.unify(ty.loc, ty_sign, sign_flag)?;
                }
            }
        }
    }
    debug!("Recheck delayed inference rules");
    infer.try_type_ops(5, &type_ops)?;

    if let Some(ty) = infer.unresolved_slot_typeid() {
        debug!("=================================");
        debug!("Inference failed");
        for (slot, ty) in &infer.slot_map {
            let ty = infer.ctx.apply(*ty);
            let ty = infer.ctx.desc(ty);
            debug!("Slot {:?} -> type {}", slot, ty);
        }
        for op in mir.ops.iter() {
            debug!("{:?}", op.op);
        }

        debug!("=================================");

        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if infer.ctx.into_kind(ty).is_err() {
                debug!("Literal {:?} -> {}", lit, infer.ctx.desc(ty));
            }
        }
        return Err(infer
            .raise_type_error(TypeCheck::UnableToDetermineType, ty.loc)
            .into());
    }
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        debug!("Slot {:?} -> type {}", slot, ty);
    }
    let final_type_map: BTreeMap<Slot, TypeId> = infer
        .slot_map
        .clone()
        .into_iter()
        .map(|(slot, ty)| {
            let ty = infer.ctx.apply(ty);
            (slot, ty)
        })
        .collect();
    let kind = final_type_map
        .iter()
        .filter(|(slot, _)| slot.is_reg())
        .map(|(slot, ty)| {
            infer
                .ctx
                .into_kind(*ty)
                .map(|val| (*slot, val))
                .map(|(slot, val)| (slot.as_reg().unwrap(), val))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()
        .unwrap();
    for op in mir.ops.iter() {
        debug!("{:?}", op.op);
    }
    let literals = mir
        .literals
        .clone()
        .into_iter()
        .map(|(slot, lit)| {
            infer
                .cast_literal_to_inferred_type(lit, final_type_map[&slot])
                .map(|value| (slot, value))
                .map(|(slot, value)| (slot.as_literal().unwrap(), value))
        })
        .collect::<Result<_>>()?;
    let ops = mir
        .ops
        .iter()
        .cloned()
        .map(|lop| {
            let loc = lop.loc;
            if let OpCode::Wrap(mut wrap) = lop.op {
                let ty = final_type_map[&wrap.lhs];
                let lhs_kind = infer.ctx.into_kind(ty)?;
                wrap.kind = Some(lhs_kind);
                Ok(LocatedOpCode {
                    loc,
                    op: OpCode::Wrap(wrap),
                })
            } else {
                Ok(lop)
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(Object {
        symbols: mir.symbols,
        ops,
        literals,
        kind,
        arguments: mir
            .arguments
            .into_iter()
            .map(|x| x.as_reg().unwrap())
            .collect(),
        return_slot: mir.return_slot,
        externals: mir.stash,
        name: mir.name,
        fn_id: mir.fn_id,
    })
}
