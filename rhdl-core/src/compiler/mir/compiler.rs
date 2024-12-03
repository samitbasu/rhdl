// Convert the AST into the mid-level representation, which is RHIF, but with out
// concrete types.

// AST elements that carry Type information:
// PatType
// ExprType - not covered
// ExprStruct
// ArmEnum
// KernelFn (ret)

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::once;

use crate::ast::ast_impl;
use crate::ast::ast_impl::BitsKind;
use crate::ast::ast_impl::ExprBits;
use crate::ast::ast_impl::ExprTry;
use crate::ast::ast_impl::ExprTypedBits;
use crate::ast::ast_impl::NodeId;
use crate::ast::ast_impl::WrapOp;
use crate::ast::ast_impl::{
    Arm, ArmKind, Block, ExprArray, ExprAssign, ExprBinary, ExprCall, ExprField, ExprForLoop,
    ExprIf, ExprIndex, ExprMatch, ExprMethodCall, ExprPath, ExprRepeat, ExprRet, ExprStruct,
    ExprTuple, ExprUnary, FieldValue, Local, Pat, PatKind, Stmt, StmtKind,
};
use crate::ast::spanned_source::SpannedSource;
use crate::ast::syn_spanned_source::build_spanned_source_for_kernel;
use crate::ast::visit::Visitor;
use crate::ast_builder::BinOp;
use crate::ast_builder::UnOp;
use crate::bitx::bitx_string;
use crate::compiler::ascii;
use crate::compiler::display_ast::pretty_print_statement;
use crate::compiler::stage1::compile;
use crate::compiler::stage1::CompilationMode;
use crate::error::RHDLError;
use crate::kernel::Kernel;
use crate::rhif;
use crate::rhif::object::LocatedOpCode;
use crate::rhif::object::SymbolMap;
use crate::rhif::rhif_builder::op_as_bits_inferred;
use crate::rhif::rhif_builder::op_as_signed_inferred;
use crate::rhif::rhif_builder::op_resize;
use crate::rhif::rhif_builder::op_resize_inferred;
use crate::rhif::rhif_builder::op_retime;
use crate::rhif::rhif_builder::op_wrap;
use crate::rhif::spec::AluUnary;
use crate::rhif::spec::CaseArgument;
use crate::rhif::spec::Member;
use crate::rhif::spec::RegisterId;
use crate::rhif::spec::{FuncId, LiteralId};
use crate::rhif::Object;
use crate::rhif::{
    rhif_builder::{
        op_array, op_as_bits, op_as_signed, op_assign, op_binary, op_case, op_comment, op_enum,
        op_exec, op_index, op_repeat, op_select, op_splice, op_struct, op_tuple, op_unary,
    },
    spec::AluBinary,
};
use crate::types::path::Path;
use crate::KernelFnKind;
use crate::Kind;
use crate::TypedBits;
use crate::{
    ast::ast_impl::{Expr, ExprKind, ExprLit, FunctionId},
    rhif::spec::{OpCode, Slot},
};

use super::error::RHDLCompileError;
use super::error::RHDLSyntaxError;
use super::error::Syntax;
use super::error::ICE;
use super::interner::Intern;
use super::interner::InternKey;
use super::mir_impl::Mir;
use super::mir_impl::TypeEquivalence;

#[derive(Debug, Clone)]
pub struct Rebind {
    from: Slot,
    to: Slot,
}

fn binop_to_alu(op: BinOp) -> AluBinary {
    match op {
        BinOp::Add | BinOp::AddAssign => AluBinary::Add,
        BinOp::Sub | BinOp::SubAssign => AluBinary::Sub,
        BinOp::Mul | BinOp::MulAssign => AluBinary::Mul,
        BinOp::BitXor | BinOp::BitXorAssign => AluBinary::BitXor,
        BinOp::And | BinOp::BitAnd | BinOp::BitAndAssign => AluBinary::BitAnd,
        BinOp::Or | BinOp::BitOr | BinOp::BitOrAssign => AluBinary::BitOr,
        BinOp::Shl | BinOp::ShlAssign => AluBinary::Shl,
        BinOp::Shr | BinOp::ShrAssign => AluBinary::Shr,
        BinOp::Eq => AluBinary::Eq,
        BinOp::Lt => AluBinary::Lt,
        BinOp::Le => AluBinary::Le,
        BinOp::Ne => AluBinary::Ne,
        BinOp::Ge => AluBinary::Ge,
        BinOp::Gt => AluBinary::Gt,
    }
}

fn unary_op_to_alu(op: UnOp) -> AluUnary {
    match op {
        UnOp::Neg => AluUnary::Neg,
        UnOp::Not => AluUnary::Not,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ScopeId(usize);

const ROOT_SCOPE: ScopeId = ScopeId(0);

impl Default for ScopeId {
    fn default() -> Self {
        ROOT_SCOPE
    }
}

fn path_as_ident(path: &ast_impl::Path) -> Option<&'static str> {
    if path.segments.len() == 1 {
        Some(path.segments[0].ident)
    } else {
        None
    }
}

fn coerce_literal_to_i32(val: &ExprLit) -> Result<i32> {
    match val {
        ExprLit::Int(i) => i.parse::<i32>().map_err(|err| err.into()),
        ExprLit::Bool(b) => Ok(if *b { 1 } else { 0 }),
        ExprLit::TypedBits(tb) => tb.value.as_i64().map(|x| x as i32),
    }
}

type KindKey = InternKey<Kind>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeIndex {
    scope: ScopeId,
    name: &'static str,
}

type LocalsMap = HashMap<ScopeIndex, Slot>;

struct Scope {
    names: HashMap<&'static str, Slot>,
    children: Vec<ScopeId>,
    parent: ScopeId,
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope {{")?;
        for (name, id) in &self.names {
            writeln!(f, "  {} -> {:?}", name, id)?;
        }
        writeln!(f, "}}")
    }
}

const EARLY_RETURN_FLAG_NAME: &str = "__$early_return_flag";

type Result<T> = std::result::Result<T, RHDLError>;

pub struct MirContext<'a> {
    kinds: Intern<Kind>,
    scopes: Vec<Scope>,
    ops: Vec<LocatedOpCode>,
    reg_count: usize,
    literals: BTreeMap<Slot, ExprLit>,
    reg_source_map: BTreeMap<Slot, NodeId>,
    ty: BTreeMap<Slot, KindKey>,
    ty_equate: HashSet<TypeEquivalence>,
    stash: BTreeMap<FuncId, Box<Object>>,
    slot_names: BTreeMap<Slot, String>,
    return_slot: Slot,
    arguments: Vec<Slot>,
    fn_id: FunctionId,
    name: &'static str,
    active_scope: ScopeId,
    spanned_source: &'a SpannedSource,
    mode: CompilationMode,
}

impl<'a> std::fmt::Debug for MirContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arguments = self
            .arguments
            .iter()
            .map(|x| format!("{x:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            f,
            "Kernel {}({})->{:?} ({:?})",
            self.name, arguments, self.return_slot, self.fn_id
        )?;
        for (slot, kind) in &self.ty {
            writeln!(f, "{:?} : {:?}", slot, self.kinds[kind])?;
        }
        for (lit, expr) in &self.literals {
            writeln!(f, "{:?} -> {:?}", lit, expr)?;
        }
        for (id, func) in self.stash.iter() {
            writeln!(f, "Function f{:?} {:?}", id, func)?;
        }
        for op in &self.ops {
            writeln!(f, "{:?}", op.op)?;
        }
        Ok(())
    }
}

