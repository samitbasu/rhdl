// A visitor pattern for the ast
// To use this, impl Visitor on a data structure, and then pass it to the appropriate top
// level walk_mut_ function.
// This version allows you to mutate the ast as you traverse it.

use crate::ast::*;
use anyhow::Result;

pub trait VisitorMut {
    fn visit_mut_block(&mut self, node: &mut Block) -> Result<()> {
        visit_mut_block(self, node)
    }
    fn visit_mut_stmt(&mut self, node: &mut Stmt) -> Result<()> {
        visit_mut_stmt(self, node)
    }
    fn visit_mut_local(&mut self, node: &mut Local) -> Result<()> {
        visit_mut_local(self, node)
    }
    fn visit_mut_pat(&mut self, node: &mut Pat) -> Result<()> {
        visit_mut_pat(self, node)
    }
    fn visit_mut_expr(&mut self, node: &mut Expr) -> Result<()> {
        visit_mut_expr(self, node)
    }
    fn visit_mut_match_arm(&mut self, node: &mut Arm) -> Result<()> {
        visit_mut_match_arm(self, node)
    }
    fn visit_mut_kernel_fn(&mut self, node: &mut KernelFn) -> Result<()> {
        visit_mut_kernel_fn(self, node)
    }
}

pub fn visit_mut_block<V>(visitor: &mut V, block: &mut Block) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for stmt in &mut block.stmts {
        visitor.visit_mut_stmt(stmt)?;
    }
    Ok(())
}

pub fn visit_mut_stmt<V>(visitor: &mut V, stmt: &mut Stmt) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    match &mut stmt.kind {
        StmtKind::Local(local) => {
            visitor.visit_mut_local(local)?;
        }
        StmtKind::Expr(expr) => {
            visitor.visit_mut_expr(expr)?;
        }
        StmtKind::Semi(expr) => {
            visitor.visit_mut_expr(expr)?;
        }
    }
    Ok(())
}

pub fn visit_mut_local<V>(visitor: &mut V, local: &mut Local) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_pat(&mut local.pat)?;
    if let Some(init) = &mut local.init {
        visitor.visit_mut_expr(init)?;
    }
    Ok(())
}

pub fn visit_mut_pat<V>(visitor: &mut V, pat: &mut Pat) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    match &mut pat.kind {
        PatKind::Tuple(tuple) => {
            for pat in &mut tuple.elements {
                visitor.visit_mut_pat(pat)?;
            }
        }
        PatKind::Slice(slice) => {
            for pat in &mut slice.elems {
                visitor.visit_mut_pat(pat)?;
            }
        }
        PatKind::TupleStruct(tuple_struct) => {
            for pat in &mut tuple_struct.elems {
                visitor.visit_mut_pat(pat)?;
            }
        }
        PatKind::Or(pat_or) => {
            for pat in &mut pat_or.segments {
                visitor.visit_mut_pat(pat)?;
            }
        }
        PatKind::Paren(pat_paren) => {
            visitor.visit_mut_pat(&mut pat_paren.pat)?;
        }
        PatKind::Struct(structure) => {
            for field in &mut structure.fields {
                visitor.visit_mut_pat(&mut field.pat)?;
            }
        }
        PatKind::Type(pat_type) => {
            visitor.visit_mut_pat(&mut pat_type.pat)?;
        }
        PatKind::Match(pat_match) => {
            visitor.visit_mut_pat(&mut pat_match.pat)?;
        }
        _ => {}
    }
    Ok(())
}

pub fn visit_mut_match_arm<V>(visitor: &mut V, arm: &mut Arm) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_pat(&mut arm.pattern)?;
    if let Some(guard) = &mut arm.guard {
        visitor.visit_mut_expr(guard)?;
    }
    visitor.visit_mut_expr(&mut arm.body)?;
    Ok(())
}

