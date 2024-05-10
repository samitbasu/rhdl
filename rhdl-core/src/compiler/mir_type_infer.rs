use std::collections::BTreeMap;

use crate::{
    ast::ast_impl::ExprLit,
    path::{sub_kind, Path, PathElement},
    rhif::spec::{AluBinary, AluUnary, CaseArgument, OpCode, Slot},
    Kind,
};
use anyhow::bail;
use anyhow::Result;

use super::{
    mir::Mir,
    mir_ty::{TypeId, UnifyContext},
};

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
pub enum TypeOperation {
    BinOp(TypeBinOp),
    Index(TypeIndex),
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
    fn slot_ty(&mut self, slot: Slot) -> TypeId {
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
        slot_tys.into_iter().all(|ty| self.ctx.is_resolved(ty))
    }
    fn try_binop(&mut self, op: &TypeBinOp) -> Result<()> {
        eprintln!(
            "Try to apply {} to {} and {}",
            op.op,
            self.ctx.desc(op.arg1),
            self.ctx.desc(op.arg2)
        );
        match &op.op {
            AluBinary::Add | AluBinary::BitAnd | AluBinary::BitOr | AluBinary::BitXor => {
                if self.ctx.is_signal(op.arg1) {
                    self.unify(op.lhs, op.arg1)?;
                }
                if self.ctx.is_signal(op.arg2) {
                    self.unify(op.lhs, op.arg2)?;
                }
                if self.ctx.equal(op.arg1, op.arg2) {
                    self.unify(op.lhs, op.arg1)?;
                    self.unify(op.lhs, op.arg2)?;
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
            }
            _ => bail!("Not implemented"),
        }
        Ok(())
    }
    fn try_index(&mut self, op: &TypeIndex) -> Result<()> {
        eprintln!(
            "Try to apply index to {} with path {}",
            self.ctx.desc(op.arg),
            op.path
        );
        let arg = self.ctx.apply(op.arg);
        let arg_kind = self.ctx.into_kind(arg)?;
        let path = approximate_dynamic_paths(&op.path);
        eprintln!("Index base kind: {}", arg_kind);
        let sub_kind = sub_kind(arg_kind, &path)?;
        eprintln!("Sub-kind: {}", sub_kind);
        let sub_kind_ty = self.ctx.from_kind(&sub_kind);
        self.unify(op.lhs, sub_kind_ty)
    }
    fn try_type_op(&mut self, op: &TypeOperation) -> Result<()> {
        match op {
            TypeOperation::BinOp(binop) => self.try_binop(binop),
            TypeOperation::Index(index) => self.try_index(index),
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
                    let lhs = self.slot_ty(as_bits.lhs);
                    let len = self.ctx.ty_const_len(as_bits.len);
                    let lhs_ty = self.ctx.ty_bits(len);
                    self.unify(lhs, lhs_ty)?;
                }
                OpCode::AsKind(as_kind) => {
                    let lhs = self.slot_ty(as_kind.lhs);
                    let kind = self.ctx.from_kind(&as_kind.kind);
                    self.unify(lhs, kind)?;
                }
                OpCode::AsSigned(as_signed) => {
                    let lhs = self.slot_ty(as_signed.lhs);
                    let len = self.ctx.ty_const_len(as_signed.len);
                    let lhs_ty = self.ctx.ty_signed(len);
                    self.unify(lhs, lhs_ty)?;
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
                    let Kind::Enum(enumerate) = &enumerate.template.kind else {
                        bail!("Expected Enum kind");
                    };
                    let lhs_ty = self.ctx.ty_enum(enumerate);
                    self.unify(lhs, lhs_ty)?;
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
                    let len = self.ctx.ty_const_len(repeat.len);
                    let lhs_ty = self.ctx.ty_array(value, len);
                    self.unify(lhs, lhs_ty)?;
                }
                OpCode::Select(select) => {
                    let lhs = self.slot_ty(select.lhs);
                    let arg1 = self.slot_ty(select.true_value);
                    let arg2 = self.slot_ty(select.false_value);
                    self.unify(lhs, arg1)?;
                    self.unify(lhs, arg2)?;
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
                            let bool_ty = self.ctx.ty_bool();
                            self.unify(lhs, bool_ty)?;
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

pub fn infer(mir: &Mir) -> Result<BTreeMap<Slot, Kind>> {
    let mut infer = MirTypeInference::new(mir);
    infer.import_literals();
    infer.import_signature()?;
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
    loop {
        type_ops
            .iter()
            .map(|op| infer.try_type_op(op))
            .collect::<Result<Vec<_>>>()?;
        if infer.all_slots_resolved() {
            break;
        }
        loop_count += 1;
        if loop_count > 10 {
            bail!("Inference loop detected");
        }
    }
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        eprintln!("Slot {} -> type {}", slot, ty);
    }
    for op in mir.ops.iter() {
        eprintln!("{}", op.op);
    }
    Ok(BTreeMap::new())
}
