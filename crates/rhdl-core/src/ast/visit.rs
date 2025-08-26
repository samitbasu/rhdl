// A visitor pattern for the ast
// To use this, impl Visitor on a data structure, and then pass it to the appropriate top
// level walk_ function.

use crate::rhdl_core::{ast::ast_impl::*, error::RHDLError};

type Result<T> = std::result::Result<T, RHDLError>;

pub trait Visitor {
    fn visit_block(&mut self, node: &Block) -> Result<()> {
        visit_block(self, node)
    }
    fn visit_stmt(&mut self, node: &Stmt) -> Result<()> {
        visit_stmt(self, node)
    }
    fn visit_local(&mut self, node: &Local) -> Result<()> {
        visit_local(self, node)
    }
    fn visit_pat(&mut self, node: &Pat) -> Result<()> {
        visit_pat(self, node)
    }
    fn visit_expr(&mut self, node: &Expr) -> Result<()> {
        visit_expr(self, node)
    }
    fn visit_match_arm(&mut self, node: &Arm) -> Result<()> {
        visit_match_arm(self, node)
    }
    fn visit_kernel_fn(&mut self, node: &KernelFn) -> Result<()> {
        visit_kernel_fn(self, node)
    }
}

pub fn visit_block<V>(visitor: &mut V, block: &Block) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt)?;
    }
    Ok(())
}

pub fn visit_stmt<V>(visitor: &mut V, stmt: &Stmt) -> Result<()>
where
    V: Visitor + ?Sized,
{
    match &stmt.kind {
        StmtKind::Local(local) => {
            visitor.visit_local(local)?;
        }
        StmtKind::Expr(expr) => {
            visitor.visit_expr(expr)?;
        }
        StmtKind::Semi(expr) => {
            visitor.visit_expr(expr)?;
        }
    }
    Ok(())
}

pub fn visit_local<V>(visitor: &mut V, local: &Local) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_pat(&local.pat)?;
    if let Some(init) = &local.init {
        visitor.visit_expr(init)?;
    }
    Ok(())
}

pub fn visit_pat<V>(visitor: &mut V, pat: &Pat) -> Result<()>
where
    V: Visitor + ?Sized,
{
    match &pat.kind {
        PatKind::Tuple(tuple) => {
            for pat in &tuple.elements {
                visitor.visit_pat(pat)?;
            }
        }
        PatKind::Slice(slice) => {
            for pat in &slice.elems {
                visitor.visit_pat(pat)?;
            }
        }
        PatKind::TupleStruct(tuple_struct) => {
            for pat in &tuple_struct.elems {
                visitor.visit_pat(pat)?;
            }
        }
        PatKind::Or(pat_or) => {
            for pat in &pat_or.segments {
                visitor.visit_pat(pat)?;
            }
        }
        PatKind::Paren(pat_paren) => {
            visitor.visit_pat(&pat_paren.pat)?;
        }
        PatKind::Struct(structure) => {
            for field in &structure.fields {
                visitor.visit_pat(&field.pat)?;
            }
        }
        PatKind::Type(pat_type) => {
            visitor.visit_pat(&pat_type.pat)?;
        }
        _ => {}
    }
    Ok(())
}

pub fn visit_match_arm<V>(visitor: &mut V, arm: &Arm) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&arm.body)?;
    Ok(())
}

pub fn visit_expr<V>(visitor: &mut V, expr: &Expr) -> Result<()>
where
    V: Visitor + ?Sized,
{
    match &expr.kind {
        ExprKind::Binary(expr) => {
            visitor.visit_expr(&expr.lhs)?;
            visitor.visit_expr(&expr.rhs)?;
        }
        ExprKind::Unary(expr) => visitor.visit_expr(&expr.expr)?,
        ExprKind::Match(expr) => {
            visitor.visit_expr(&expr.expr)?;
            for arm in &expr.arms {
                visitor.visit_match_arm(arm)?;
            }
        }
        ExprKind::Ret(expr) => {
            if let Some(expr) = &expr.expr {
                visitor.visit_expr(expr)?;
            }
        }
        ExprKind::If(expr) => {
            visitor.visit_expr(&expr.cond)?;
            visitor.visit_block(&expr.then_branch)?;
            if let Some(else_branch) = &expr.else_branch {
                visitor.visit_expr(else_branch)?;
            }
        }
        ExprKind::Index(expr) => {
            visitor.visit_expr(&expr.expr)?;
            visitor.visit_expr(&expr.index)?;
        }
        ExprKind::Paren(expr) => {
            visitor.visit_expr(&expr.expr)?;
        }
        ExprKind::Tuple(expr) => {
            for expr in &expr.elements {
                visitor.visit_expr(expr)?;
            }
        }
        ExprKind::ForLoop(expr) => {
            visitor.visit_pat(&expr.pat)?;
            visitor.visit_expr(&expr.expr)?;
            visitor.visit_block(&expr.body)?;
        }
        ExprKind::Assign(expr) => {
            visitor.visit_expr(&expr.lhs)?;
            visitor.visit_expr(&expr.rhs)?;
        }
        ExprKind::Group(expr) => {
            visitor.visit_expr(&expr.expr)?;
        }
        ExprKind::Field(expr) => {
            visitor.visit_expr(&expr.expr)?;
        }
        ExprKind::Block(expr) => {
            visitor.visit_block(&expr.block)?;
        }
        ExprKind::Array(expr) => {
            for expr in &expr.elems {
                visitor.visit_expr(expr)?;
            }
        }
        ExprKind::Range(expr) => {
            if let Some(start) = &expr.start {
                visitor.visit_expr(start)?;
            }
            if let Some(end) = &expr.end {
                visitor.visit_expr(end)?;
            }
        }
        ExprKind::Let(expr) => {
            visitor.visit_pat(&expr.pattern)?;
            visitor.visit_expr(&expr.value)?;
        }
        ExprKind::Repeat(expr) => {
            visitor.visit_expr(&expr.value)?;
        }
        ExprKind::Struct(expr) => {
            for field in &expr.fields {
                visitor.visit_expr(&field.value)?;
            }
            if let Some(rest) = &expr.rest {
                visitor.visit_expr(rest)?;
            }
        }
        ExprKind::Call(expr) => {
            for arg in &expr.args {
                visitor.visit_expr(arg)?;
            }
        }
        ExprKind::MethodCall(expr) => {
            visitor.visit_expr(&expr.receiver)?;
            for arg in &expr.args {
                visitor.visit_expr(arg)?;
            }
        }
        ExprKind::Bits(bits) => {
            visitor.visit_expr(&bits.arg)?;
        }
        _ => {}
    }
    Ok(())
}

pub fn visit_kernel_fn<V>(visitor: &mut V, kernel_fn: &KernelFn) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for arg in &kernel_fn.inputs {
        visitor.visit_pat(arg)?;
    }
    visitor.visit_block(&kernel_fn.body)?;
    Ok(())
}
