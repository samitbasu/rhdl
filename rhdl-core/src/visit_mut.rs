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
    fn visit_mut_path_segment(&mut self, node: &mut PathSegment) -> Result<()> {
        visit_mut_path_segment(self, node)
    }
    fn visit_mut_path(&mut self, node: &mut Path) -> Result<()> {
        visit_mut_path(self, node)
    }
    fn visit_mut_pat_const(&mut self, node: &mut PatConst) -> Result<()> {
        visit_mut_pat_const(self, node)
    }
    fn visit_mut_pat_ident(&mut self, node: &mut PatIdent) -> Result<()> {
        visit_mut_pat_ident(self, node)
    }
    fn visit_mut_pat_tuple(&mut self, node: &mut PatTuple) -> Result<()> {
        visit_mut_pat_tuple(self, node)
    }
    fn visit_mut_pat_tuple_struct(&mut self, node: &mut PatTupleStruct) -> Result<()> {
        visit_mut_pat_tuple_struct(self, node)
    }
    fn visit_mut_pat_lit(&mut self, node: &mut PatLit) -> Result<()> {
        visit_mut_pat_lit(self, node)
    }
    fn visit_mut_pat_or(&mut self, node: &mut PatOr) -> Result<()> {
        visit_mut_pat_or(self, node)
    }
    fn visit_mut_pat_paren(&mut self, node: &mut PatParen) -> Result<()> {
        visit_mut_pat_paren(self, node)
    }
    fn visit_mut_pat_path(&mut self, node: &mut PatPath) -> Result<()> {
        visit_mut_pat_path(self, node)
    }
    fn visit_mut_pat_struct(&mut self, node: &mut PatStruct) -> Result<()> {
        visit_mut_pat_struct(self, node)
    }
    fn visit_mut_pat_slice(&mut self, node: &mut PatSlice) -> Result<()> {
        visit_mut_pat_slice(self, node)
    }
    fn visit_mut_pat_type(&mut self, node: &mut PatType) -> Result<()> {
        visit_mut_pat_type(self, node)
    }
    fn visit_pat_wild(&mut self) -> Result<()> {
        Ok(())
    }
    fn visit_mut_expr(&mut self, node: &mut Expr) -> Result<()> {
        visit_mut_expr(self, node)
    }
    fn visit_mut_expr_binary(&mut self, node: &mut ExprBinary) -> Result<()> {
        visit_mut_expr_binary(self, node)
    }
    fn visit_mut_expr_unary(&mut self, node: &mut ExprUnary) -> Result<()> {
        visit_mut_expr_unary(self, node)
    }
    fn visit_mut_expr_match(&mut self, node: &mut ExprMatch) -> Result<()> {
        visit_mut_expr_match(self, node)
    }
    fn visit_mut_expr_ret(&mut self, node: &mut ExprRet) -> Result<()> {
        visit_mut_expr_ret(self, node)
    }
    fn visit_mut_expr_if(&mut self, node: &mut ExprIf) -> Result<()> {
        visit_mut_expr_if(self, node)
    }
    fn visit_mut_expr_index(&mut self, node: &mut ExprIndex) -> Result<()> {
        visit_mut_expr_index(self, node)
    }
    fn visit_mut_expr_paren(&mut self, node: &mut ExprParen) -> Result<()> {
        visit_mut_expr_paren(self, node)
    }
    fn visit_mut_expr_tuple(&mut self, node: &mut ExprTuple) -> Result<()> {
        visit_mut_expr_tuple(self, node)
    }
    fn visit_mut_expr_for_loop(&mut self, node: &mut ExprForLoop) -> Result<()> {
        visit_mut_expr_for_loop(self, node)
    }
    fn visit_mut_expr_assign(&mut self, node: &mut ExprAssign) -> Result<()> {
        visit_mut_expr_assign(self, node)
    }
    fn visit_mut_expr_group(&mut self, node: &mut ExprGroup) -> Result<()> {
        visit_mut_expr_group(self, node)
    }
    fn visit_mut_expr_field(&mut self, node: &mut ExprField) -> Result<()> {
        visit_mut_expr_field(self, node)
    }
    fn visit_mut_expr_block(&mut self, node: &mut ExprBlock) -> Result<()> {
        visit_mut_expr_block(self, node)
    }
    fn visit_mut_expr_array(&mut self, node: &mut ExprArray) -> Result<()> {
        visit_mut_expr_array(self, node)
    }
    fn visit_mut_expr_range(&mut self, node: &mut ExprRange) -> Result<()> {
        visit_mut_expr_range(self, node)
    }
    fn visit_mut_expr_path(&mut self, node: &mut ExprPath) -> Result<()> {
        visit_mut_expr_path(self, node)
    }
    fn visit_mut_expr_let(&mut self, node: &mut ExprLet) -> Result<()> {
        visit_mut_expr_let(self, node)
    }
    fn visit_mut_expr_repeat(&mut self, node: &mut ExprRepeat) -> Result<()> {
        visit_mut_expr_repeat(self, node)
    }
    fn visit_mut_expr_struct(&mut self, node: &mut ExprStruct) -> Result<()> {
        visit_mut_expr_struct(self, node)
    }
    fn visit_mut_expr_call(&mut self, node: &mut ExprCall) -> Result<()> {
        visit_mut_expr_call(self, node)
    }
    fn visit_mut_expr_method_call(&mut self, node: &mut ExprMethodCall) -> Result<()> {
        visit_mut_expr_method_call(self, node)
    }
    fn visit_mut_match_arm(&mut self, node: &mut Arm) -> Result<()> {
        visit_mut_match_arm(self, node)
    }
    fn visit_mut_expr_lit(&mut self, node: &mut ExprLit) -> Result<()> {
        visit_mut_expr_lit(self, node)
    }
    fn visit_mut_field_value(&mut self, node: &mut FieldValue) -> Result<()> {
        visit_mut_field_value(self, node)
    }
    fn visit_mut_field_pat(&mut self, node: &mut FieldPat) -> Result<()> {
        visit_mut_field_pat(self, node)
    }
    fn visit_mut_expr_type(&mut self, node: &mut ExprType) -> Result<()> {
        visit_mut_expr_type(self, node)
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

pub fn visit_mut_pat_const<V>(visitor: &mut V, pat_const: &mut PatConst) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    Ok(())
}

