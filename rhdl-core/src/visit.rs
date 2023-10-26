// A visitor pattern for the ast
// To use this, impl Visitor on a data structure, and then pass it to the appropriate top
// level walk_ function.

use crate::ast::*;
use anyhow::Result;

pub trait Visitor {
    fn visit_block(&mut self, _block: &Block) -> Result<()> {
        Ok(())
    }
    fn visit_stmt(&mut self, _stmt: &Stmt) -> Result<()> {
        Ok(())
    }
    fn visit_local(&mut self, _local: &Local) -> Result<()> {
        Ok(())
    }
    fn visit_pat(&mut self, _pat: &Pat) -> Result<()> {
        Ok(())
    }
    fn visit_path_segment(&mut self, _path_segment: &PathSegment) -> Result<()> {
        Ok(())
    }
    fn visit_path(&mut self, _path: &Path) -> Result<()> {
        Ok(())
    }
    fn visit_pat_ident(&mut self, _pat_ident: &PatIdent) -> Result<()> {
        Ok(())
    }
    fn visit_pat_tuple(&mut self, _pat_tuple: &PatTuple) -> Result<()> {
        Ok(())
    }
    fn visit_pat_tuple_struct(&mut self, _pat_tuple_struct: &PatTupleStruct) -> Result<()> {
        Ok(())
    }
    fn visit_pat_lit(&mut self, _pat_lit: &PatLit) -> Result<()> {
        Ok(())
    }
    fn visit_pat_or(&mut self, _pat_or: &PatOr) -> Result<()> {
        Ok(())
    }
    fn visit_pat_paren(&mut self, _pat_paren: &PatParen) -> Result<()> {
        Ok(())
    }
    fn visit_pat_path(&mut self, _pat_path: &PatPath) -> Result<()> {
        Ok(())
    }
    fn visit_pat_struct(&mut self, _pat_struct: &PatStruct) -> Result<()> {
        Ok(())
    }
    fn visit_pat_type(&mut self, _pat_type: &PatType) -> Result<()> {
        Ok(())
    }
    fn visit_pat_wild(&mut self) -> Result<()> {
        Ok(())
    }
    fn visit_expr(&mut self, _expr: &Expr) -> Result<()> {
        Ok(())
    }
    fn visit_expr_binary(&mut self, _expr_binary: &ExprBinary) -> Result<()> {
        Ok(())
    }
    fn visit_expr_unary(&mut self, _expr_unary: &ExprUnary) -> Result<()> {
        Ok(())
    }
    fn visit_expr_match(&mut self, _expr_match: &ExprMatch) -> Result<()> {
        Ok(())
    }
    fn visit_expr_ret(&mut self, _expr_return: &ExprRet) -> Result<()> {
        Ok(())
    }
    fn visit_expr_if(&mut self, _expr_if: &ExprIf) -> Result<()> {
        Ok(())
    }
    fn visit_expr_index(&mut self, _expr_index: &ExprIndex) -> Result<()> {
        Ok(())
    }
    fn visit_expr_paren(&mut self, _expr_paren: &ExprParen) -> Result<()> {
        Ok(())
    }
    fn visit_expr_tuple(&mut self, _expr_tuple: &ExprTuple) -> Result<()> {
        Ok(())
    }
    fn visit_expr_for_loop(&mut self, _expr_for_loop: &ExprForLoop) -> Result<()> {
        Ok(())
    }
    fn visit_expr_assign(&mut self, _expr_assign: &ExprAssign) -> Result<()> {
        Ok(())
    }
    fn visit_expr_group(&mut self, _expr_group: &ExprGroup) -> Result<()> {
        Ok(())
    }
    fn visit_expr_field(&mut self, _expr_field: &ExprField) -> Result<()> {
        Ok(())
    }
    fn visit_expr_block(&mut self, _expr_block: &ExprBlock) -> Result<()> {
        Ok(())
    }
    fn visit_expr_array(&mut self, _expr_array: &ExprArray) -> Result<()> {
        Ok(())
    }
    fn visit_expr_range(&mut self, _expr_range: &ExprRange) -> Result<()> {
        Ok(())
    }
    fn visit_expr_path(&mut self, _expr_path: &ExprPath) -> Result<()> {
        Ok(())
    }
    fn visit_expr_let(&mut self, _expr_let: &ExprLet) -> Result<()> {
        Ok(())
    }
    fn visit_expr_repeat(&mut self, _expr_repeat: &ExprRepeat) -> Result<()> {
        Ok(())
    }
    fn visit_expr_struct(&mut self, _expr_struct: &ExprStruct) -> Result<()> {
        Ok(())
    }
    fn visit_expr_call(&mut self, _expr_call: &ExprCall) -> Result<()> {
        Ok(())
    }
    fn visit_expr_method_call(&mut self, _expr_method_call: &ExprMethodCall) -> Result<()> {
        Ok(())
    }
    fn visit_match_arm(&mut self, _arm: &Arm) -> Result<()> {
        Ok(())
    }
    fn visit_expr_lit(&mut self, _lit: &ExprLit) -> Result<()> {
        Ok(())
    }
    fn visit_field_value(&mut self, _field_value: &FieldValue) -> Result<()> {
        Ok(())
    }
    fn visit_field_pat(&mut self, _field_pat: &FieldPat) -> Result<()> {
        Ok(())
    }
}

