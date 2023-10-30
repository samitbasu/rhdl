// A visitor pattern for the ast
// To use this, impl Visitor on a data structure, and then pass it to the appropriate top
// level walk_ function.

use crate::ast::*;
use anyhow::Result;

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
    fn visit_path_segment(&mut self, node: &PathSegment) -> Result<()> {
        visit_path_segment(self, node)
    }
    fn visit_path(&mut self, node: &Path) -> Result<()> {
        visit_path(self, node)
    }
    fn visit_pat_ident(&mut self, node: &PatIdent) -> Result<()> {
        visit_pat_ident(self, node)
    }
    fn visit_pat_tuple(&mut self, node: &PatTuple) -> Result<()> {
        visit_pat_tuple(self, node)
    }
    fn visit_pat_tuple_struct(&mut self, node: &PatTupleStruct) -> Result<()> {
        visit_pat_tuple_struct(self, node)
    }
    fn visit_pat_lit(&mut self, node: &PatLit) -> Result<()> {
        visit_pat_lit(self, node)
    }
    fn visit_pat_or(&mut self, node: &PatOr) -> Result<()> {
        visit_pat_or(self, node)
    }
    fn visit_pat_paren(&mut self, node: &PatParen) -> Result<()> {
        visit_pat_paren(self, node)
    }
    fn visit_pat_path(&mut self, node: &PatPath) -> Result<()> {
        visit_pat_path(self, node)
    }
    fn visit_pat_struct(&mut self, node: &PatStruct) -> Result<()> {
        visit_pat_struct(self, node)
    }
    fn visit_pat_type(&mut self, node: &PatType) -> Result<()> {
        visit_pat_type(self, node)
    }
    fn visit_pat_slice(&mut self, node: &PatSlice) -> Result<()> {
        visit_pat_slice(self, node)
    }
    fn visit_pat_wild(&mut self) -> Result<()> {
        visit_pat_wild(self)
    }
    fn visit_expr(&mut self, node: &Expr) -> Result<()> {
        visit_expr(self, node)
    }
    fn visit_expr_binary(&mut self, node: &ExprBinary) -> Result<()> {
        visit_expr_binary(self, node)
    }
    fn visit_expr_unary(&mut self, node: &ExprUnary) -> Result<()> {
        visit_expr_unary(self, node)
    }
    fn visit_expr_match(&mut self, node: &ExprMatch) -> Result<()> {
        visit_expr_match(self, node)
    }
    fn visit_expr_ret(&mut self, node: &ExprRet) -> Result<()> {
        visit_expr_ret(self, node)
    }
    fn visit_expr_if(&mut self, node: &ExprIf) -> Result<()> {
        visit_expr_if(self, node)
    }
    fn visit_expr_index(&mut self, node: &ExprIndex) -> Result<()> {
        visit_expr_index(self, node)
    }
    fn visit_expr_paren(&mut self, node: &ExprParen) -> Result<()> {
        visit_expr_paren(self, node)
    }
    fn visit_expr_tuple(&mut self, node: &ExprTuple) -> Result<()> {
        visit_expr_tuple(self, node)
    }
    fn visit_expr_for_loop(&mut self, node: &ExprForLoop) -> Result<()> {
        visit_expr_for_loop(self, node)
    }
    fn visit_expr_assign(&mut self, node: &ExprAssign) -> Result<()> {
        visit_expr_assign(self, node)
    }
    fn visit_expr_group(&mut self, node: &ExprGroup) -> Result<()> {
        visit_expr_group(self, node)
    }
    fn visit_expr_field(&mut self, node: &ExprField) -> Result<()> {
        visit_expr_field(self, node)
    }
    fn visit_expr_block(&mut self, node: &ExprBlock) -> Result<()> {
        visit_expr_block(self, node)
    }
    fn visit_expr_array(&mut self, node: &ExprArray) -> Result<()> {
        visit_expr_array(self, node)
    }
    fn visit_expr_range(&mut self, node: &ExprRange) -> Result<()> {
        visit_expr_range(self, node)
    }
    fn visit_expr_path(&mut self, node: &ExprPath) -> Result<()> {
        visit_expr_path(self, node)
    }
    fn visit_expr_let(&mut self, node: &ExprLet) -> Result<()> {
        visit_expr_let(self, node)
    }
    fn visit_expr_repeat(&mut self, node: &ExprRepeat) -> Result<()> {
        visit_expr_repeat(self, node)
    }
    fn visit_expr_struct(&mut self, node: &ExprStruct) -> Result<()> {
        visit_expr_struct(self, node)
    }
    fn visit_expr_call(&mut self, node: &ExprCall) -> Result<()> {
        visit_expr_call(self, node)
    }
    fn visit_expr_method_call(&mut self, node: &ExprMethodCall) -> Result<()> {
        visit_expr_method_call(self, node)
    }
    fn visit_match_arm(&mut self, node: &Arm) -> Result<()> {
        visit_match_arm(self, node)
    }
    fn visit_expr_lit(&mut self, node: &ExprLit) -> Result<()> {
        visit_expr_lit(self, node)
    }
    fn visit_field_value(&mut self, node: &FieldValue) -> Result<()> {
        visit_field_value(self, node)
    }
    fn visit_field_pat(&mut self, node: &FieldPat) -> Result<()> {
        visit_field_pat(self, node)
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

pub fn visit_pat_ident<V>(visitor: &mut V, pat_ident: &PatIdent) -> Result<()>
where
    V: Visitor + ?Sized,
{
    Ok(())
}

pub fn visit_pat_slice<V>(visitor: &mut V, pat_slice: &PatSlice) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for pat in &pat_slice.elems {
        visitor.visit_pat(pat)?;
    }
    Ok(())
}