pub fn visit_mut_pat_ident<V>(visitor: &mut V, pat_ident: &mut PatIdent) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    Ok(())
}

pub fn visit_mut_pat_slice<V>(visitor: &mut V, pat_slice: &mut PatSlice) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for pat in &mut pat_slice.elems {
        visitor.visit_mut_pat(pat)?;
    }
    Ok(())
}

pub fn visit_mut_pat_tuple<V>(visitor: &mut V, pat_tuple: &mut PatTuple) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for pat in &mut pat_tuple.elements {
        visitor.visit_mut_pat(pat)?;
    }
    Ok(())
}

pub fn visit_mut_pat_tuple_struct<V>(
    visitor: &mut V,
    pat_tuple_struct: &mut PatTupleStruct,
) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_path(&mut pat_tuple_struct.path)?;
    for pat in &mut pat_tuple_struct.elems {
        visitor.visit_mut_pat(pat)?;
    }
    Ok(())
}

pub fn visit_mut_pat_lit<V>(visitor: &mut V, pat_lit: &mut PatLit) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    Ok(())
}

pub fn visit_mut_pat_or<V>(visitor: &mut V, pat_or: &mut PatOr) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for pat in &mut pat_or.segments {
        visitor.visit_mut_pat(pat)?;
    }
    Ok(())
}

pub fn visit_mut_pat_paren<V>(visitor: &mut V, pat_paren: &mut PatParen) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_pat(&mut pat_paren.pat)?;
    Ok(())
}

pub fn visit_mut_pat_path<V>(visitor: &mut V, pat_path: &mut PatPath) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_path(&mut pat_path.path)?;
    Ok(())
}

pub fn visit_mut_pat_struct<V>(visitor: &mut V, pat_struct: &mut PatStruct) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_path(&mut pat_struct.path)?;
    for field in &mut pat_struct.fields {
        visitor.visit_mut_field_pat(field)?;
    }
    Ok(())
}

pub fn visit_mut_field_pat<V>(visitor: &mut V, field_pat: &mut FieldPat) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_pat(&mut field_pat.pat)?;
    Ok(())
}

pub fn visit_mut_pat_type<V>(visitor: &mut V, pat_type: &mut PatType) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_pat(&mut pat_type.pat)?;
    Ok(())
}

pub fn visit_mut_pat_wild<V>(visitor: &mut V) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    Ok(())
}