pub fn walk_block(visitor: &mut dyn Visitor, block: &Block) -> Result<()> {
    visitor.visit_block(block)?;
    for stmt in &block.stmts {
        walk_stmt(visitor, stmt)?;
    }
    Ok(())
}

pub fn walk_stmt(visitor: &mut dyn Visitor, stmt: &Stmt) -> Result<()> {
    visitor.visit_stmt(stmt)?;
    match &stmt.kind {
        StmtKind::Local(local) => {
            walk_local(visitor, local)?;
        }
        StmtKind::Expr(expr) => {
            walk_expr(visitor, expr)?;
        }
        StmtKind::Semi(expr) => {
            walk_expr(visitor, expr)?;
        }
    }
    Ok(())
}

pub fn walk_local(visitor: &mut dyn Visitor, local: &Local) -> Result<()> {
    visitor.visit_local(local)?;
    walk_pat(visitor, &local.pat)?;
    if let Some(init) = &local.init {
        walk_expr(visitor, init)?;
    }
    Ok(())
}

pub fn walk_pat_ident(visitor: &mut dyn Visitor, pat_ident: &PatIdent) -> Result<()> {
    visitor.visit_pat_ident(pat_ident)?;
    Ok(())
}

pub fn walk_pat_tuple(visitor: &mut dyn Visitor, pat_tuple: &PatTuple) -> Result<()> {
    visitor.visit_pat_tuple(pat_tuple)?;
    for pat in &pat_tuple.elements {
        walk_pat(visitor, pat)?;
    }
    Ok(())
}

pub fn walk_pat_tuple_struct(
    visitor: &mut dyn Visitor,
    pat_tuple_struct: &PatTupleStruct,
) -> Result<()> {
    visitor.visit_pat_tuple_struct(pat_tuple_struct)?;
    walk_path(visitor, &pat_tuple_struct.path)?;
    for pat in &pat_tuple_struct.elems {
        walk_pat(visitor, pat)?;
    }
    Ok(())
}

pub fn walk_pat_lit(visitor: &mut dyn Visitor, pat_lit: &PatLit) -> Result<()> {
    visitor.visit_pat_lit(pat_lit)?;
    Ok(())
}

pub fn walk_pat_or(visitor: &mut dyn Visitor, pat_or: &PatOr) -> Result<()> {
    visitor.visit_pat_or(pat_or)?;
    for pat in &pat_or.segments {
        walk_pat(visitor, pat)?;
    }
    Ok(())
}

pub fn walk_pat_paren(visitor: &mut dyn Visitor, pat_paren: &PatParen) -> Result<()> {
    visitor.visit_pat_paren(pat_paren)?;
    walk_pat(visitor, &pat_paren.pat)?;
    Ok(())
}

pub fn walk_pat_path(visitor: &mut dyn Visitor, pat_path: &PatPath) -> Result<()> {
    visitor.visit_pat_path(pat_path)?;
    walk_path(visitor, &pat_path.path)?;
    Ok(())
}