pub fn visit_pat_tuple<V>(visitor: &mut V, pat_tuple: &PatTuple) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for pat in &pat_tuple.elements {
        visitor.visit_pat(pat)?;
    }
    Ok(())
}

pub fn visit_pat_tuple_struct<V>(visitor: &mut V, pat_tuple_struct: &PatTupleStruct) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_path(&pat_tuple_struct.path)?;
    for pat in &pat_tuple_struct.elems {
        visitor.visit_pat(pat)?;
    }
    Ok(())
}

pub fn visit_pat_lit<V>(visitor: &mut V, pat_lit: &PatLit) -> Result<()>
where
    V: Visitor + ?Sized,
{
    Ok(())
}

pub fn visit_pat_or<V>(visitor: &mut V, pat_or: &PatOr) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for pat in &pat_or.segments {
        visitor.visit_pat(pat)?;
    }
    Ok(())
}

pub fn visit_pat_paren<V>(visitor: &mut V, pat_paren: &PatParen) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_pat(&pat_paren.pat)?;
    Ok(())
}

pub fn visit_pat_path<V>(visitor: &mut V, pat_path: &PatPath) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_path(&pat_path.path)?;
    Ok(())
}

pub fn visit_pat_struct<V>(visitor: &mut V, pat_struct: &PatStruct) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_path(&pat_struct.path)?;
    for field in &pat_struct.fields {
        visitor.visit_field_pat(field)?;
    }
    Ok(())
}

pub fn visit_field_pat<V>(visitor: &mut V, field_pat: &FieldPat) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_pat(&field_pat.pat)?;
    Ok(())
}

pub fn visit_pat_type<V>(visitor: &mut V, pat_type: &PatType) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_pat(&pat_type.pat)?;
    Ok(())
}

pub fn visit_pat_wild<V>(visitor: &mut V) -> Result<()>
where
    V: Visitor + ?Sized,
{
    Ok(())
}

pub fn visit_pat<V>(visitor: &mut V, pat: &Pat) -> Result<()>
where
    V: Visitor + ?Sized,
{
    match &pat.kind {
        PatKind::Ident(pat) => {
            visitor.visit_pat_ident(pat)?;
        }
        PatKind::Tuple(tuple) => {
            visitor.visit_pat_tuple(tuple)?;
        }
        PatKind::Slice(slice) => {
            visitor.visit_pat_slice(slice)?;
        }
        PatKind::TupleStruct(tuple_struct) => {
            visitor.visit_pat_tuple_struct(tuple_struct)?;
        }
        PatKind::Lit(lit) => {
            visitor.visit_pat_lit(lit)?;
        }
        PatKind::Or(pat_or) => {
            visitor.visit_pat_or(pat_or)?;
        }
        PatKind::Paren(pat_paren) => {
            visitor.visit_pat_paren(pat_paren)?;
        }
        PatKind::Path(path) => {
            visitor.visit_pat_path(path)?;
        }
        PatKind::Struct(structure) => {
            visitor.visit_pat_struct(structure)?;
        }
        PatKind::Type(pat_type) => {
            visitor.visit_pat_type(pat_type)?;
        }
        PatKind::Wild => {
            visitor.visit_pat_wild()?;
        }
    }
    Ok(())
}