pub fn visit_mut_pat<V>(visitor: &mut V, pat: &mut Pat) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    match &mut pat.kind {
        PatKind::Const(pat_const) => {
            visitor.visit_mut_pat_const(pat_const)?;
        }
        PatKind::Ident(pat) => {
            visitor.visit_mut_pat_ident(pat)?;
        }
        PatKind::Tuple(tuple) => {
            visitor.visit_mut_pat_tuple(tuple)?;
        }
        PatKind::TupleStruct(tuple_struct) => {
            visitor.visit_mut_pat_tuple_struct(tuple_struct)?;
        }
        PatKind::Lit(lit) => {
            visitor.visit_mut_pat_lit(lit)?;
        }
        PatKind::Or(pat_or) => {
            visitor.visit_mut_pat_or(pat_or)?;
        }
        PatKind::Paren(pat_paren) => {
            visitor.visit_mut_pat_paren(pat_paren)?;
        }
        PatKind::Path(path) => {
            visitor.visit_mut_pat_path(path)?;
        }
        PatKind::Struct(structure) => {
            visitor.visit_mut_pat_struct(structure)?;
        }
        PatKind::Type(pat_type) => {
            visitor.visit_mut_pat_type(pat_type)?;
        }
        PatKind::Slice(pat_slice) => {
            visitor.visit_mut_pat_slice(pat_slice)?;
        }
        PatKind::Wild => {
            visit_mut_pat_wild(visitor)?;
        }
    }
    Ok(())
}

pub fn visit_mut_expr_binary<V>(visitor: &mut V, expr_binary: &mut ExprBinary) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_binary.lhs)?;
    visitor.visit_mut_expr(&mut expr_binary.rhs)?;
    Ok(())
}

pub fn visit_mut_expr_unary<V>(visitor: &mut V, expr_unary: &mut ExprUnary) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_unary.expr)?;
    Ok(())
}

pub fn visit_mut_expr_match<V>(visitor: &mut V, expr_match: &mut ExprMatch) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_match.expr)?;
    for arm in &mut expr_match.arms {
        visitor.visit_mut_match_arm(arm)?;
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

pub fn visit_mut_expr_ret<V>(visitor: &mut V, expr_return: &mut ExprRet) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    if let Some(expr) = &mut expr_return.expr {
        visitor.visit_mut_expr(expr)?;
    }
    Ok(())
}

pub fn visit_mut_expr_if<V>(visitor: &mut V, expr_if: &mut ExprIf) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_if.cond)?;
    visitor.visit_mut_block(&mut expr_if.then_branch)?;
    if let Some(else_branch) = &mut expr_if.else_branch {
        visitor.visit_mut_expr(else_branch)?;
    }
    Ok(())
}

pub fn visit_mut_expr_index<V>(visitor: &mut V, expr_index: &mut ExprIndex) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_index.expr)?;
    visitor.visit_mut_expr(&mut expr_index.index)?;
    Ok(())
}

pub fn visit_mut_expr_lit<V>(visitor: &mut V, lit: &mut ExprLit) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    Ok(())
}

pub fn visit_mut_expr_paren<V>(visitor: &mut V, expr_paren: &mut ExprParen) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_paren.expr)?;
    Ok(())
}

pub fn visit_mut_expr_tuple<V>(visitor: &mut V, expr_tuple: &mut ExprTuple) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for expr in &mut expr_tuple.elements {
        visitor.visit_mut_expr(expr)?;
    }
    Ok(())
}

pub fn visit_mut_expr_for_loop<V>(visitor: &mut V, expr_for_loop: &mut ExprForLoop) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_pat(&mut expr_for_loop.pat)?;
    visitor.visit_mut_expr(&mut expr_for_loop.expr)?;
    visitor.visit_mut_block(&mut expr_for_loop.body)?;
    Ok(())
}

pub fn visit_mut_expr_assign<V>(visitor: &mut V, expr_assign: &mut ExprAssign) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_assign.lhs)?;
    visitor.visit_mut_expr(&mut expr_assign.rhs)?;
    Ok(())
}

pub fn visit_mut_expr_group<V>(visitor: &mut V, expr_group: &mut ExprGroup) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_group.expr)?;
    Ok(())
}

pub fn visit_mut_expr_field<V>(visitor: &mut V, expr_field: &mut ExprField) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_field.expr)?;
    Ok(())
}

pub fn visit_mut_expr_block<V>(visitor: &mut V, expr_block: &mut ExprBlock) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_block(&mut expr_block.block)?;
    Ok(())
}

pub fn visit_mut_expr_array<V>(visitor: &mut V, expr_array: &mut ExprArray) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for expr in &mut expr_array.elems {
        visitor.visit_mut_expr(expr)?;
    }
    Ok(())
}

pub fn visit_mut_expr_range<V>(visitor: &mut V, expr_range: &mut ExprRange) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    if let Some(start) = &mut expr_range.start {
        visitor.visit_mut_expr(start)?;
    }
    if let Some(end) = &mut expr_range.end {
        visitor.visit_mut_expr(end)?;
    }
    Ok(())
}

