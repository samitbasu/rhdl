// Convert the AST into the mid-level representation, which is RHIF, but with out
// concrete types.

use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use crate::ast::ast_impl;
use crate::ast::ast_impl::Block;
use crate::ast::ast_impl::ExprArray;
use crate::ast::ast_impl::ExprAssign;
use crate::ast::ast_impl::ExprBinary;
use crate::ast::ast_impl::ExprCall;
use crate::ast::ast_impl::ExprField;
use crate::ast::ast_impl::ExprForLoop;
use crate::ast::ast_impl::ExprIf;
use crate::ast::ast_impl::ExprIndex;
use crate::ast::ast_impl::ExprMethodCall;
use crate::ast::ast_impl::ExprRepeat;
use crate::ast::ast_impl::ExprRet;
use crate::ast::ast_impl::ExprStruct;
use crate::ast::ast_impl::ExprTuple;
use crate::ast::ast_impl::ExprUnary;
use crate::ast::ast_impl::FieldValue;
use crate::ast::ast_impl::Local;
use crate::ast::ast_impl::Pat;
use crate::ast::ast_impl::PatKind;
use crate::ast::ast_impl::Stmt;
use crate::ast::ast_impl::StmtKind;
use crate::ast_builder::BinOp;
use crate::ast_builder::UnOp;
use crate::rhif;
use crate::rhif::rhif_builder::op_array;
use crate::rhif::rhif_builder::op_as_bits;
use crate::rhif::rhif_builder::op_as_signed;
use crate::rhif::rhif_builder::op_assign;
use crate::rhif::rhif_builder::op_binary;
use crate::rhif::rhif_builder::op_comment;
use crate::rhif::rhif_builder::op_enum;
use crate::rhif::rhif_builder::op_exec;
use crate::rhif::rhif_builder::op_index;
use crate::rhif::rhif_builder::op_repeat;
use crate::rhif::rhif_builder::op_select;
use crate::rhif::rhif_builder::op_splice;
use crate::rhif::rhif_builder::op_struct;
use crate::rhif::rhif_builder::op_tuple;
use crate::rhif::rhif_builder::op_unary;
use crate::rhif::spec::AluBinary;
use crate::rhif::spec::AluUnary;
use crate::rhif::spec::ExternalFunctionCode;
use crate::rhif::spec::FuncId;
use crate::KernelFnKind;
use crate::Kind;
use crate::{
    ast::ast_impl::{Expr, ExprKind, ExprLit, FunctionId, NodeId},
    rhif::spec::{ExternalFunction, OpCode, Slot},
};

use super::display_ast::pretty_print_statement;
use super::ty::TypeId;
use super::UnifyContext;

#[derive(Clone, Debug, PartialEq)]
pub struct OpCodeWithSource {
    op: OpCode,
    source: NodeId,
}

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

#[derive(Copy, Clone, Debug, PartialEq)]
struct ScopeId(usize);

const ROOT_SCOPE: ScopeId = ScopeId(0);

impl Default for ScopeId {
    fn default() -> Self {
        ROOT_SCOPE
    }
}

fn collapse_path(path: &ast_impl::Path) -> String {
    path.segments
        .iter()
        .map(|x| x.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}


#[derive(Debug)]
struct Scope {
    names: HashMap<String, NodeId>,
    children: Vec<ScopeId>,
    parent: ScopeId,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope {{")?;
        for (name, id) in &self.names {
            writeln!(f, "  {} -> {}", name, id)?;
        }
        writeln!(f, "}}")
    }
}

const EARLY_RETURN_FLAG_NODE: NodeId = NodeId::new(!0);

type LocalsMap = BTreeMap<NodeId, Slot>;
pub struct MirContext {
    scopes: Vec<Scope>,
    ops: Vec<OpCodeWithSource>,
    reg_count: usize,
    literals: BTreeMap<Slot, ExprLit>,
    context: BTreeMap<Slot, NodeId>,
    locals: LocalsMap,
    ty: BTreeMap<Slot, Kind>,
    stash: Vec<ExternalFunction>,
    return_node: NodeId,
    arguments: Vec<Slot>,
    fn_id: FunctionId,
    name: String,
    active_scope: ScopeId,
}

impl std::fmt::Display for MirContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Kernel {} ({})", self.name, self.fn_id)?;
        for (lit, expr) in &self.literals {
            writeln!(f, "{} -> {}", lit, expr)?;
        }
        for (ndx, func) in self.stash.iter().enumerate() {
            writeln!(
                f,
                "Function f{} name: {} code: {} signature: {}",
                ndx, func.path, func.code, func.signature
            )?;
        }
        for op in &self.ops {
            writeln!(f, "{}", op.op)?;
        }
        Ok(())
    }
}