pub fn walk_pat_struct(visitor: &mut dyn Visitor, pat_struct: &PatStruct) -> Result<()> {
    visitor.visit_pat_struct(pat_struct)?;
    walk_path(visitor, &pat_struct.path)?;
    for field in &pat_struct.fields {
        walk_field_pat(visitor, field)?;
    }
    Ok(())
}

pub fn walk_field_pat(visitor: &mut dyn Visitor, field_pat: &FieldPat) -> Result<()> {
    visitor.visit_field_pat(field_pat)?;
    walk_pat(visitor, &field_pat.pat)?;
    Ok(())
}

pub fn walk_pat_type(visitor: &mut dyn Visitor, pat_type: &PatType) -> Result<()> {
    visitor.visit_pat_type(pat_type)?;
    walk_pat(visitor, &pat_type.pat)?;
    Ok(())
}

pub fn walk_pat_wild(visitor: &mut dyn Visitor) -> Result<()> {
    visitor.visit_pat_wild()?;
    Ok(())
}

pub fn walk_pat(visitor: &mut dyn Visitor, pat: &Pat) -> Result<()> {
    visitor.visit_pat(pat)?;
    match &pat.kind {
        PatKind::Ident(pat) => {
            walk_pat_ident(visitor, pat)?;
        }
        PatKind::Tuple(tuple) => {
            walk_pat_tuple(visitor, tuple)?;
        }
        PatKind::TupleStruct(tuple_struct) => {
            walk_pat_tuple_struct(visitor, tuple_struct)?;
        }
        PatKind::Lit(lit) => {
            walk_pat_lit(visitor, lit)?;
        }
        PatKind::Or(pat_or) => {
            walk_pat_or(visitor, pat_or)?;
        }
        PatKind::Paren(pat_paren) => {
            walk_pat_paren(visitor, pat_paren)?;
        }
        PatKind::Path(path) => {
            walk_pat_path(visitor, path)?;
        }
        PatKind::Struct(structure) => {
            walk_pat_struct(visitor, structure)?;
        }
        PatKind::Type(pat_type) => {
            walk_pat_type(visitor, pat_type)?;
        }
        PatKind::Wild => {
            walk_pat_wild(visitor)?;
        }
    }
    Ok(())
}

pub fn walk_expr_binary(visitor: &mut dyn Visitor, expr_binary: &ExprBinary) -> Result<()> {
    visitor.visit_expr_binary(expr_binary)?;
    walk_expr(visitor, &expr_binary.lhs)?;
    walk_expr(visitor, &expr_binary.rhs)?;
    Ok(())
}

pub fn walk_expr_unary(visitor: &mut dyn Visitor, expr_unary: &ExprUnary) -> Result<()> {
    visitor.visit_expr_unary(expr_unary)?;
    walk_expr(visitor, &expr_unary.expr)?;
    Ok(())
}

pub fn walk_expr_match(visitor: &mut dyn Visitor, expr_match: &ExprMatch) -> Result<()> {
    visitor.visit_expr_match(expr_match)?;
    walk_expr(visitor, &expr_match.expr)?;
    for arm in &expr_match.arms {
        walk_match_arm(visitor, arm)?;
    }
    Ok(())
}

pub fn walk_match_arm(visitor: &mut dyn Visitor, arm: &Arm) -> Result<()> {
    visitor.visit_match_arm(arm)?;
    walk_pat(visitor, &arm.pattern)?;
    if let Some(guard) = &arm.guard {
        walk_expr(visitor, guard)?;
    }
    walk_expr(visitor, &arm.body)?;
    Ok(())
}

pub fn walk_expr_ret(visitor: &mut dyn Visitor, expr_return: &ExprRet) -> Result<()> {
    visitor.visit_expr_ret(expr_return)?;
    if let Some(expr) = &expr_return.expr {
        walk_expr(visitor, expr)?;
    }
    Ok(())
}