pub fn visit_mut_expr_path<V>(visitor: &mut V, expr_path: &mut ExprPath) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_path(&mut expr_path.path)?;
    Ok(())
}

pub fn visit_mut_expr_let<V>(visitor: &mut V, expr_let: &mut ExprLet) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_pat(&mut expr_let.pattern)?;
    visitor.visit_mut_expr(&mut expr_let.value)?;
    Ok(())
}

pub fn visit_mut_expr_repeat<V>(visitor: &mut V, expr_repeat: &mut ExprRepeat) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_repeat.value)?;
    visitor.visit_mut_expr(&mut expr_repeat.len)?;
    Ok(())
}

pub fn visit_mut_expr_struct<V>(visitor: &mut V, expr_struct: &mut ExprStruct) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_path(&mut expr_struct.path)?;
    for field in &mut expr_struct.fields {
        visitor.visit_mut_field_value(field)?;
    }
    if let Some(rest) = &mut expr_struct.rest {
        visitor.visit_mut_expr(rest)?;
    }
    Ok(())
}

pub fn visit_mut_expr_call<V>(visitor: &mut V, expr_call: &mut ExprCall) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_path(&mut expr_call.path)?;
    for arg in &mut expr_call.args {
        visitor.visit_mut_expr(arg)?;
    }
    Ok(())
}

pub fn visit_mut_expr_method_call<V>(
    visitor: &mut V,
    expr_method_call: &mut ExprMethodCall,
) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut expr_method_call.receiver)?;
    for arg in &mut expr_method_call.args {
        visitor.visit_mut_expr(arg)?;
    }
    Ok(())
}

pub fn visit_mut_expr_type<V>(visitor: &mut V, expr_type: &mut ExprType) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    Ok(())
}

pub fn visit_mut_expr<V>(visitor: &mut V, expr: &mut Expr) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    match &mut expr.kind {
        ExprKind::Binary(expr) => visitor.visit_mut_expr_binary(expr),
        ExprKind::Unary(expr) => visitor.visit_mut_expr_unary(expr),
        ExprKind::Match(expr) => visitor.visit_mut_expr_match(expr),
        ExprKind::Ret(expr) => visitor.visit_mut_expr_ret(expr),
        ExprKind::If(expr) => visitor.visit_mut_expr_if(expr),
        ExprKind::Index(expr) => visitor.visit_mut_expr_index(expr),
        ExprKind::Lit(expr) => visitor.visit_mut_expr_lit(expr),
        ExprKind::Paren(expr) => visitor.visit_mut_expr_paren(expr),
        ExprKind::Tuple(expr) => visitor.visit_mut_expr_tuple(expr),
        ExprKind::ForLoop(expr) => visitor.visit_mut_expr_for_loop(expr),
        ExprKind::Assign(expr) => visitor.visit_mut_expr_assign(expr),
        ExprKind::Group(expr) => visitor.visit_mut_expr_group(expr),
        ExprKind::Field(expr) => visitor.visit_mut_expr_field(expr),
        ExprKind::Block(expr) => visitor.visit_mut_expr_block(expr),
        ExprKind::Array(expr) => visitor.visit_mut_expr_array(expr),
        ExprKind::Range(expr) => visitor.visit_mut_expr_range(expr),
        ExprKind::Path(expr) => visitor.visit_mut_expr_path(expr),
        ExprKind::Let(expr) => visitor.visit_mut_expr_let(expr),
        ExprKind::Repeat(expr) => visitor.visit_mut_expr_repeat(expr),
        ExprKind::Struct(expr) => visitor.visit_mut_expr_struct(expr),
        ExprKind::Call(expr) => visitor.visit_mut_expr_call(expr),
        ExprKind::MethodCall(expr) => visitor.visit_mut_expr_method_call(expr),
        ExprKind::Type(expr) => visitor.visit_mut_expr_type(expr),
    }
}

pub fn visit_mut_path<V>(visitor: &mut V, path: &mut Path) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    for segment in &mut path.segments {
        visitor.visit_mut_path_segment(segment)?;
    }
    Ok(())
}

pub fn visit_mut_path_segment<V>(visitor: &mut V, path_segment: &mut PathSegment) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    Ok(())
}

pub fn visit_mut_field_value<V>(visitor: &mut V, field_value: &mut FieldValue) -> Result<()>
where
    V: VisitorMut + ?Sized,
{
    visitor.visit_mut_expr(&mut field_value.value)?;
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