impl MirContext {
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
    // Create a local variable binding to the given NodeId.
    fn bind(&mut self, id: NodeId, name: &str) -> Result<()> {
        let reg = self.reg(id);
        eprintln!("Binding {}#{} to {:?}", name, id, reg);
        if self.locals.insert(id, reg).is_some() {
            bail!("ICE - duplicate local binding for {}", id);
        }
        Ok(())
    }
    // Rebind a local variable to a new slot.
    fn rebind(&mut self, id: NodeId) -> Result<Rebind> {
        let reg = self.reg(id);
        let Some(prev) = self.locals.get(&id).copied() else {
            bail!("ICE - rebind of unbound variable {}", id);
        };
        eprintln!("Rebinding {}#{} to {:?}", id, prev, reg);
        self.locals.insert(id, reg);
        Ok(Rebind {
            from: prev,
            to: reg,
        })
    }
    fn reg(&mut self, id: NodeId) -> Slot {
        let reg = Slot::Register(self.reg_count);
        self.context.insert(reg, id);
        self.reg_count += 1;
        reg
    }
    fn lit(&mut self, id: NodeId, lit: ExprLit) -> Slot {
        let ndx = self.literals.len();
        let slot = Slot::Literal(ndx);
        self.literals.insert(slot, lit);
        self.context.insert(slot, id);
        slot
    }
    fn literal_int(&mut self, id: NodeId, val: i32) -> Slot {
        self.lit(id, ExprLit::Int(val.to_string()))
    }
    fn literal_bool(&mut self, id: NodeId, val: bool) -> Slot {
        self.lit(id, ExprLit::Bool(val))
    }
    fn resolve_local(&self, id: NodeId) -> Result<Slot> {
        self.locals
            .get(&id)
            .copied()
            .ok_or(anyhow!("ICE - unbound local {}", id))
    }
    fn lookup_name(&self, path: &str) -> Option<NodeId> {
        let mut scope = self.active_scope;
        loop {
            if let Some(id) = self.scopes[scope.0].names.get(path) {
                return Some(*id);
            }
            if scope == ROOT_SCOPE {
                break;
            }
            scope = self.scopes[scope.0].parent;
        }
        None
    }
    fn slot_to_index(&self, slot: Slot) -> Result<usize> {
        let Some(value) = self.literals.get(&slot) else {
            bail!("ICE - slot_to_index called with non-literal slot {:?}", slot);
        };
        let ndx = match value {
            ExprLit::TypedBits(tb) => tb.value.as_i64()? as usize,
            ExprLit::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            ExprLit::Int(i) => i.parse::<usize>()?,
        };
        Ok(ndx)
    }
    fn initialize_local(&mut self, pat: &Pat, rhs: Slot) -> Result<()> {
        match &pat.kind {
            PatKind::Ident(_ident) => {
                if let Ok(lhs) = self.resolve_local(pat.id) {
                    self.op(op_assign(lhs, rhs), pat.id);
                }
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                for (ndx, pat) in tuple.elements.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(element_rhs, rhs, crate::path::Path::default().index(ndx)),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Struct(struct_pat) => {
                for field in &struct_pat.fields {
                    let element_rhs = self.reg(field.pat.id);
                    let path = field.member.clone().into();
                    self.op(op_index(element_rhs, rhs, path), field.pat.id);
                    self.initialize_local(&field.pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(tuple_struct_pat) => {
                for (ndx, pat) in tuple_struct_pat.elems.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(element_rhs, rhs, crate::path::Path::default().index(ndx)),
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
                        op_index(element_rhs, rhs, crate::path::Path::default().index(ndx)),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Type(type_pat) => todo!("Need to handle Kind information here"), //self.initialize_local(&type_pat.pat, rhs),
            PatKind::Wild | PatKind::Lit(_) | PatKind::Path(_) => Ok(()),
            _ => bail!("Pattern not supported"),
        }
    }
    fn stash(&mut self, func: ExternalFunction) -> Result<FuncId> {
        let ndx = self.stash.len();
        self.stash.push(func);
        Ok(FuncId(ndx))
    }
    fn op(&mut self, op: OpCode, node: NodeId) {
        self.ops.push(OpCodeWithSource { op, source: node });
    }
    fn array(&mut self, id: NodeId, array: &ExprArray) -> Result<Slot> {
        let lhs = self.reg(id);
        let elements = self.expr_list(&array.elems)?;
        self.op(op_array(lhs, elements), id);
        Ok(lhs)
    }
    fn assign(&mut self, id: NodeId, assign: &ExprAssign) -> Result<Slot> {
        let rhs = self.expr(&assign.rhs)?;
        let (rebind, path) = self.expr_lhs(&assign.lhs)?;
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
        ensure!(op.is_self_assign(), "ICE - self_assign_binop {:?}", op);
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
            _ => bail!("ICE - self_assign_binop {:?}", op),
        };
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
        for (ndx, statement) in block.stmts.iter().enumerate() {
            let is_last = ndx == statement_count - 1;
            let result = self.stmt(statement)?;
            if is_last && (block_result != result) {
                self.op(op_assign(block_result, result), statement.id);
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
    fn bind_pattern(&mut self, pattern: &Pat) -> Result<()> {
        match &pattern.kind {
            PatKind::Ident(ident) => self.bind(pattern.id, &ident.name),
            PatKind::Tuple(tuple) => {
                for element in &tuple.elements {
                    self.bind_pattern(element)?;
                }
                Ok(())
            }
            PatKind::Type(type_pat) => todo!("Need to handle Kind information here"), //self.bind_pattern(&type_pat.pat),
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
            _ => bail!("Pattern not supported"),
        }
    }
    fn call(&mut self, id: NodeId, call: &ExprCall) -> Result<Slot> {
        let lhs = self.reg(id);
        let path = collapse_path(&call.path);
        let args = self.expr_list(&call.args)?;
        let Some(code) = &call.code else {
            bail!("No code for call {:?}", call)
        };
        // inline calls to bits and signed
        match code {
            KernelFnKind::BitConstructor(len) => self.op(op_as_bits(lhs, args[0], *len), id),
            KernelFnKind::SignedBitsConstructor(len) => {
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
                let func = self.stash(ExternalFunction {
                    code: ExternalFunctionCode::Kernel(kernel.clone()),
                    path: path.clone(),
                    signature: call.signature.clone(),
                })?;
                self.op(op_exec(lhs, func, args), id);
            }
            KernelFnKind::Extern(code) => {
                let func = self.stash(ExternalFunction {
                    code: ExternalFunctionCode::Extern(code.clone()),
                    path: path.clone(),
                    signature: call.signature.clone(),
                })?;
                self.op(op_exec(lhs, func, args), id);
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
        ExprKind::Path(_path) => self.resolve_parent(expr.id),
        ExprKind::Struct(_struct) => self.struct_expr(expr.id, _struct),
        ExprKind::Tuple(tuple) => self.tuple(expr.id, tuple),
        ExprKind::Unary(unary) => self.unop(expr.id, unary),
        ExprKind::Match(_match) => self.match_expr(expr.id, _match),
        ExprKind::Ret(_return) => self.return_expr(expr.id, _return),
        ExprKind::ForLoop(for_loop) => self.for_loop( for_loop),
        ExprKind::Assign(assign) => self.assign(expr.id, assign),
        ExprKind::Range(_) => bail!("Ranges are only supported in for loops"),
        ExprKind::Let(_) => bail!("Fallible let expressions are not currently supported in rhdl.  Use a match instead"),
        ExprKind::Repeat(repeat) => self.repeat(expr.id, repeat),
        ExprKind::Call(call) => self.call(expr.id, call),
        ExprKind::MethodCall(method) => self.method_call(expr.id, method),
        ExprKind::Type(_) => Ok(Slot::Empty),
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
    fn expr_lhs(&mut self, expr: &Expr) -> Result<(Rebind, crate::path::Path)> {
        match &expr.kind {
            ExprKind::Path(path) => {
                let Some(parent_id) = self.
            }
        }
    }
    fn field(&mut self, id: NodeId, field: &ExprField) -> Result<Slot> {
        let lhs = self.reg(id);
        let arg = self.expr(&field.expr)?;
        let path = field.member.clone().into();
        self.op(op_index(lhs, arg, path), id);
        Ok(lhs)
    }
    fn field_value(&mut self, element: &FieldValue) -> Result<rhif::spec::FieldValue> {
        let value = self.expr(&element.value)?;
        Ok(rhif::spec::FieldValue {
            member: element.member.clone().into(),
            value,
        })
    }
    fn for_loop(&mut self, for_loop: &ExprForLoop) -> Result<Slot> {
        self.new_scope();
        self.bind_pattern(&for_loop.pat)?;
        let index_reg = self.resolve_local(for_loop.pat.id)?;
        let ExprKind::Range(range) = &for_loop.expr.kind else {
            bail!("for loop with non-range expression is not supported");
        };
        let Some(start) = &range.start else {
            bail!("for loop with no start value is not supported");
        };
        let Some(end) = &range.end else {
            bail!("for loop with no end value is not supported");
        };
        let Expr { id: _, kind: ExprKind::Lit(ExprLit::Int(start_lit))} = start.as_ref() else {
            bail!("for loop with non-integer start value is not supported");
        };
        let Expr {id: _, kind: ExprKind::Lit(ExprLit::Int(end_lit))} = end.as_ref() else {
            bail!("for loop with non-integer end value is not supported");
        };
        let start_lit = start_lit.parse::<i32>()?;
        let end_lit = end_lit.parse::<i32>()?;
        for ndx in start_lit..end_lit {
            let value = self.literal_int(for_loop.pat.id, ndx);
            self.rebind(for_loop.pat.id)?;
            self.initialize_local(&for_loop.pat, value)?;
            self.block(Slot::Empty, &for_loop.body)?;
        }
        self.end_scope();
        Ok(Slot::Empty)
    }
    fn if_expr(&mut self, id: NodeId, if_expr: &ExprIf) -> Result<Slot> {
        let op_result = self.reg(id);
        let then_result = self.reg(id);
        let else_result = self.reg(id);
        let cond = self.expr(&if_expr.cond)?;
        let locals_prior_to_branch = self.locals.clone();
        eprintln!("Locals prior to branch {:?}", locals_prior_to_branch);
        self.block(then_result, &if_expr.then_branch)?;
        let locals_after_then_branch = self.locals.clone();
        eprintln!("Locals after then branch {:?}", locals_after_then_branch);
        self.locals = locals_prior_to_branch.clone();
        if let Some(expr) = if_expr.else_branch.as_ref() {
            self.wrap_expr_in_block(else_result, expr)?;
        }
        let locals_after_else_branch = self.locals.clone();
        self.locals = locals_prior_to_branch.clone();
        // Linearize the if statement.
        // TODO - For now, inline this logic, but ultimately, we want
        // to be able to generalize to remove the `case` op.
        let mut rebound_locals =
            get_locals_changed(&locals_prior_to_branch, &locals_after_then_branch)?;
        rebound_locals.extend(get_locals_changed(
            &locals_prior_to_branch,
            &locals_after_else_branch,
        )?);
        // Next, for each local variable in rebindings, we need a new
        // binding for that variable in the current scope.
        let post_branch_bindings: BTreeMap<NodeId, Rebind> = rebound_locals
            .iter()
            .map(|x| self.rebind((*x).into()).map(|r| (*x, r)))
            .collect::<Result<_>>()?;
        eprintln!("post_branch bindings set {:?}", post_branch_bindings);
        for (var, rebind) in &post_branch_bindings {
            let then_binding = *locals_after_then_branch.get(var).ok_or(anyhow!(
                "ICE - no local var found for binding {var:?} in then branch"
            ))?;
            let else_binding = *locals_after_else_branch.get(var).ok_or(anyhow!(
                "ICE - no local var found for binding {var:?} in else branch"
            ))?;
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
        if index.is_literal() {
            let ndx = self.slot_to_index(index)?;
            self.op(op_index(lhs, arg, crate::path::Path::default().index(ndx)), id);
        } else {
            self.op(op_index(lhs, arg, crate::path::Path::default().dynamic(index)), id);
        }
        Ok(lhs)
    }
    fn local(&mut self, local: &Local) -> Result<()> {
        self.bind_pattern(&local.pat)?;
        if let Some(init) = &local.init {
            let rhs = self.expr(init)?;
            self.initialize_local(&local.pat, rhs)?;
        }
        Ok(())
    }
    fn method_call(&mut self, id: NodeId, method_call: &ExprMethodCall) -> Result<Slot> {
        // The `val` method is a special case used to strip the clocking context
        // from a signal.
        if method_call.method.as_str() == "val" {
            let lhs = self.reg(id);
            let arg = self.expr(&method_call.receiver)?;
            self.op(op_assign(lhs, arg), id);
            /*
            self.op(
                op_index(lhs, arg, crate::path::Path::default().signal_value()),
                id,
            );*/
            return Ok(lhs);
        }
        // First handle unary ops only
        let op = match method_call.method.as_str() {
            "any" => AluUnary::Any,
            "all" => AluUnary::All,
            "xor" => AluUnary::Xor,
            "as_unsigned" => AluUnary::Unsigned,
            "as_signed" => AluUnary::Signed,
            _ => bail!("Unsupported method call {:?}", method_call),
        };
        let lhs = self.reg(id);
        let arg = self.expr(&method_call.receiver)?;
        self.op(op_unary(op, lhs, arg), id);
        Ok(lhs)
    }
    fn repeat(&mut self, id: NodeId, repeat: &ExprRepeat) -> Result<Slot> {
        let lhs = self.reg(id);
        let len = self.expr(&repeat.len)?;
        let len = self.slot_to_index(len)?;
        let value = self.expr(&repeat.value)?;
        self.op(op_repeat(lhs, value, len), id);
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
        let early_return_flag = self.rebind(EARLY_RETURN_FLAG_NODE)?;
        let return_slot = self.rebind(self.return_node)?;
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
        let type_context = UnifyContext::default();
        let statement_text = pretty_print_statement(statement, &type_context)?;
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
        eprintln!("Struct expr {:?} template: {}", strukt, strukt.template);
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

fn get_locals_changed(from: &LocalsMap, to: &LocalsMap) -> Result<BTreeSet<NodeId>> {
    from.iter()
        .filter_map(|(id, slot)| {
            {
                if let Some(to_slot) = to.get(id) {
                    Ok(if to_slot != slot { Some(*id) } else { None })
                } else {
                    Err(anyhow!(
                        "ICE - local variable {:?} not found in branch map",
                        id
                    ))
                }
            }
            .transpose()
        })
        .collect()
}