pub fn walk_expr_if(visitor: &mut dyn Visitor, expr_if: &ExprIf) -> Result<()> {
    visitor.visit_expr_if(expr_if)?;
    walk_expr(visitor, &expr_if.cond)?;
    walk_block(visitor, &expr_if.then_branch)?;
    if let Some(else_branch) = &expr_if.else_branch {
        walk_expr(visitor, else_branch)?;
    }
    Ok(())
}

pub fn walk_expr_index(visitor: &mut dyn Visitor, expr_index: &ExprIndex) -> Result<()> {
    visitor.visit_expr_index(expr_index)?;
    walk_expr(visitor, &expr_index.expr)?;
    walk_expr(visitor, &expr_index.index)?;
    Ok(())
}

pub fn walk_expr_lit(visitor: &mut dyn Visitor, lit: &ExprLit) -> Result<()> {
    visitor.visit_expr_lit(lit)?;
    Ok(())
}

pub fn walk_expr_paren(visitor: &mut dyn Visitor, expr_paren: &ExprParen) -> Result<()> {
    visitor.visit_expr_paren(expr_paren)?;
    walk_expr(visitor, &expr_paren.expr)?;
    Ok(())
}

pub fn walk_expr_tuple(visitor: &mut dyn Visitor, expr_tuple: &ExprTuple) -> Result<()> {
    visitor.visit_expr_tuple(expr_tuple)?;
    for expr in &expr_tuple.elements {
        walk_expr(visitor, expr)?;
    }
    Ok(())
}

pub fn walk_expr_for_loop(visitor: &mut dyn Visitor, expr_for_loop: &ExprForLoop) -> Result<()> {
    visitor.visit_expr_for_loop(expr_for_loop)?;
    walk_pat(visitor, &expr_for_loop.pat)?;
    walk_expr(visitor, &expr_for_loop.expr)?;
    walk_block(visitor, &expr_for_loop.body)?;
    Ok(())
}

pub fn walk_expr_assign(visitor: &mut dyn Visitor, expr_assign: &ExprAssign) -> Result<()> {
    visitor.visit_expr_assign(expr_assign)?;
    walk_expr(visitor, &expr_assign.lhs)?;
    walk_expr(visitor, &expr_assign.rhs)?;
    Ok(())
}

pub fn walk_expr_group(visitor: &mut dyn Visitor, expr_group: &ExprGroup) -> Result<()> {
    visitor.visit_expr_group(expr_group)?;
    walk_expr(visitor, &expr_group.expr)?;
    Ok(())
}

pub fn walk_expr_field(visitor: &mut dyn Visitor, expr_field: &ExprField) -> Result<()> {
    visitor.visit_expr_field(expr_field)?;
    walk_expr(visitor, &expr_field.expr)?;
    Ok(())
}

pub fn walk_expr_block(visitor: &mut dyn Visitor, expr_block: &ExprBlock) -> Result<()> {
    visitor.visit_expr_block(expr_block)?;
    walk_block(visitor, &expr_block.block)?;
    Ok(())
}

pub fn walk_expr_array(visitor: &mut dyn Visitor, expr_array: &ExprArray) -> Result<()> {
    visitor.visit_expr_array(expr_array)?;
    for expr in &expr_array.elems {
        walk_expr(visitor, expr)?;
    }
    Ok(())
}

pub fn walk_expr_range(visitor: &mut dyn Visitor, expr_range: &ExprRange) -> Result<()> {
    visitor.visit_expr_range(expr_range)?;
    if let Some(start) = &expr_range.start {
        walk_expr(visitor, start)?;
    }
    if let Some(end) = &expr_range.end {
        walk_expr(visitor, end)?;
    }
    Ok(())
}

pub fn walk_expr_path(visitor: &mut dyn Visitor, expr_path: &ExprPath) -> Result<()> {
    visitor.visit_expr_path(expr_path)?;
    walk_path(visitor, &expr_path.path)?;
    Ok(())
}

pub fn walk_expr_let(visitor: &mut dyn Visitor, expr_let: &ExprLet) -> Result<()> {
    visitor.visit_expr_let(expr_let)?;
    walk_pat(visitor, &expr_let.pattern)?;
    walk_expr(visitor, &expr_let.value)?;
    walk_expr(visitor, &expr_let.body)?;
    Ok(())
}

