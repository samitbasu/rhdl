use std::collections::BTreeMap;

use crate::{
    ast::ast_impl::ExprLit,
    path::{sub_kind, Path, PathElement},
    rhif::{
        spec::{AluBinary, AluUnary, CaseArgument, OpCode, Slot},
        Object,
    },
    Digital, Kind, TypedBits,
};
use anyhow::bail;
use anyhow::Result;

use super::{
    mir::Mir,
    mir_ty::{TypeId, UnifyContext},
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
    lhs: TypeId,
    selector: TypeId,
    true_value: TypeId,
    false_value: TypeId,
}

#[derive(Debug, Clone)]
pub enum TypeOperation {
    UnaryOp(TypeUnaryOp),
    BinOp(TypeBinOp),
    Index(TypeIndex),
    Select(TypeSelect),
}

// For type checking purposes, a dynamic path index
// cannot change the types of any of the parts of the
// expression.  So we replace all of the dynamic components
// with 0.
fn approximate_dynamic_paths(path: &Path) -> Path {
    path.iter()
        .map(|e| match e {
            PathElement::DynamicIndex(_) => PathElement::Index(0),
            _ => e.clone(),
        })
        .collect()
}

pub struct MirTypeInference<'a> {
    ctx: UnifyContext,
    slot_map: BTreeMap<Slot, TypeId>,
    mir: &'a Mir,
    type_ops: Vec<TypeOperation>,
}