pub fn visit_mut_expr<V>(visitor: &mut V, expr: &mut Expr) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    match &mut expr.kind {
        ExprKind::Binary(expr) => {
            visitor.visit_mut_expr(&mut expr.lhs)?;
            visitor.visit_mut_expr(&mut expr.rhs)?;
        }
        ExprKind::Unary(expr) => visitor.visit_mut_expr(&mut expr.expr)?,
        ExprKind::Match(expr) => {
            visitor.visit_mut_expr(&mut expr.expr)?;
            for arm in &mut expr.arms {
                visitor.visit_mut_match_arm(arm)?;
            }
        }
        ExprKind::Ret(expr) => {
            if let Some(expr) = &mut expr.expr {
                visitor.visit_mut_expr(expr)?;
            }
        }
        ExprKind::If(expr) => {
            visitor.visit_mut_expr(&mut expr.cond)?;
            visitor.visit_mut_block(&mut expr.then_branch)?;
            if let Some(else_branch) = &mut expr.else_branch {
                visitor.visit_mut_expr(else_branch)?;
            }
        }
        ExprKind::Index(expr) => {
            visitor.visit_mut_expr(&mut expr.expr)?;
            visitor.visit_mut_expr(&mut expr.index)?;
        }
        ExprKind::Paren(expr) => {
            visitor.visit_mut_expr(&mut expr.expr)?;
        }
        ExprKind::Tuple(expr) => {
            for expr in &mut expr.elements {
                visitor.visit_mut_expr(expr)?;
            }
        }
        ExprKind::ForLoop(expr) => {
            visitor.visit_mut_pat(&mut expr.pat)?;
            visitor.visit_mut_expr(&mut expr.expr)?;
            visitor.visit_mut_block(&mut expr.body)?;
        }
        ExprKind::Assign(expr) => {
            visitor.visit_mut_expr(&mut expr.lhs)?;
            visitor.visit_mut_expr(&mut expr.rhs)?;
        }
        ExprKind::Group(expr) => {
            visitor.visit_mut_expr(&mut expr.expr)?;
        }
        ExprKind::Field(expr) => {
            visitor.visit_mut_expr(&mut expr.expr)?;
        }
        ExprKind::Block(expr) => {
            visitor.visit_mut_block(&mut expr.block)?;
        }
        ExprKind::Array(expr) => {
            for expr in &mut expr.elems {
                visitor.visit_mut_expr(expr)?;
            }
        }
        ExprKind::Range(expr) => {
            if let Some(start) = &mut expr.start {
                visitor.visit_mut_expr(start)?;
            }
            if let Some(end) = &mut expr.end {
                visitor.visit_mut_expr(end)?;
            }
        }
        ExprKind::Let(expr) => {
            visitor.visit_mut_pat(&mut expr.pattern)?;
            visitor.visit_mut_expr(&mut expr.value)?;
        }
        ExprKind::Repeat(expr) => {
            visitor.visit_mut_expr(&mut expr.value)?;
            visitor.visit_mut_expr(&mut expr.len)?;
        }
        ExprKind::Struct(expr) => {
            for field in &mut expr.fields {
                visitor.visit_mut_expr(&mut field.value)?;
            }
            if let Some(rest) = &mut expr.rest {
                visitor.visit_mut_expr(rest)?;
            }
        }
        ExprKind::Call(expr) => {
            for arg in &mut expr.args {
                visitor.visit_mut_expr(arg)?;
            }
        }
        ExprKind::MethodCall(expr) => {
            visitor.visit_mut_expr(&mut expr.receiver)?;
            for arg in &mut expr.args {
                visitor.visit_mut_expr(arg)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn visit_mut_kernel_fn<V>(visitor: &mut V, kernel_fn: &mut KernelFn) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for arg in &mut kernel_fn.inputs {
        visitor.visit_mut_pat(arg)?;
    }

    visitor.visit_mut_block(&mut kernel_fn.body)
}