pub fn walk_expr_repeat(visitor: &mut dyn Visitor, expr_repeat: &ExprRepeat) -> Result<()> {
    visitor.visit_expr_repeat(expr_repeat)?;
    walk_expr(visitor, &expr_repeat.value)?;
    walk_expr(visitor, &expr_repeat.len)?;
    Ok(())
}

pub fn walk_expr_struct(visitor: &mut dyn Visitor, expr_struct: &ExprStruct) -> Result<()> {
    visitor.visit_expr_struct(expr_struct)?;
    walk_path(visitor, &expr_struct.path)?;
    for field in &expr_struct.fields {
        walk_field_value(visitor, field)?;
    }
    if let Some(rest) = &expr_struct.rest {
        walk_expr(visitor, rest)?;
    }
    Ok(())
}

pub fn walk_expr_call(visitor: &mut dyn Visitor, expr_call: &ExprCall) -> Result<()> {
    visitor.visit_expr_call(expr_call)?;
    walk_path(visitor, &expr_call.path)?;
    for arg in &expr_call.args {
        walk_expr(visitor, arg)?;
    }
    Ok(())
}

pub fn walk_expr_method_call(
    visitor: &mut dyn Visitor,
    expr_method_call: &ExprMethodCall,
) -> Result<()> {
    visitor.visit_expr_method_call(expr_method_call)?;
    walk_expr(visitor, &expr_method_call.receiver)?;
    for arg in &expr_method_call.args {
        walk_expr(visitor, arg)?;
    }
    Ok(())
}

pub fn walk_expr(visitor: &mut dyn Visitor, expr: &Expr) -> Result<()> {
    visitor.visit_expr(expr)?;
    match &expr.kind {
        ExprKind::Binary(expr) => walk_expr_binary(visitor, expr),
        ExprKind::Unary(expr) => walk_expr_unary(visitor, expr),
        ExprKind::Match(expr) => walk_expr_match(visitor, expr),
        ExprKind::Ret(expr) => walk_expr_ret(visitor, expr),
        ExprKind::If(expr) => walk_expr_if(visitor, expr),
        ExprKind::Index(expr) => walk_expr_index(visitor, expr),
        ExprKind::Lit(expr) => walk_expr_lit(visitor, expr),
        ExprKind::Paren(expr) => walk_expr_paren(visitor, expr),
        ExprKind::Tuple(expr) => walk_expr_tuple(visitor, expr),
        ExprKind::ForLoop(expr) => walk_expr_for_loop(visitor, expr),
        ExprKind::Assign(expr) => walk_expr_assign(visitor, expr),
        ExprKind::Group(expr) => walk_expr_group(visitor, expr),
        ExprKind::Field(expr) => walk_expr_field(visitor, expr),
        ExprKind::Block(expr) => walk_expr_block(visitor, expr),
        ExprKind::Array(expr) => walk_expr_array(visitor, expr),
        ExprKind::Range(expr) => walk_expr_range(visitor, expr),
        ExprKind::Path(expr) => walk_expr_path(visitor, expr),
        ExprKind::Let(expr) => walk_expr_let(visitor, expr),
        ExprKind::Repeat(expr) => walk_expr_repeat(visitor, expr),
        ExprKind::Struct(expr) => walk_expr_struct(visitor, expr),
        ExprKind::Call(expr) => walk_expr_call(visitor, expr),
        ExprKind::MethodCall(expr) => walk_expr_method_call(visitor, expr),
    }
}

pub fn walk_path(visitor: &mut dyn Visitor, path: &Path) -> Result<()> {
    visitor.visit_path(path)?;
    for segment in &path.segments {
        walk_path_segment(visitor, segment)?;
    }
    Ok(())
}

pub fn walk_path_segment(visitor: &mut dyn Visitor, path_segment: &PathSegment) -> Result<()> {
    visitor.visit_path_segment(path_segment)?;
    Ok(())
}

pub fn walk_field_value(visitor: &mut dyn Visitor, field_value: &FieldValue) -> Result<()> {
    visitor.visit_field_value(field_value)?;
    walk_expr(visitor, &field_value.value)?;
    Ok(())
}