pub fn visit_expr_binary<V>(visitor: &mut V, expr_binary: &ExprBinary) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_binary.lhs)?;
    visitor.visit_expr(&expr_binary.rhs)?;
    Ok(())
}

pub fn visit_expr_unary<V>(visitor: &mut V, expr_unary: &ExprUnary) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_unary.expr)?;
    Ok(())
}

pub fn visit_expr_match<V>(visitor: &mut V, expr_match: &ExprMatch) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_match.expr)?;
    for arm in &expr_match.arms {
        visitor.visit_match_arm(arm)?;
    }
    Ok(())
}

pub fn visit_match_arm<V>(visitor: &mut V, arm: &Arm) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_pat(&arm.pattern)?;
    if let Some(guard) = &arm.guard {
        visitor.visit_expr(guard)?;
    }
    visitor.visit_expr(&arm.body)?;
    Ok(())
}

pub fn visit_expr_ret<V>(visitor: &mut V, expr_return: &ExprRet) -> Result<()>
where
    V: Visitor + ?Sized,
{
    if let Some(expr) = &expr_return.expr {
        visitor.visit_expr(expr)?;
    }
    Ok(())
}

pub fn visit_expr_if<V>(visitor: &mut V, expr_if: &ExprIf) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_if.cond)?;
    visitor.visit_block(&expr_if.then_branch)?;
    if let Some(else_branch) = &expr_if.else_branch {
        visitor.visit_expr(else_branch)?;
    }
    Ok(())
}

pub fn visit_expr_index<V>(visitor: &mut V, expr_index: &ExprIndex) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_index.expr)?;
    visitor.visit_expr(&expr_index.index)?;
    Ok(())
}

pub fn visit_expr_lit<V>(visitor: &mut V, lit: &ExprLit) -> Result<()>
where
    V: Visitor + ?Sized,
{
    Ok(())
}

pub fn visit_expr_paren<V>(visitor: &mut V, expr_paren: &ExprParen) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_paren.expr)?;
    Ok(())
}

pub fn visit_expr_tuple<V>(visitor: &mut V, expr_tuple: &ExprTuple) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for expr in &expr_tuple.elements {
        visitor.visit_expr(expr)?;
    }
    Ok(())
}

pub fn visit_expr_for_loop<V>(visitor: &mut V, expr_for_loop: &ExprForLoop) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_pat(&expr_for_loop.pat)?;
    visitor.visit_expr(&expr_for_loop.expr)?;
    visitor.visit_block(&expr_for_loop.body)?;
    Ok(())
}

pub fn visit_expr_assign<V>(visitor: &mut V, expr_assign: &ExprAssign) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_assign.lhs)?;
    visitor.visit_expr(&expr_assign.rhs)?;
    Ok(())
}

pub fn visit_expr_group<V>(visitor: &mut V, expr_group: &ExprGroup) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_group.expr)?;
    Ok(())
}

pub fn visit_expr_field<V>(visitor: &mut V, expr_field: &ExprField) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_field.expr)?;
    Ok(())
}

pub fn visit_expr_block<V>(visitor: &mut V, expr_block: &ExprBlock) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_block(&expr_block.block)?;
    Ok(())
}

pub fn visit_expr_array<V>(visitor: &mut V, expr_array: &ExprArray) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for expr in &expr_array.elems {
        visitor.visit_expr(expr)?;
    }
    Ok(())
}

pub fn visit_expr_range<V>(visitor: &mut V, expr_range: &ExprRange) -> Result<()>
where
    V: Visitor + ?Sized,
{
    if let Some(start) = &expr_range.start {
        visitor.visit_expr(start)?;
    }
    if let Some(end) = &expr_range.end {
        visitor.visit_expr(end)?;
    }
    Ok(())
}