impl<'a> MirContext<'a> {
    fn new(spanned_source: &'a SpannedSource, mode: CompilationMode, fn_id: FunctionId) -> Self {
        MirContext {
            kinds: Intern::default(),
            scopes: vec![Scope {
                names: HashMap::new(),
                children: vec![],
                parent: ROOT_SCOPE,
            }],
            ops: vec![],
            reg_count: 0,
            literals: BTreeMap::new(),
            reg_source_map: BTreeMap::new(),
            ty: BTreeMap::new(),
            ty_equate: Default::default(),
            stash: Default::default(),
            return_slot: Slot::Empty,
            arguments: vec![],
            fn_id,
            name: "",
            active_scope: ROOT_SCOPE,
            slot_names: BTreeMap::new(),
            spanned_source,
            mode,
        }
    }

    fn new_scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
            names: HashMap::new(),
            children: Vec::new(),
            parent: self.active_scope,
        });
        self.scopes[self.active_scope.0].children.push(id);
        self.active_scope = id;
        id
    }
    fn end_scope(&mut self) {
        self.active_scope = self.scopes[self.active_scope.0].parent;
    }
    fn bind_slot_to_type(&mut self, slot: Slot, kind: &Kind) {
        let key = self.kinds.intern(kind);
        self.ty.insert(slot, key);
    }
    // Walk the scope hierarchy, and return a list of all local variables
    // visible from the current scope.  Some of these may not be accessible
    // due to shadowing.
    fn locals(&self) -> LocalsMap {
        let mut locals = LocalsMap::new();
        let mut scope = self.active_scope;
        loop {
            locals.extend(
                self.scopes[scope.0]
                    .names
                    .iter()
                    .map(|(k, v)| (ScopeIndex { scope, name: k }, *v)),
            );
            if scope == ROOT_SCOPE {
                break;
            }
            scope = self.scopes[scope.0].parent;
        }
        locals
    }
    fn set_locals(&mut self, map: &LocalsMap, loc: NodeId) -> Result<()> {
        for (ndx, slot) in map {
            let ScopeIndex { scope, name } = ndx;
            let Some(binding) = self.scopes[scope.0].names.get_mut(name) else {
                return Err(self
                    .raise_ice(
                        ICE::LocalVariableDoesNotExist {
                            name: (*name).to_owned(),
                        },
                        loc,
                    )
                    .into());
            };
            *binding = *slot;
        }
        Ok(())
    }
    fn unpack_arguments(&mut self, args: &[Box<Pat>], id: NodeId) -> Result<()> {
        for arg in args {
            let slot = self.reg(arg.id);
            let PatKind::Type(ty) = &arg.kind else {
                return Err(self
                    .raise_ice(ICE::UnsupportedArgumentPattern { arg: arg.clone() }, id)
                    .into());
            };
            self.bind_slot_to_type(slot, &ty.kind);
            self.arguments.push(slot);
        }
        Ok(())
    }
    // Create a local variable binding to the given name, and return the
    // resulting register.
    fn bind(&mut self, name: &'static str, id: NodeId) {
        let reg = self.reg(id);
        eprintln!("Binding {}#{:?} to {:?}", name, id, reg);
        self.slot_names.insert(reg, name.to_string());
        self.scopes[self.active_scope.0].names.insert(name, reg);
    }
    // Rebind a local variable to a new slot.  We need
    // to know the previous name for the slot in this case.
    // For example, if we have:
    // ```
    //  let a = 5; //<-- original binding to name of "a"
    //  let a = 6; //<-- rebind of "a" to new slot
    //```
    fn rebind(&mut self, name: &'static str, id: NodeId) -> Result<Rebind> {
        let reg = self.reg(id);
        let Some((prev, scope)) = self.lookup_name(name) else {
            return Err(self
                .raise_ice(
                    ICE::RebindOfUnboundVariable {
                        name: name.to_string(),
                    },
                    id,
                )
                .into());
        };
        self.slot_names.insert(reg, name.to_string());
        self.scopes[scope.0].names.insert(name, reg);
        eprintln!("Rebound {} from {:?} to {:?}", name, prev, reg);
        Ok(Rebind {
            from: prev,
            to: reg,
        })
    }
    fn reg(&mut self, id: NodeId) -> Slot {
        let reg = Slot::Register(RegisterId(self.reg_count));
        self.reg_source_map.insert(reg, id);
        self.reg_count += 1;
        reg
    }
    fn lit(&mut self, id: NodeId, lit: ExprLit) -> Slot {
        let ndx = self.literals.len();
        let slot = Slot::Literal(LiteralId(ndx));
        eprintln!("Allocate literal {:?} for {:?} -> {:?}", lit, id, slot);
        self.literals.insert(slot, lit);
        self.reg_source_map.insert(slot, id);
        slot
    }
    fn literal_int(&mut self, id: NodeId, val: i32) -> Slot {
        self.lit(id, ExprLit::Int(val.to_string()))
    }
    fn literal_bool(&mut self, id: NodeId, val: bool) -> Slot {
        self.lit(id, ExprLit::Bool(val))
    }
    fn literal_tb(&mut self, id: NodeId, place_holder: &TypedBits) -> Slot {
        self.lit(
            id,
            ExprLit::TypedBits(ExprTypedBits {
                path: Box::new(ast_impl::Path { segments: vec![] }),
                value: place_holder.clone(),
                code: String::new(),
            }),
        )
    }
    fn lookup_name(&self, path: &str) -> Option<(Slot, ScopeId)> {
        let mut scope = self.active_scope;
        loop {
            if let Some(id) = self.scopes[scope.0].names.get(path) {
                return Some((*id, scope));
            }
            if scope == ROOT_SCOPE {
                break;
            }
            scope = self.scopes[scope.0].parent;
        }
        None
    }
    fn slot_to_index(&self, slot: Slot, id: NodeId) -> Result<usize> {
        let Some(value) = self.literals.get(&slot) else {
            return Err(self
                .raise_ice(ICE::SlotToIndexNonLiteralSlot { slot }, id)
                .into());
        };
        let ndx = coerce_literal_to_i32(value)? as usize;
        Ok(ndx)
    }
    fn initialize_local(&mut self, pat: &Pat, rhs: Slot) -> Result<()> {
        match &pat.kind {
            PatKind::Ident(ident) => {
                let Some((lhs, _)) = self.lookup_name(ident.name) else {
                    return Err(self
                        .raise_ice(
                            ICE::InitializeLocalOnUnboundVariable {
                                name: ident.name.to_owned(),
                            },
                            pat.id,
                        )
                        .into());
                };
                self.op(op_assign(lhs, rhs), pat.id);
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                for (ndx, pat) in tuple.elements.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(
                            element_rhs,
                            rhs,
                            crate::types::path::Path::default().tuple_index(ndx),
                        ),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Struct(struct_pat) => {
                for field in &struct_pat.fields {
                    let element_rhs = self.reg(field.pat.id);
                    let path = Path::default().member(&field.member);
                    self.op(op_index(element_rhs, rhs, path), field.pat.id);
                    self.initialize_local(&field.pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(tuple_struct_pat) => {
                for (ndx, pat) in tuple_struct_pat.elems.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(
                            element_rhs,
                            rhs,
                            crate::types::path::Path::default().tuple_index(ndx),
                        ),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for (ndx, pat) in slice.elems.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(
                            element_rhs,
                            rhs,
                            crate::types::path::Path::default().index(ndx),
                        ),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Type(type_pat) => {
                // We can ignore the type information here because
                // initialize_local needs to be called after bind_pattern.
                self.initialize_local(&type_pat.pat, rhs)
            }
            PatKind::Wild | PatKind::Lit(_) | PatKind::Path(_) => Ok(()),
            _ => Err(self
                .raise_ice(
                    ICE::UnsupportedPatternInInitializeLocal {
                        pat: Box::new(pat.clone()),
                    },
                    pat.id,
                )
                .into()),
        }
    }
    fn stash(&mut self, kernel: &Kernel) -> Result<FuncId> {
        let ndx = self.stash.len().into();
        let object = compile(kernel.clone(), self.mode)?;
        self.stash.insert(ndx, Box::new(object));
        Ok(ndx)
    }
    fn op(&mut self, op: OpCode, node: NodeId) {
        self.ops.push((op, (self.fn_id, node).into()).into());
    }
    fn insert_implicit_return(
        &mut self,
        id: NodeId,
        slot: Slot,
        early_return_name: &'static str,
    ) -> Result<()> {
        let (early_return_flag, _) = self
            .lookup_name(EARLY_RETURN_FLAG_NAME)
            .ok_or_else(|| self.raise_ice(ICE::NoEarlyReturnFlagFound { func: self.fn_id }, id))?;
        let early_return_slot = self.rebind(early_return_name, id)?;
        self.op(
            op_select(
                early_return_slot.to,
                early_return_flag,
                early_return_slot.from,
                slot,
            ),
            id,
        );
        Ok(())
    }
    fn raise_syntax_error(&self, cause: Syntax, id: NodeId) -> Box<RHDLSyntaxError> {
        let source_span = self.spanned_source.span(id);
        Box::new(RHDLSyntaxError {
            cause,
            src: self.spanned_source.source.clone(),
            err_span: source_span.into(),
        })
    }
    fn raise_ice(&self, cause: ICE, loc: NodeId) -> Box<RHDLCompileError> {
        let source_span = self.spanned_source.span(loc);
        Box::new(RHDLCompileError {
            cause,
            src: self.spanned_source.source.clone(),
            err_span: source_span.into(),
        })
    }
    fn get_locals_changed(
        &self,
        node_id: NodeId,
        from: &LocalsMap,
        to: &LocalsMap,
    ) -> Result<BTreeSet<ScopeIndex>> {
        from.iter()
            .filter_map(|(id, slot)| {
                {
                    if let Some(to_slot) = to.get(id) {
                        Ok(if to_slot != slot {
                            Some(id.clone())
                        } else {
                            None
                        })
                    } else {
                        Err(self
                            .raise_ice(
                                ICE::LocalVariableNotFoundInBranchMap { id: id.clone() },
                                node_id,
                            )
                            .into())
                    }
                }
                .transpose()
            })
            .collect()
    }
    fn array(&mut self, id: NodeId, array: &ExprArray) -> Result<Slot> {
        let lhs = self.reg(id);
        let elements = self.expr_list(&array.elems)?;
        self.op(op_array(lhs, elements), id);
        Ok(lhs)
    }
    fn arm(&mut self, target: Slot, lhs: Slot, arm: &Arm) -> Result<CaseArgument> {
        match &arm.kind {
            ArmKind::Wild => {
                self.wrap_expr_in_block(lhs, &arm.body)?;
                Ok(CaseArgument::Wild)
            }
            ArmKind::Constant(constant) => {
                self.wrap_expr_in_block(lhs, &arm.body)?;
                let value = self.lit(arm.id, constant.value.clone());
                let disc = self.reg(arm.id);
                self.op(
                    op_index(
                        disc,
                        value,
                        crate::types::path::Path::default().discriminant(),
                    ),
                    arm.id,
                );
                Ok(CaseArgument::Slot(disc))
            }
            ArmKind::Enum(arm_enum) => {
                self.new_scope();
                // Allocate the local bindings for the match pattern
                self.bind_pattern(&arm_enum.pat)?;
                let discriminant = arm_enum.discriminant.clone();
                let discriminant_slot = self.lit(
                    arm.id,
                    ExprLit::TypedBits(ast_impl::ExprTypedBits {
                        path: Box::new(ast_impl::Path { segments: vec![] }),
                        value: discriminant.clone(),
                        code: String::new(),
                    }),
                );
                let disc_as_i64 = discriminant.as_i64()?;
                let path = crate::types::path::Path::default().payload_by_value(disc_as_i64);
                let payload = self.reg(arm_enum.pat.id);
                self.op(op_index(payload, target, path), arm_enum.pat.id);
                self.initialize_local(&arm_enum.pat, payload)?;
                let result = self.expr(&arm.body)?;
                self.op(op_assign(lhs, result), arm_enum.pat.id);
                self.end_scope();
                Ok(CaseArgument::Slot(discriminant_slot))
            }
        }
    }
    fn assign(&mut self, id: NodeId, assign: &ExprAssign) -> Result<Slot> {
        let rhs = self.expr(&assign.rhs)?;
        let (rebind, path) = self.expr_lhs(&assign.lhs)?;
        self.ty_equate.insert(TypeEquivalence {
            loc: (self.fn_id, id).into(),
            lhs: rebind.to,
            rhs: rebind.from,
        });
        if path.is_empty() {
            self.op(op_assign(rebind.to, rhs), id);
        } else {
            self.op(op_splice(rebind.to, rebind.from, path, rhs), id);
        }
        Ok(Slot::Empty)
    }
    fn assign_binop(&mut self, id: NodeId, bin: &ExprBinary) -> Result<Slot> {
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let (dest, path) = self.expr_lhs(&bin.lhs)?;
        let temp = self.reg(bin.lhs.id);
        let result = Slot::Empty;
        let op = &bin.op;
        if !op.is_self_assign() {
            return Err(self
                .raise_ice(ICE::NonSelfAssignBinop { op: *op }, id)
                .into());
        }
        let alu = binop_to_alu(*op);
        match op {
            BinOp::AddAssign => AluBinary::Add,
            BinOp::SubAssign => AluBinary::Sub,
            BinOp::MulAssign => AluBinary::Mul,
            BinOp::BitXorAssign => AluBinary::BitXor,
            BinOp::BitAndAssign => AluBinary::BitAnd,
            BinOp::BitOrAssign => AluBinary::BitOr,
            BinOp::ShlAssign => AluBinary::Shl,
            BinOp::ShrAssign => AluBinary::Shr,
            _ => {
                return Err(self
                    .raise_ice(ICE::UnexpectedBinopInSelfAssign { op: *op }, id)
                    .into())
            }
        };
        self.ty_equate.insert(TypeEquivalence {
            loc: (self.fn_id, id).into(),
            lhs: dest.to,
            rhs: dest.from,
        });
        self.op(op_binary(alu, temp, lhs, rhs), id);
        if path.is_empty() {
            self.op(op_assign(dest.to, temp), id);
        } else {
            self.op(op_splice(dest.to, dest.from, path, temp), id);
        }
        Ok(result)
    }
    fn block(&mut self, block_result: Slot, block: &Block) -> Result<()> {
        let statement_count = block.stmts.len();
        self.new_scope();
        if block.stmts.is_empty() {
            self.op(op_assign(block_result, Slot::Empty), block.id);
        } else {
            for (ndx, statement) in block.stmts.iter().enumerate() {
                let is_last = ndx == statement_count - 1;
                let result = self.stmt(statement)?;
                if is_last && (block_result != result) {
                    self.op(op_assign(block_result, result), statement.id);
                }
            }
        }
        self.end_scope();
        Ok(())
    }
    fn binop(&mut self, id: NodeId, bin: &ExprBinary) -> Result<Slot> {
        let op = &bin.op;
        let self_assign = matches!(
            op,
            BinOp::AddAssign
                | BinOp::SubAssign
                | BinOp::MulAssign
                | BinOp::BitXorAssign
                | BinOp::BitAndAssign
                | BinOp::ShlAssign
                | BinOp::BitOrAssign
                | BinOp::ShrAssign
        );
        if self_assign {
            return self.assign_binop(id, bin);
        }
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let result = self.reg(id);
        let alu = binop_to_alu(*op);
        assert!(!self_assign);
        self.op(op_binary(alu, result, lhs, rhs), id);
        Ok(result)
    }
    fn type_pattern(&mut self, pattern: &Pat, kind: &Kind) -> Result<()> {
        eprintln!("Type pattern {:?} {:?}", pattern, kind);
        match &pattern.kind {
            PatKind::Ident(ident) => {
                let (slot, _) = self.lookup_name(ident.name).ok_or_else(|| {
                    self.raise_ice(
                        ICE::NoLocalVariableFoundForTypedPattern {
                            pat: Box::new(pattern.clone()),
                        },
                        pattern.id,
                    )
                })?;
                eprintln!("Binding {:?} to {:?} via {:?}", ident, kind, slot);
                self.bind_slot_to_type(slot, kind);
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                for (ndx, element) in tuple.elements.iter().enumerate() {
                    self.type_pattern(element, &kind.get_tuple_kind(ndx)?)?;
                }
                Ok(())
            }
            PatKind::Struct(struct_pat) => {
                for field in &struct_pat.fields {
                    self.type_pattern(&field.pat, &kind.get_field_kind(&field.member)?)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(tuple_struct) => {
                for (ndx, field) in tuple_struct.elems.iter().enumerate() {
                    self.type_pattern(field, &kind.get_field_kind(&Member::Unnamed(ndx as u32))?)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for element in &slice.elems {
                    self.type_pattern(element, &kind.get_base_kind()?)?;
                }
                Ok(())
            }
            PatKind::Paren(paren) => self.type_pattern(&paren.pat, kind),
            PatKind::Wild => Ok(()),
            _ => Err(self
                .raise_ice(
                    ICE::UnsupportedPatternInTypePattern {
                        pat: Box::new(pattern.clone()),
                    },
                    pattern.id,
                )
                .into()),
        }
    }
    fn bind_pattern(&mut self, pattern: &Pat) -> Result<()> {
        match &pattern.kind {
            PatKind::Ident(ident) => {
                self.bind(ident.name, pattern.id);
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                for element in &tuple.elements {
                    self.bind_pattern(element)?;
                }
                Ok(())
            }
            PatKind::Type(type_pat) => {
                self.bind_pattern(&type_pat.pat)?;
                self.type_pattern(&type_pat.pat, &type_pat.kind)
            }
            PatKind::Struct(struct_pat) => {
                for field in &struct_pat.fields {
                    self.bind_pattern(&field.pat)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(tuple_struct) => {
                for field in &tuple_struct.elems {
                    self.bind_pattern(field)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for element in &slice.elems {
                    self.bind_pattern(element)?;
                }
                Ok(())
            }
            PatKind::Paren(paren) => self.bind_pattern(&paren.pat),
            PatKind::Wild => Ok(()),
            _ => Err(self
                .raise_ice(
                    ICE::UnsupportedPatternInBindPattern {
                        pat: Box::new(pattern.clone()),
                    },
                    pattern.id,
                )
                .into()),
        }
    }
    // Handle the case of a bits or signed call with inferred length
    fn bits(&mut self, id: NodeId, bits: &ExprBits) -> Result<Slot> {
        let lhs = self.reg(id);
        let arg = self.expr(&bits.arg)?;
        match bits.kind {
            BitsKind::Unsigned => self.op(op_as_bits_inferred(lhs, arg), id),
            BitsKind::Signed => self.op(op_as_signed_inferred(lhs, arg), id),
        };
        Ok(lhs)
    }
    fn call(&mut self, id: NodeId, call: &ExprCall) -> Result<Slot> {
        let lhs = self.reg(id);
        let args = self.expr_list(&call.args)?;
        let Some(code) = &call.code else {
            return Err(self
                .raise_ice(ICE::CallToKernelWithNoCode { call: call.clone() }, id)
                .into());
        };
        // inline calls to bits and signed
        match code {
            KernelFnKind::BitConstructor(len) => self.op(op_as_bits(lhs, args[0], *len), id),
            KernelFnKind::SignedBitsConstructor(len) => {
                self.bind_slot_to_type(args[0], &Kind::make_signed(128));
                self.op(op_as_signed(lhs, args[0], *len), id)
            }
            KernelFnKind::TupleStructConstructor(tb) => {
                let fields = args
                    .iter()
                    .enumerate()
                    .map(|(ndx, x)| rhif::spec::FieldValue {
                        value: *x,
                        member: rhif::spec::Member::Unnamed(ndx as u32),
                    })
                    .collect();
                self.op(op_struct(lhs, fields, None, tb.clone()), id);
            }
            KernelFnKind::EnumTupleStructConstructor(template) => {
                let fields = args
                    .iter()
                    .enumerate()
                    .map(|(ndx, x)| rhif::spec::FieldValue {
                        value: *x,
                        member: rhif::spec::Member::Unnamed(ndx as u32),
                    })
                    .collect();
                self.op(op_enum(lhs, fields, template.clone()), id);
            }
            KernelFnKind::Kernel(kernel) => {
                let func = self.stash(kernel)?;
                self.op(op_exec(lhs, func, args), id);
            }
            KernelFnKind::SignalConstructor(color) => {
                self.op(op_retime(lhs, args[0], *color), id);
            }
            KernelFnKind::BitCast(to) => {
                self.op(op_as_bits(lhs, args[0], *to), id);
            }
            KernelFnKind::SignedCast(to) => {
                self.op(op_as_signed(lhs, args[0], *to), id);
            }
            KernelFnKind::Wrap(wrap_op) => {
                match wrap_op {
                    WrapOp::None => self.op(op_wrap(lhs, Slot::Empty, *wrap_op), id),
                    _ => self.op(op_wrap(lhs, args[0], *wrap_op), id),
                };
            }
        }
        Ok(lhs)
    }

    fn expr_list(&mut self, exprs: &[Box<Expr>]) -> Result<Vec<Slot>> {
        exprs.iter().map(|expr| self.expr(expr)).collect()
    }
    fn expr(&mut self, expr: &Expr) -> Result<Slot> {
        match &expr.kind {
            ExprKind::Array(array) => self.array(expr.id, array),
            ExprKind::Binary(bin) => self.binop(expr.id, bin),
            ExprKind::Block(block) => {
                let block_result = self.reg(expr.id);
                self.block(block_result, &block.block)?;
                Ok(block_result)
            }
            ExprKind::If(if_expr) => self.if_expr(expr.id, if_expr),
            ExprKind::Lit(lit) => Ok(self.lit(expr.id, lit.clone())),
            ExprKind::Field(field) => self.field(expr.id, field),
            ExprKind::Group(group) => self.expr(&group.expr),
            ExprKind::Index(index) => self.index(expr.id, index),
            ExprKind::Paren(paren) => self.expr(&paren.expr),
            ExprKind::Path(path) => self.path(expr.id, path),
            ExprKind::Struct(_struct) => self.struct_expr(expr.id, _struct),
            ExprKind::Tuple(tuple) => self.tuple(expr.id, tuple),
            ExprKind::Unary(unary) => self.unop(expr.id, unary),
            ExprKind::Match(_match) => self.match_expr(expr.id, _match),
            ExprKind::Ret(_return) => self.return_expr(expr.id, _return),
            ExprKind::ForLoop(for_loop) => self.for_loop(for_loop),
            ExprKind::Assign(assign) => self.assign(expr.id, assign),
            ExprKind::Range(_) => Err(self
                .raise_syntax_error(Syntax::RangesInForLoopsOnly, expr.id)
                .into()),
            ExprKind::Let(_) => Err(self
                .raise_syntax_error(Syntax::FallibleLetExpr, expr.id)
                .into()),
            ExprKind::Repeat(repeat) => self.repeat(expr.id, repeat),
            ExprKind::Call(call) => self.call(expr.id, call),
            ExprKind::MethodCall(method) => self.method_call(expr.id, method),
            ExprKind::Type(_) => Ok(Slot::Empty),
            ExprKind::Bits(bits) => self.bits(expr.id, bits),
            ExprKind::Try(tri) => self.try_expr(expr.id, tri),
        }
    }
    // We need three components
    //  - the original variable that holds the LHS
    //  - the path to change (if any)
    //  - the new place to write the value
    // So, for example, if we have
    //   a[n] = b
    // Then we need to know:
    //  - The original binding of `a`
    //  - The path corresponding to `[n]`
    //  - The place to store the result of splicing `a[n]<-b` in a
    // new binding of the name `a`.
    fn expr_lhs(&mut self, expr: &Expr) -> Result<(Rebind, crate::types::path::Path)> {
        match &expr.kind {
            ExprKind::Path(path) => {
                let name = path_as_ident(&path.path).ok_or_else(|| {
                    self.raise_ice(
                        ICE::UnexpectedComplexPath {
                            path: path.to_owned(),
                        },
                        expr.id,
                    )
                })?;
                let rebind = self.rebind(name, expr.id)?;
                Ok((rebind, crate::types::path::Path::default()))
            }
            ExprKind::Field(field) => {
                let (rebind, path) = self.expr_lhs(&field.expr)?;
                let field = field.member.clone().into();
                Ok((rebind, path.join(&field)))
            }
            ExprKind::Index(index) => {
                let (rebind, path) = self.expr_lhs(&index.expr)?;
                let index = self.expr(&index.index)?;
                if index.is_literal() {
                    let ndx = self.slot_to_index(index, expr.id)?;
                    Ok((rebind, path.index(ndx)))
                } else {
                    Ok((rebind, path.dynamic(index)))
                }
            }
            _ => todo!("expr_lhs {:?}", expr),
        }
    }
    fn field(&mut self, id: NodeId, field: &ExprField) -> Result<Slot> {
        let lhs = self.reg(id);
        let arg = self.expr(&field.expr)?;
        let path = field.member.clone().into();
        eprintln!("field path {:?}", path);
        self.op(op_index(lhs, arg, path), id);
        Ok(lhs)
    }
    fn field_value(&mut self, element: &FieldValue) -> Result<rhif::spec::FieldValue> {
        let value = self.expr(&element.value)?;
        Ok(rhif::spec::FieldValue {
            member: element.member.clone(),
            value,
        })
    }
    fn for_loop(&mut self, for_loop: &ExprForLoop) -> Result<Slot> {
        self.new_scope();
        let PatKind::Ident(loop_var) = &for_loop.pat.kind else {
            return Err(self
                .raise_syntax_error(Syntax::ForLoopNonIdentPattern, for_loop.pat.id)
                .into());
        };
        self.bind_pattern(&for_loop.pat)?;
        let ExprKind::Range(range) = &for_loop.expr.kind else {
            return Err(self
                .raise_syntax_error(Syntax::ForLoopNonRangeExpr, for_loop.expr.id)
                .into());
        };
        let Some(start) = &range.start else {
            return Err(self
                .raise_syntax_error(Syntax::ForLoopNoStartValue, for_loop.expr.id)
                .into());
        };
        let Some(end) = &range.end else {
            return Err(self
                .raise_syntax_error(Syntax::ForLoopNoEndValue, for_loop.expr.id)
                .into());
        };
        let Expr {
            id: _,
            kind: ExprKind::Lit(start_expr),
        } = start.as_ref()
        else {
            return Err(self
                .raise_syntax_error(Syntax::ForLoopNonIntegerStartValue, start.id)
                .into());
        };
        let Expr {
            id: _,
            kind: ExprKind::Lit(end_expr),
        } = end.as_ref()
        else {
            return Err(self
                .raise_syntax_error(Syntax::ForLoopNonIntegerEndValue, end.id)
                .into());
        };
        let start_lit = coerce_literal_to_i32(start_expr)?;
        let end_lit = coerce_literal_to_i32(end_expr)?;
        for ndx in start_lit..end_lit {
            let value = self.literal_int(for_loop.pat.id, ndx);
            self.rebind(loop_var.name, for_loop.pat.id)?;
            self.initialize_local(&for_loop.pat, value)?;
            self.block(Slot::Empty, &for_loop.body)?;
        }
        self.end_scope();
        Ok(Slot::Empty)
    }
    fn if_expr(&mut self, id: NodeId, if_expr: &ExprIf) -> Result<Slot> {
        let op_result = self.reg(id);
        let then_result = self.reg(if_expr.then_branch.id);
        let else_id = if_expr.else_branch.as_ref().map(|x| x.id).unwrap_or(id);
        let else_result = self.reg(else_id);
        let cond = self.expr(&if_expr.cond)?;
        let locals_prior_to_branch = self.locals();
        eprintln!("Locals prior to branch {:?}", locals_prior_to_branch);
        self.block(then_result, &if_expr.then_branch)?;
        let locals_after_then_branch = self.locals().clone();
        eprintln!("Locals after then branch {:?}", locals_after_then_branch);
        self.set_locals(&locals_prior_to_branch, id)?;
        if let Some(expr) = if_expr.else_branch.as_ref() {
            self.wrap_expr_in_block(else_result, expr)?;
        } else {
            self.op(op_assign(else_result, Slot::Empty), id);
        }
        let locals_after_else_branch = self.locals();
        self.set_locals(&locals_prior_to_branch, id)?;
        // Linearize the if statement.
        // TODO - For now, inline this logic, but ultimately, we want
        // to be able to generalize to remove the `case` op.
        let mut rebound_locals =
            self.get_locals_changed(id, &locals_prior_to_branch, &locals_after_then_branch)?;
        rebound_locals.extend(self.get_locals_changed(
            id,
            &locals_prior_to_branch,
            &locals_after_else_branch,
        )?);
        // Next, for each local variable in rebindings, we need a new
        // binding for that variable in the current scope.
        let post_branch_bindings: BTreeMap<ScopeIndex, Rebind> = rebound_locals
            .iter()
            .map(|x| self.rebind(x.name, id).map(|r| (x.clone(), r)))
            .collect::<Result<_>>()?;
        eprintln!("post_branch bindings set {:?}", post_branch_bindings);
        for (var, rebind) in &post_branch_bindings {
            let then_binding = *locals_after_then_branch.get(var).ok_or_else(|| {
                self.raise_ice(
                    ICE::MissingLocalVariableForBindingInThenBranch { var: var.clone() },
                    id,
                )
            })?;
            let else_binding = *locals_after_else_branch.get(var).ok_or_else(|| {
                self.raise_ice(
                    ICE::MissingLocalVariableForBindingInElseBranch { var: var.clone() },
                    id,
                )
            })?;
            let new_binding = rebind.to;
            self.op(op_select(new_binding, cond, then_binding, else_binding), id);
        }
        self.op(op_select(op_result, cond, then_result, else_result), id);
        Ok(op_result)
    }
    fn index(&mut self, id: NodeId, index: &ExprIndex) -> Result<Slot> {
        let lhs = self.reg(id);
        let arg = self.expr(&index.expr)?;
        let index = self.expr(&index.index)?;
        self.op(
            op_index(lhs, arg, crate::types::path::Path::default().dynamic(index)),
            id,
        );
        Ok(lhs)
    }
    fn local(&mut self, local: &Local) -> Result<()> {
        let mut rhs = None;
        if let Some(init) = &local.init {
            rhs = Some(self.expr(init)?);
        }
        self.bind_pattern(&local.pat)?;
        if let Some(rhs) = rhs {
            self.initialize_local(&local.pat, rhs)?;
        }
        Ok(())
    }
    fn match_expr(&mut self, id: NodeId, match_expr: &ExprMatch) -> Result<Slot> {
        let lhs = self.reg(id);
        let target = self.expr(&match_expr.expr)?;
        let discriminant = self.reg(id);
        self.op(
            op_index(
                discriminant,
                target,
                crate::types::path::Path::default().discriminant(),
            ),
            id,
        );
        // Need to handle local rebindings in the bodies of the arms.
        let locals_prior_to_match = self.locals().clone();
        let mut arguments = vec![];
        let mut arm_locals = vec![];
        let mut arm_lhs = vec![];
        for arm in &match_expr.arms {
            self.set_locals(&locals_prior_to_match, id)?;
            let lhs = self.reg(id);
            let disc = self.arm(target, lhs, arm)?;
            arm_lhs.push(lhs);
            arguments.push(disc);
            arm_locals.push(self.locals().clone());
        }
        self.set_locals(&locals_prior_to_match, id)?;
        let mut rebound_locals = BTreeSet::new();
        for branch_locals in &arm_locals {
            let branch_rebindings = self.get_locals_changed(id, &self.locals(), branch_locals)?;
            rebound_locals.extend(branch_rebindings);
        }
        // Next, for each local variable in rebindings, we need a new
        // binding for that variable in the current scope.
        let post_branch_bindings: BTreeMap<ScopeIndex, Rebind> = rebound_locals
            .iter()
            .map(|x| self.rebind(x.name, id).map(|r| (x.clone(), r)))
            .collect::<Result<_>>()?;
        for (var, rebind) in &post_branch_bindings {
            let arm_bindings = arm_locals
                .iter()
                .map(|x| {
                    x.get(var).ok_or_else(|| {
                        self.raise_ice(
                            ICE::MissingLocalVariableForBindingInMatchArm { var: var.clone() },
                            id,
                        )
                        .into()
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let cases = arguments
                .iter()
                .cloned()
                .zip(arm_bindings.into_iter().cloned())
                .collect::<Vec<_>>();
            let new_binding = rebind.to;
            self.op(op_case(new_binding, discriminant, cases), id);
        }
        let match_expr_table = arguments.iter().cloned().zip(arm_lhs).collect::<Vec<_>>();
        self.op(op_case(lhs, discriminant, match_expr_table), id);
        Ok(lhs)
    }
    fn resize(&mut self, id: NodeId, cast: &ExprMethodCall) -> Result<Slot> {
        // First the inferred case...
        let lhs = self.reg(id);
        let arg = self.expr(&cast.receiver)?;
        if let Some(len) = cast.turbo {
            self.op(op_resize(lhs, arg, len), id);
        } else {
            self.op(op_resize_inferred(lhs, arg), id);
        }
        Ok(lhs)
    }
    fn method_call(&mut self, id: NodeId, method_call: &ExprMethodCall) -> Result<Slot> {
        // Special case the `cast` method calls
        if method_call.method == "resize" {
            return self.resize(id, method_call);
        }
        let op = match method_call.method {
            "any" => AluUnary::Any,
            "all" => AluUnary::All,
            "xor" => AluUnary::Xor,
            "as_unsigned" => AluUnary::Unsigned,
            "as_signed" => AluUnary::Signed,
            // The `val` method is a special case used to strip the clocking context
            // from a signal.
            "val" => AluUnary::Val,
            _ => {
                return Err(self
                    .raise_syntax_error(Syntax::UnsupportedMethodCall, id)
                    .into())
            }
        };
        let lhs = self.reg(id);
        let arg = self.expr(&method_call.receiver)?;
        self.op(op_unary(op, lhs, arg), id);
        Ok(lhs)
    }
    fn path(&mut self, id: NodeId, path: &ExprPath) -> Result<Slot> {
        let lhs = self.reg(id);
        if path.path.segments.len() == 1 && path.path.segments[0].arguments.is_empty() {
            let name = path.path.segments[0].ident;
            let rhs = self.lookup_name(name).map(|x| x.0).ok_or_else(|| {
                self.raise_ice(
                    ICE::NameNotFoundInPath {
                        name: name.into(),
                        path: path.clone(),
                    },
                    id,
                )
            })?;
            self.op(op_assign(lhs, rhs), id);
            self.ty_equate.insert(TypeEquivalence {
                loc: (self.fn_id, id).into(),
                lhs,
                rhs,
            });
            return Ok(lhs);
        }
        Err(self
            .raise_syntax_error(Syntax::UnsupportedPathWithArguments, id)
            .into())
    }
    fn repeat(&mut self, id: NodeId, repeat: &ExprRepeat) -> Result<Slot> {
        let lhs = self.reg(id);
        let value = self.expr(&repeat.value)?;
        self.op(op_repeat(lhs, value, repeat.len as _), id);
        Ok(lhs)
    }
    fn try_expr(&mut self, id: NodeId, try_expr: &ExprTry) -> Result<Slot> {
        // The try operation is reduced to the following steps
        // 1.  First, we evaluate the expression that is being tried
        // 2.  Next, we need to check if the expression is an error - here, we
        //     need to check if the discriminant of the expression is equal to
        //     one - by convention, the Option and Result types use a discriminant
        //           of 0 to mean "Bad"
        // 3.  If the result is good, we unwrap the value and return it as the result
        // 4.  Otherwise, we make the early return of the function equal to the
        //     error value.
        // Start with the thing being evaluated.
        let arg = self.expr(&try_expr.expr)?;
        // The unwrap opcode takes a value that is either an option or a result,
        // and then splits it into three pieces:
        //  unwrap(val) -> (good, bad, selector)
        // Where selector indicates which of the two values is valid.  The other
        // value will be initialized to the init value of this type.
        // This is a complicated operation, but we have an unwrapped op code for it.
        let lhs = self.reg(id); // This holds the unwrapped value (which may be invalid)
        let is_good = self.reg(id);
        self.op(
            op_index(
                lhs,
                arg,
                crate::types::path::Path::default()
                    .payload_by_value(1)
                    .tuple_index(0),
            ),
            id,
        );
        self.op(
            op_index(
                is_good,
                arg,
                crate::types::path::Path::default().discriminant(),
            ),
            id,
        );
        // Next, we duplicate the early return logic here, using the is_bad in the selection
        let literal_true = self.literal_bool(id, true);
        let early_return_flag = self.rebind(EARLY_RETURN_FLAG_NAME, id)?;
        let name = self.name;
        let return_slot = self.rebind(name, id)?;
        let early_return_expr = arg;
        // We want to overwrite the return slot if it the flag has not been set and
        // our value is bad, i.e., !early_return_flag.from && is_bad
        // Equivalently, we want to pass the existing return slot through
        // if early_return_flag.from || is_good
        let pass_flag = self.reg(id);
        self.op(
            op_binary(AluBinary::BitOr, pass_flag, early_return_flag.from, is_good),
            id,
        );
        self.op(
            op_select(
                return_slot.to,
                pass_flag,
                return_slot.from,
                early_return_expr,
            ),
            id,
        );
        self.op(
            op_select(
                early_return_flag.to,
                pass_flag,
                early_return_flag.from,
                literal_true,
            ),
            id,
        );
        Ok(lhs)
    }
    fn return_expr(&mut self, id: NodeId, return_expr: &ExprRet) -> Result<Slot> {
        // An early return of the type "return <expr>" is transformed
        // into the following equivalent expression
        // if !__early_return {
        //    __early_return = true;
        //    return_slot = <expr>
        // }
        // Because the function is pure, and has no side effects, the rest
        // of the function can continue as normal.  We do need to make sure
        // that a phi node is inserted at the end of this synthetic `if` statement
        // to build a distributed priority encoder for the `__early_return` flag and
        // for the return slot itself.

        let literal_true = self.literal_bool(id, true);
        let early_return_flag = self.rebind(EARLY_RETURN_FLAG_NAME, id)?;
        let name = self.name;
        let return_slot = self.rebind(name, id)?;
        let early_return_expr = if let Some(return_expr) = &return_expr.expr {
            self.expr(return_expr)?
        } else {
            Slot::Empty
        };
        // Next, we need to code the following:
        //  if early_return_flag.from {
        //     return_slot.to = return_slot.from
        //     early_return_flag.to = early_return_flag.from
        //  } else {
        //     return_slot.to = <expr>
        //     early_return_flag.to = true
        //  }
        // These need to be encoded into 2 select instructions as:
        // return_slot.to = select(early_return_flag.from, return_slot.from, <expr>)
        // early_return_flag.to = select(early_return_flag.from, early_return_flag.from, true)
        self.op(
            op_select(
                return_slot.to,
                early_return_flag.from,
                return_slot.from,
                early_return_expr,
            ),
            id,
        );
        self.op(
            op_select(
                early_return_flag.to,
                early_return_flag.from,
                early_return_flag.from,
                literal_true,
            ),
            id,
        );
        Ok(Slot::Empty)
    }
    fn stmt(&mut self, statement: &Stmt) -> Result<Slot> {
        let statement_text = pretty_print_statement(statement)?;
        self.op(op_comment(statement_text), statement.id);
        match &statement.kind {
            StmtKind::Local(local) => {
                self.local(local)?;
                Ok(Slot::Empty)
            }
            StmtKind::Expr(expr) => self.expr(expr),
            StmtKind::Semi(expr) => {
                self.expr(expr)?;
                Ok(Slot::Empty)
            }
        }
    }
    fn struct_expr(&mut self, id: NodeId, strukt: &ExprStruct) -> Result<Slot> {
        eprintln!("Struct expr {:?}", strukt,);
        let lhs = self.reg(id);
        let fields = strukt
            .fields
            .iter()
            .map(|x| self.field_value(x))
            .collect::<Result<_>>()?;
        let rest = strukt.rest.as_ref().map(|x| self.expr(x)).transpose()?;
        if let Kind::Enum(_enum) = &strukt.template.kind {
            eprintln!("Emitting enum opcode");
            self.op(op_enum(lhs, fields, strukt.template.clone()), id);
        } else {
            eprintln!("Emitting struct opcode");
            self.op(op_struct(lhs, fields, rest, strukt.template.clone()), id);
        }
        Ok(lhs)
    }
    fn tuple(&mut self, id: NodeId, tuple: &ExprTuple) -> Result<Slot> {
        let elements = self.expr_list(&tuple.elements)?;
        let result = self.reg(id);
        self.op(op_tuple(result, elements), id);
        Ok(result)
    }
    fn unop(&mut self, id: NodeId, unary: &ExprUnary) -> Result<Slot> {
        let arg = self.expr(&unary.expr)?;
        let result = self.reg(id);
        let op = unary_op_to_alu(unary.op);
        self.op(op_unary(op, result, arg), id);
        Ok(result)
    }
    fn wrap_expr_in_block(&mut self, block_result: Slot, expr: &Expr) -> Result<()> {
        let result = self.expr(expr)?;
        if block_result != result {
            self.op(op_assign(block_result, result), expr.id);
        }
        Ok(())
    }
}

impl<'a> Visitor for MirContext<'a> {
    fn visit_kernel_fn(&mut self, node: &ast_impl::KernelFn) -> Result<()> {
        self.unpack_arguments(&node.inputs, node.id)?;
        let block_result = self.reg(node.id);
        if block_result.is_empty() {
            return Err(self
                .raise_syntax_error(Syntax::EmptyReturnForFunction, node.id)
                .into());
        }
        self.return_slot = block_result;
        // We create 2 bindings (local vars) inside the function
        //   - the early return flag - a flag of type bool that we initialize to false
        //   - the return slot - a register of type ret<fn>, that we initialize to the default
        // This is equivalent to injecting
        //   let mut __early$exit = false;
        //   let mut fn_name = Default::default();
        // at the beginning of the function body.
        // Each "return" statement must then be replaced with
        //    return x --> if !__early$exit { __early$exit = true; fn_name = x; }
        // The return slot is then used to return the value of the function.
        self.bind(EARLY_RETURN_FLAG_NAME, node.id);
        self.bind(node.name, node.id);
        self.name.clone_from(&node.name);
        // Initialize the early exit flag in the main block
        let init_early_exit_op = op_assign(
            self.lookup_name(EARLY_RETURN_FLAG_NAME).unwrap().0,
            self.literal_bool(node.id, false),
        );
        // Initialize the return slot in the main block
        let init_return_slot = op_assign(
            self.lookup_name(node.name).unwrap().0,
            self.literal_tb(node.id, &node.ret.place_holder()),
        );
        // Initialize the arguments in the main block
        for (arg, slot) in node.inputs.iter().zip(self.arguments.clone().iter()) {
            self.bind_pattern(arg)?;
            self.initialize_local(arg, *slot)?;
        }
        self.block(block_result, &node.body)?;
        self.ops
            .insert(0, (init_early_exit_op, (self.fn_id, node.id).into()).into());
        self.ops
            .insert(1, (init_return_slot, (self.fn_id, node.id).into()).into());
        self.insert_implicit_return(node.body.id, block_result, node.name)?;
        self.return_slot = self
            .lookup_name(node.name)
            .ok_or_else(|| {
                self.raise_ice(
                    ICE::ReturnSlotNotFound {
                        name: node.name.to_owned(),
                    },
                    node.id,
                )
            })?
            .0;
        self.fn_id = node.fn_id;
        Ok(())
    }
}

pub fn compile_mir(func: Kernel, mode: CompilationMode) -> Result<Mir> {
    let source = build_spanned_source_for_kernel(func.inner())?;
    for id in 0..func.inner().id.as_u32() {
        let node = NodeId::new(id);
        if !source.span_map.contains_key(&node) {
            eprintln!("AST: {}", ascii::render_ast_to_string(&func)?);
            panic!("Missing span for node {:?}", node);
        }
    }
    let copy_source = source.clone();
    let mut compiler = MirContext::new(&source, mode, func.inner().fn_id);
    compiler.visit_kernel_fn(func.inner())?;
    compiler.bind_slot_to_type(compiler.return_slot, &func.inner().ret);
    let ty: BTreeMap<Slot, Kind> = compiler
        .ty
        .iter()
        .map(|(k, v)| (*k, compiler.kinds[v].to_owned()))
        .collect();
    if let Some(kind) = ty.get(&compiler.return_slot) {
        if kind.is_empty() {
            return Err(compiler
                .raise_syntax_error(Syntax::EmptyReturnForFunction, func.inner().id)
                .into());
        }
    }
    let fn_id = compiler.fn_id;
    let slot_map = compiler
        .reg_source_map
        .into_iter()
        .chain(once((Slot::Empty, func.inner().id)))
        .map(|(slot, node)| (slot, (fn_id, node).into()))
        .collect();
    Ok(Mir {
        symbols: SymbolMap {
            slot_map,
            source_set: (fn_id, copy_source).into(),
            slot_names: compiler.slot_names,
            aliases: Default::default(),
        },
        ops: compiler.ops,
        arguments: compiler.arguments,
        literals: compiler.literals,
        return_slot: compiler.return_slot,
        fn_id: compiler.fn_id,
        ty,
        ty_equate: compiler.ty_equate,
        stash: compiler.stash,
        name: compiler.name.to_string(),
    })
}
