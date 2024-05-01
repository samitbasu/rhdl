// Convert the AST into the mid-level representation, which is RHIF, but with out
// concrete types.

use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;
use std::collections::BTreeMap;

use crate::ast::ast_impl::Block;
use crate::ast::ast_impl::ExprArray;
use crate::ast::ast_impl::ExprBinary;
use crate::ast::ast_impl::Stmt;
use crate::ast::ast_impl::StmtKind;
use crate::ast_builder::BinOp;
use crate::rhif::rhif_builder::op_array;
use crate::rhif::rhif_builder::op_assign;
use crate::rhif::rhif_builder::op_binary;
use crate::rhif::rhif_builder::op_comment;
use crate::rhif::rhif_builder::op_splice;
use crate::rhif::spec::AluBinary;
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

pub struct MirContext {
    ops: Vec<OpCodeWithSource>,
    reg_count: usize,
    literals: BTreeMap<Slot, ExprLit>,
    context: BTreeMap<Slot, NodeId>,
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
    fn reg(&mut self, id: NodeId) -> Slot {
        let reg = Slot::Register(self.reg_count);
        self.reg_count += 1;
        reg
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
    fn local(&mut self, local: &Local) -> Result<()> {
        self.bind_pattern(&local.pat)
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
}