pub fn visit_expr_path<V>(visitor: &mut V, expr_path: &ExprPath) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_path(&expr_path.path)?;
    Ok(())
}

pub fn visit_expr_let<V>(visitor: &mut V, expr_let: &ExprLet) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_pat(&expr_let.pattern)?;
    visitor.visit_expr(&expr_let.value)?;
    Ok(())
}

pub fn visit_expr_repeat<V>(visitor: &mut V, expr_repeat: &ExprRepeat) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_repeat.value)?;
    visitor.visit_expr(&expr_repeat.len)?;
    Ok(())
}

pub fn visit_expr_struct<V>(visitor: &mut V, expr_struct: &ExprStruct) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_path(&expr_struct.path)?;
    for field in &expr_struct.fields {
        visitor.visit_field_value(field)?;
    }
    if let Some(rest) = &expr_struct.rest {
        visitor.visit_expr(rest)?;
    }
    Ok(())
}

pub fn visit_expr_call<V>(visitor: &mut V, expr_call: &ExprCall) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_path(&expr_call.path)?;
    for arg in &expr_call.args {
        visitor.visit_expr(arg)?;
    }
    Ok(())
}

pub fn visit_expr_method_call<V>(visitor: &mut V, expr_method_call: &ExprMethodCall) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&expr_method_call.receiver)?;
    for arg in &expr_method_call.args {
        visitor.visit_expr(arg)?;
    }
    Ok(())
}

pub fn visit_expr<V>(visitor: &mut V, expr: &Expr) -> Result<()>
where
    V: Visitor + ?Sized,
{
    match &expr.kind {
        ExprKind::Binary(expr) => visitor.visit_expr_binary(expr),
        ExprKind::Unary(expr) => visitor.visit_expr_unary(expr),
        ExprKind::Match(expr) => visitor.visit_expr_match(expr),
        ExprKind::Ret(expr) => visitor.visit_expr_ret(expr),
        ExprKind::If(expr) => visitor.visit_expr_if(expr),
        ExprKind::Index(expr) => visitor.visit_expr_index(expr),
        ExprKind::Lit(expr) => visitor.visit_expr_lit(expr),
        ExprKind::Paren(expr) => visitor.visit_expr_paren(expr),
        ExprKind::Tuple(expr) => visitor.visit_expr_tuple(expr),
        ExprKind::ForLoop(expr) => visitor.visit_expr_for_loop(expr),
        ExprKind::Assign(expr) => visitor.visit_expr_assign(expr),
        ExprKind::Group(expr) => visitor.visit_expr_group(expr),
        ExprKind::Field(expr) => visitor.visit_expr_field(expr),
        ExprKind::Block(expr) => visitor.visit_expr_block(expr),
        ExprKind::Array(expr) => visitor.visit_expr_array(expr),
        ExprKind::Range(expr) => visitor.visit_expr_range(expr),
        ExprKind::Path(expr) => visitor.visit_expr_path(expr),
        ExprKind::Let(expr) => visitor.visit_expr_let(expr),
        ExprKind::Repeat(expr) => visitor.visit_expr_repeat(expr),
        ExprKind::Struct(expr) => visitor.visit_expr_struct(expr),
        ExprKind::Call(expr) => visitor.visit_expr_call(expr),
        ExprKind::MethodCall(expr) => visitor.visit_expr_method_call(expr),
    }
}

pub fn visit_path<V>(visitor: &mut V, path: &Path) -> Result<()>
where
    V: Visitor + ?Sized,
{
    for segment in &path.segments {
        visitor.visit_path_segment(segment)?;
    }
    Ok(())
}

pub fn visit_path_segment<V>(visitor: &mut V, path_segment: &PathSegment) -> Result<()>
where
    V: Visitor + ?Sized,
{
    Ok(())
}

pub fn visit_field_value<V>(visitor: &mut V, field_value: &FieldValue) -> Result<()>
where
    V: Visitor + ?Sized,
{
    visitor.visit_expr(&field_value.value)?;
    Ok(())
}