impl<'a> MirTypeInference<'a> {
    fn new(mir: &'a Mir) -> Self {
        Self {
            mir,
            ctx: UnifyContext::default(),
            slot_map: BTreeMap::default(),
            type_ops: Vec::new(),
        }
    }
    fn cast_literal_to_inferred_type(&mut self, t: ExprLit, ty: TypeId) -> Result<TypedBits> {
        let kind = self.ctx.into_kind(ty)?;
        Ok(match t {
            ExprLit::TypedBits(tb) => {
                if tb.value.kind != kind {
                    bail!(
                        "Literal with explicit type {} is inferred as {} instead",
                        tb.value.kind,
                        kind
                    );
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
                    x_as_u128.typed_bits().unsigned_cast(kind.bits())?
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
                    x_as_i128.typed_bits().signed_cast(kind.bits())?
                }
            }
            ExprLit::Bool(b) => b.typed_bits(),
        })
    }
    fn unify(&mut self, lhs: TypeId, rhs: TypeId) -> Result<()> {
        eprintln!("Unifying {} and {}", self.ctx.desc(lhs), self.ctx.desc(rhs));
        self.ctx.unify(lhs, rhs)
    }
    fn import_literals(&mut self) {
        for (slot, lit) in &self.mir.literals {
            let ty = match lit {
                ExprLit::TypedBits(tb) => self.ctx.from_kind(&tb.value.kind),
                ExprLit::Int(_) => self.ctx.ty_integer(),
                ExprLit::Bool(_) => self.ctx.ty_bool(),
            };
            self.slot_map.insert(*slot, ty);
        }
    }
    fn import_signature(&mut self) -> Result<()> {
        for slot in &self.mir.arguments {
            let Some(ty) = self.mir.ty.get(slot) else {
                bail!("Missing type for argument slot {}", slot);
            };
            let ty = self.ctx.from_kind(ty);
            self.slot_map.insert(*slot, ty);
        }
        let Some(return_ty) = self.mir.ty.get(&self.mir.return_slot) else {
            bail!("Missing type for return slot");
        };
        let return_ty = self.ctx.from_kind(return_ty);
        self.slot_map.insert(self.mir.return_slot, return_ty);
        Ok(())
    }
    fn import_type_equality(&mut self) -> Result<()> {
        for (lhs, rhs) in &self.mir.ty_equate {
            let lhs_ty = self.slot_ty(*lhs);
            let rhs_ty = self.slot_ty(*rhs);
            self.unify(lhs_ty, rhs_ty)?;
        }
        Ok(())
    }
    fn slot_ty(&mut self, slot: Slot) -> TypeId {
        if matches!(slot, Slot::Empty) {
            return self.ctx.ty_empty();
        }
        if let Some(ty) = self.slot_map.get(&slot) {
            *ty
        } else {
            let var = self.ctx.ty_var();
            self.slot_map.insert(slot, var);
            var
        }
    }
    fn slot_tys(&mut self, slots: &[Slot]) -> Vec<TypeId> {
        slots.iter().map(|slot| self.slot_ty(*slot)).collect()
    }
    fn all_slots_resolved(&mut self) -> bool {
        let slot_tys = self.slot_map.values().copied().collect::<Vec<_>>();
        slot_tys
            .into_iter()
            .all(|ty| self.ctx.into_kind(ty).is_ok())
    }
    fn try_unary(&mut self, op: &TypeUnaryOp) -> Result<()> {
        let a1 = self.ctx.apply(op.arg1);
        match op.op {
            AluUnary::All | AluUnary::Any | AluUnary::Xor => {
                let bool_ty = self.ctx.ty_bool();
                if self.ctx.is_signal(a1) {
                    let clock_ty = self.ctx.ty_var();
                    let bool_sig = self.ctx.ty_signal(bool_ty, clock_ty);
                    self.unify(op.lhs, bool_sig)?;
                    if let Some(a1_clock) = self.ctx.project_signal_clock(a1) {
                        self.unify(clock_ty, a1_clock)?;
                    }
                } else {
                    self.unify(op.lhs, bool_ty)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn try_binop(&mut self, op: &TypeBinOp) -> Result<()> {
        let a1 = self.ctx.apply(op.arg1);
        let a2 = self.ctx.apply(op.arg2);
        eprintln!(
            "Try to apply {} to {} and {}",
            op.op,
            self.ctx.desc(a1),
            self.ctx.desc(a2)
        );
        match &op.op {
            AluBinary::Add
            | AluBinary::Mul
            | AluBinary::BitAnd
            | AluBinary::BitOr
            | AluBinary::BitXor
            | AluBinary::Sub => {
                let a1_is_signal = self.ctx.is_signal(op.arg1);
                let a2_is_signal = self.ctx.is_signal(op.arg2);
                if a1_is_signal {
                    self.unify(op.lhs, op.arg1)?;
                }
                if a2_is_signal {
                    self.unify(op.lhs, op.arg2)?;
                }
                if !a1_is_signal && !a2_is_signal {
                    self.unify(op.lhs, op.arg1)?;
                    self.unify(op.lhs, op.arg2)?;
                }
                if let (Some(arg1_data), Some(arg2_data)) = (
                    self.ctx.project_signal_value(op.arg1),
                    self.ctx.project_signal_value(op.arg2),
                ) {
                    self.unify(arg1_data, arg2_data)?;
                }
                if let (Some(lhs_data), Some(arg1_data)) = (
                    self.ctx.project_signal_value(op.lhs),
                    self.ctx.project_signal_value(op.arg1),
                ) {
                    self.unify(lhs_data, arg1_data)?;
                }
                if let (Some(lhs_data), Some(arg2_data)) = (
                    self.ctx.project_signal_value(op.lhs),
                    self.ctx.project_signal_value(op.arg2),
                ) {
                    self.unify(lhs_data, arg2_data)?;
                }
            }
            AluBinary::Eq
            | AluBinary::Lt
            | AluBinary::Le
            | AluBinary::Ne
            | AluBinary::Ge
            | AluBinary::Gt => {
                if let Some(arg1_clock) = self.ctx.project_signal_clock(op.arg1) {
                    let lhs_var = self.ctx.ty_bool();
                    let lhs_sig = self.ctx.ty_signal(lhs_var, arg1_clock);
                    self.unify(op.lhs, lhs_sig)?;
                }
                if let Some(arg2_clock) = self.ctx.project_signal_clock(op.arg2) {
                    let lhs_var = self.ctx.ty_bool();
                    let lhs_sig = self.ctx.ty_signal(lhs_var, arg2_clock);
                    self.unify(op.lhs, lhs_sig)?;
                }
                if !self.ctx.is_signal(op.arg1) && !self.ctx.is_signal(op.arg2) {
                    let lhs_var = self.ctx.ty_bool();
                    self.unify(op.lhs, lhs_var)?;
                }
                if let (Some(arg1_data), Some(arg2_data)) = (
                    self.ctx.project_signal_value(op.arg1),
                    self.ctx.project_signal_value(op.arg2),
                ) {
                    self.unify(arg1_data, arg2_data)?;
                }
            }
            AluBinary::Shl | AluBinary::Shr => {
                if let (Some(lhs_data), Some(arg1_data)) = (
                    self.ctx.project_signal_value(op.lhs),
                    self.ctx.project_signal_value(op.arg1),
                ) {
                    self.unify(lhs_data, arg1_data)?;
                }
            }
        }
        Ok(())
    }

    fn ty_path_project(&mut self, arg: TypeId, path: &Path) -> Result<TypeId> {
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
                    arg = self.ctx.ty_enum_discriminant(arg)?;
                }
                PathElement::TupleIndex(ndx) => {
                    arg = self.ctx.ty_index(arg, *ndx)?;
                }
                PathElement::EnumPayload(member) => {
                    arg = self.ctx.ty_variant(arg, member)?;
                }
                PathElement::DynamicIndex(slot) => {
                    let index = self.slot_ty(*slot);
                    let usize_ty = self.ctx.ty_usize();
                    if slot.is_literal() {
                        self.ctx.unify(index, usize_ty)?;
                    } else {
                        let reg_ty = self.ctx.apply(index);
                        if self.ctx.is_generic_integer(reg_ty) {
                            // For more clearly defined types, it is someone else's problem
                            // to ensure that the index is properly typed.
                            self.ctx.unify(reg_ty, usize_ty)?;
                        }
                    }
                    arg = self.ctx.ty_index(arg, 0)?;
                }
                _ => {
                    bail!("Unsupported path element {:?} in path", element);
                }
            }
        }
        Ok(arg)
    }

    fn try_index(&mut self, op: &TypeIndex) -> Result<()> {
        eprintln!(
            "Try to apply index to {} with path {}",
            self.ctx.desc(op.arg),
            op.path
        );
        let arg = self.ctx.apply(op.arg);
        match self.ty_path_project(arg, &op.path) {
            Ok(ty) => self.unify(op.lhs, ty),
            Err(err) => {
                eprintln!("Error: {}", err);
                Ok(())
            }
        }
    }
    fn enforce_clocks(&mut self, t: &[TypeId]) -> Result<()> {
        for first in t.iter() {
            for second in t.iter() {
                if let (Some(clock1), Some(clock2)) = (
                    self.ctx.project_signal_clock(*first),
                    self.ctx.project_signal_clock(*second),
                ) {
                    self.unify(clock1, clock2)?;
                }
            }
        }
        Ok(())
    }
    fn try_select(&mut self, op: &TypeSelect) -> Result<()> {
        self.enforce_clocks(&[op.selector, op.true_value, op.false_value])
    }
    fn try_type_op(&mut self, op: &TypeOperation) -> Result<()> {
        match op {
            TypeOperation::BinOp(binop) => self.try_binop(binop),
            TypeOperation::Index(index) => self.try_index(index),
            TypeOperation::UnaryOp(unary) => self.try_unary(unary),
            TypeOperation::Select(select) => self.try_select(select),
        }
    }
    fn process_ops(&mut self) -> Result<()> {
        for op in &self.mir.ops {
            eprintln!("Processing op {}", op.op);
            match &op.op {
                OpCode::Array(array) => {
                    let lhs = self.slot_ty(array.lhs);
                    let rhs = self.slot_tys(&array.elements);
                    let array_base = self.ctx.ty_var();
                    let array_len = self.ctx.ty_const_len(rhs.len());
                    let lhs_ty = self.ctx.ty_array(array_base, array_len);
                    self.unify(lhs, lhs_ty)?;
                    for element in rhs {
                        self.unify(element, array_base)?;
                    }
                }
                OpCode::Assign(assign) => {
                    let lhs = self.slot_ty(assign.lhs);
                    let rhs = self.slot_ty(assign.rhs);
                    self.unify(lhs, rhs)?;
                }
                OpCode::AsBits(as_bits) => {
                    let arg = self.slot_ty(as_bits.arg);
                    let lhs = self.slot_ty(as_bits.lhs);
                    let len = if let Some(len) = as_bits.len {
                        self.ctx.ty_const_len(len)
                    } else {
                        self.ctx.ty_var()
                    };
                    let lhs_ty = self.ctx.ty_bits(len);
                    self.unify(lhs, lhs_ty)?;
                    let len_128 = self.ctx.ty_const_len(128);
                    let arg_ty = self.ctx.ty_bits(len_128);
                    self.unify(arg, arg_ty)?;
                }
                OpCode::AsKind(as_kind) => {
                    let lhs = self.slot_ty(as_kind.lhs);
                    let kind = self.ctx.from_kind(&as_kind.kind);
                    self.unify(lhs, kind)?;
                }
                OpCode::AsSigned(as_signed) => {
                    let arg = self.slot_ty(as_signed.arg);
                    let lhs = self.slot_ty(as_signed.lhs);
                    let len = if let Some(len) = as_signed.len {
                        self.ctx.ty_const_len(len)
                    } else {
                        self.ctx.ty_var()
                    };
                    let lhs_ty = self.ctx.ty_signed(len);
                    self.unify(lhs, lhs_ty)?;
                    let len_128 = self.ctx.ty_const_len(128);
                    let arg_ty = self.ctx.ty_signed(len_128);
                    self.unify(arg, arg_ty)?;
                }
                OpCode::Binary(binary) => {
                    let lhs = self.slot_ty(binary.lhs);
                    let arg1 = self.slot_ty(binary.arg1);
                    let arg2 = self.slot_ty(binary.arg2);
                    self.type_ops.push(TypeOperation::BinOp(TypeBinOp {
                        op: binary.op,
                        lhs,
                        arg1,
                        arg2,
                    }));
                }
                OpCode::Case(case) => {
                    let lhs = self.slot_ty(case.lhs);
                    let disc = self.slot_ty(case.discriminant);
                    for (test, value) in case.table.iter() {
                        match test {
                            CaseArgument::Constant(_) => bail!("Constant in case table"),
                            CaseArgument::Slot(slot) => {
                                let ty = self.slot_ty(*slot);
                                let free_var = self.ctx.ty_var();
                                eprintln!(
                                    "Adding constraint {} = {} == {}",
                                    self.ctx.desc(free_var),
                                    self.ctx.desc(disc),
                                    self.ctx.desc(ty)
                                );
                                self.type_ops.push(TypeOperation::BinOp(TypeBinOp {
                                    op: AluBinary::Eq,
                                    lhs: free_var,
                                    arg1: disc,
                                    arg2: ty,
                                }));
                            }
                            CaseArgument::Wild => {}
                        }
                        let val_ty = self.slot_ty(*value);
                        self.unify(lhs, val_ty)?;
                    }
                }
                OpCode::Enum(enumerate) => {
                    let lhs = self.slot_ty(enumerate.lhs);
                    let Kind::Enum(enum_k) = &enumerate.template.kind else {
                        bail!("Expected Enum kind");
                    };
                    let lhs_ty = self.ctx.ty_enum(enum_k);
                    self.unify(lhs, lhs_ty)?;
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
                        let field_kind = sub_kind(enumerate.template.kind.clone(), &path)?;
                        let field_ty = self.ctx.from_kind(&field_kind);
                        let field_slot = self.slot_ty(field.value);
                        self.unify(field_ty, field_slot)?;
                    }
                }
                OpCode::Exec(exec) => {
                    let external_fn = &self.mir.stash[exec.id.0];
                    let signature = &external_fn.signature;
                    for (arg_kind, arg_slot) in signature.arguments.iter().zip(exec.args.iter()) {
                        let arg_ty = self.slot_ty(*arg_slot);
                        let arg_kind = self.ctx.from_kind(arg_kind);
                        self.unify(arg_ty, arg_kind)?;
                    }
                    let ret_ty = self.slot_ty(exec.lhs);
                    let ret_kind = self.ctx.from_kind(&signature.ret);
                    self.unify(ret_ty, ret_kind)?;
                }
                OpCode::Index(index) => {
                    let arg = self.slot_ty(index.arg);
                    let lhs = self.slot_ty(index.lhs);
                    let path = index.path.clone();
                    self.type_ops
                        .push(TypeOperation::Index(TypeIndex { lhs, arg, path }));
                }
                OpCode::Repeat(repeat) => {
                    let lhs = self.slot_ty(repeat.lhs);
                    let value = self.slot_ty(repeat.value);
                    let len = self.ctx.ty_const_len(repeat.len as usize);
                    let lhs_ty = self.ctx.ty_array(value, len);
                    self.unify(lhs, lhs_ty)?;
                }
                OpCode::Select(select) => {
                    let lhs = self.slot_ty(select.lhs);
                    let cond = self.slot_ty(select.cond);
                    let arg1 = self.slot_ty(select.true_value);
                    let arg2 = self.slot_ty(select.false_value);
                    self.unify(lhs, arg1)?;
                    self.unify(lhs, arg2)?;
                    self.type_ops.push(TypeOperation::Select(TypeSelect {
                        lhs,
                        selector: cond,
                        true_value: arg1,
                        false_value: arg2,
                    }));
                }
                OpCode::Splice(splice) => {
                    let lhs = self.slot_ty(splice.lhs);
                    let orig = self.slot_ty(splice.orig);
                    let subst = self.slot_ty(splice.subst);
                    let path = &splice.path;
                    self.unify(lhs, orig)?;
                    // Reflect the constraint that
                    // ty(subst) = ty(lhs[path])
                    self.type_ops.push(TypeOperation::Index(TypeIndex {
                        lhs: subst,
                        arg: lhs,
                        path: path.clone(),
                    }));
                }
                OpCode::Struct(structure) => {
                    let lhs = self.slot_ty(structure.lhs);
                    let Kind::Struct(strukt) = &structure.template.kind else {
                        bail!("Expected Struct kind");
                    };
                    let lhs_ty = self.ctx.ty_struct(strukt);
                    self.unify(lhs, lhs_ty)?;
                    for field in &structure.fields {
                        let field_kind = strukt.get_field_kind(&field.member)?;
                        let field_ty = self.ctx.from_kind(&field_kind);
                        let field_slot = self.slot_ty(field.value);
                        self.unify(field_ty, field_slot)?;
                    }
                    if let Some(rest) = structure.rest {
                        let rest_ty = self.slot_ty(rest);
                        self.unify(lhs_ty, rest_ty)?;
                    }
                    self.unify(lhs, lhs_ty)?;
                }
                OpCode::Tuple(tuple) => {
                    let lhs = self.slot_ty(tuple.lhs);
                    let tys = tuple
                        .fields
                        .iter()
                        .map(|slot| self.slot_ty(*slot))
                        .collect();
                    let lhs_ty = self.ctx.ty_tuple(tys);
                    self.unify(lhs, lhs_ty)?;
                }
                OpCode::Unary(unary) => {
                    let lhs = self.slot_ty(unary.lhs);
                    let arg1 = self.slot_ty(unary.arg1);
                    match unary.op {
                        AluUnary::Not => {
                            self.unify(lhs, arg1)?;
                        }
                        AluUnary::Neg => {
                            let len = self.ctx.ty_var();
                            let signed_ty = self.ctx.ty_signed(len);
                            self.unify(lhs, signed_ty)?;
                            self.unify(arg1, signed_ty)?;
                        }
                        AluUnary::All | AluUnary::Any | AluUnary::Xor => {
                            self.type_ops.push(TypeOperation::UnaryOp(TypeUnaryOp {
                                op: unary.op,
                                lhs,
                                arg1,
                            }));
                        }
                        AluUnary::Unsigned => {
                            let len = self.ctx.ty_var();
                            let signed_ty = self.ctx.ty_signed(len);
                            let unsigned_ty = self.ctx.ty_bits(len);
                            self.unify(lhs, unsigned_ty)?;
                            self.unify(arg1, signed_ty)?;
                        }
                        AluUnary::Signed => {
                            let len = self.ctx.ty_var();
                            let signed_ty = self.ctx.ty_signed(len);
                            let unsigned_ty = self.ctx.ty_bits(len);
                            self.unify(lhs, signed_ty)?;
                            self.unify(arg1, unsigned_ty)?;
                        }
                    }
                }
                _ => {}
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
    eprintln!("=================================");
    eprintln!("Before inference");
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        eprintln!("Slot {} -> type {}", slot, ty);
    }
    for op in mir.ops.iter() {
        eprintln!("{}", op.op);
    }
    eprintln!("=================================");
    if let Err(e) = infer.process_ops() {
        eprintln!("Error: {}", e);
        for (slot, ty) in &infer.slot_map {
            let ty = infer.ctx.apply(*ty);
            let ty = infer.ctx.desc(ty);
            eprintln!("Slot {} -> type {}", slot, ty);
        }
        bail!("Type inference failed {e}");
    }
    infer.process_ops()?;
    let type_ops = infer.type_ops.clone();
    let mut loop_count = 0;
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        eprintln!("Slot {} -> type {}", slot, ty);
    }
    for loop_count in 0..3 {
        type_ops
            .iter()
            .map(|op| infer.try_type_op(op))
            .collect::<Result<Vec<_>>>()?;
        if infer.all_slots_resolved() {
            break;
        }
    }
    // Try to replace generic literals with i32s
    if !infer.all_slots_resolved() {
        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if infer.ctx.is_generic_integer(ty) {
                let i32_len = infer.ctx.ty_const_len(32);
                let i32_ty = infer.ctx.ty_signed(i32_len);
                infer.unify(ty, i32_ty)?;
            }
        }
    }
    for loop_count in 0..3 {
        type_ops
            .iter()
            .map(|op| infer.try_type_op(op))
            .collect::<Result<Vec<_>>>()?;
        if infer.all_slots_resolved() {
            break;
        }
    }
    if !infer.all_slots_resolved() {
        eprintln!("=================================");
        eprintln!("Inference failed");
        for (slot, ty) in &infer.slot_map {
            let ty = infer.ctx.apply(*ty);
            let ty = infer.ctx.desc(ty);
            eprintln!("Slot {} -> type {}", slot, ty);
        }
        for op in mir.ops.iter() {
            eprintln!("{}", op.op);
        }

        eprintln!("=================================");

        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if infer.ctx.into_kind(ty).is_err() {
                eprintln!("Literal {} -> {}", lit, infer.ctx.desc(ty));
            }
        }

        bail!("Inference failure detected");
    }
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        eprintln!("Slot {} -> type {}", slot, ty);
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
        .map(|(slot, ty)| infer.ctx.into_kind(*ty).map(|val| (*slot, val)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    for op in mir.ops.iter() {
        eprintln!("{}", op.op);
    }
    let literals = mir
        .literals
        .clone()
        .into_iter()
        .map(|(slot, lit)| {
            infer
                .cast_literal_to_inferred_type(lit, final_type_map[&slot])
                .map(|value| (slot, value))
        })
        .collect::<Result<_>>()?;
    let ops = mir.ops.into_iter().map(|op| op.op).collect();
    Ok(Object {
        symbols: mir.symbols,
        ops,
        literals,
        kind,
        arguments: mir.arguments.clone(),
        return_slot: mir.return_slot,
        externals: mir.stash,
        name: mir.name,
        fn_id: mir.fn_id,
    })
}
