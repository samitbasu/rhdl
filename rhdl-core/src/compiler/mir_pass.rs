// Convert the AST into the mid-level representation, which is RHIF, but with out
// concrete types.

use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::ast::ast_impl::Block;
use crate::ast::ast_impl::ExprArray;
use crate::ast::ast_impl::ExprBinary;
use crate::ast::ast_impl::ExprIf;
use crate::ast::ast_impl::ExprTuple;
use crate::ast::ast_impl::ExprUnary;
use crate::ast::ast_impl::Local;
use crate::ast::ast_impl::Pat;
use crate::ast::ast_impl::PatKind;
use crate::ast::ast_impl::Stmt;
use crate::ast::ast_impl::StmtKind;
use crate::ast_builder::BinOp;
use crate::ast_builder::UnOp;
use crate::rhif::rhif_builder::op_array;
use crate::rhif::rhif_builder::op_assign;
use crate::rhif::rhif_builder::op_binary;
use crate::rhif::rhif_builder::op_comment;
use crate::rhif::rhif_builder::op_index;
use crate::rhif::rhif_builder::op_select;
use crate::rhif::rhif_builder::op_splice;
use crate::rhif::rhif_builder::op_tuple;
use crate::rhif::rhif_builder::op_unary;
use crate::rhif::spec::AluBinary;
use crate::rhif::spec::AluUnary;
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

type LocalsMap = BTreeMap<NodeId, Slot>;
pub struct MirContext {
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
        self.reg_count += 1;
        reg
    }
    fn resolve_local(&self, id: NodeId) -> Result<Slot> {
        self.locals
            .get(&id)
            .copied()
            .ok_or(anyhow!("ICE - unbound local {}", id))
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
                    self.op(op_index(element_rhs, rhs, path), field.pat.id)?;
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

    fn op(&mut self, op: OpCode, node: NodeId) {
        self.ops.push(OpCodeWithSource { op, source: node });
    }
    fn array(&mut self, id: NodeId, array: &ExprArray) -> Result<Slot> {
        let lhs = self.reg(id);
        let elements = self.expr_list(&array.elems)?;
        self.op(op_array(lhs, elements), id);
        Ok(lhs)
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
        for (ndx, statement) in block.stmts.iter().enumerate() {
            let is_last = ndx == statement_count - 1;
            let result = self.stmt(statement)?;
            if is_last && (block_result != result) {
                self.op(op_assign(block_result, result), statement.id);
            }
        }
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
        ExprKind::Lit(lit) => {
            let ndx = self.literals.len();
            let ty = self.ty(expr.id)?;
            self.literals.push(lit.clone());
            self.ty.insert(Slot::Literal(ndx), ty);
            self.context.insert(Slot::Literal(ndx), expr.id);
            Ok(Slot::Literal(ndx))
        }
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
        let post_branch_bindings: BTreeMap<TypeId, Rebind> = rebound_locals
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
    fn local(&mut self, local: &Local) -> Result<()> {
        self.bind_pattern(&local.pat)?;
        if let Some(init) = &local.init {
            let rhs = self.expr(init)?;
            self.initialize_local(&local.pat, rhs)?;
        }
        Ok(())
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
    fn tuple(&mut self, id: NodeId, tuple: &ExprTuple) -> Result<Slot> {
        let elements = self.expr_list(&tuple.elements)?;
        let result = self.reg(id);
        self.op(op_tuple(result, elements), id);
        Ok(result)
    }
    fn unop(&mut self, id: NodeId, unary: ExprUnary) -> Result<Slot> {
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
