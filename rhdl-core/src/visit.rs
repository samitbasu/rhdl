// A visitor pattern for the ast
// To use this, impl Visitor on a data structure, and then pass it to the appropriate top
// level walk_ function.

use crate::ast::*;

pub trait Visitor {
    fn visit_block(&mut self, _block: &mut Block) {}
    fn visit_stmt(&mut self, _stmt: &mut Stmt) {}
    fn visit_local(&mut self, _local: &mut Local) {}
    fn visit_pat(&mut self, _pat: &mut Pat) {}
    fn visit_path_segment(&mut self, _path_segment: &mut PathSegment) {}
    fn visit_path(&mut self, _path: &mut Path) {}
    fn visit_pat_ident(&mut self, _pat_ident: &mut PatIdent) {}
    fn visit_pat_tuple(&mut self, _pat_tuple: &mut PatTuple) {}
    fn visit_pat_tuple_struct(&mut self, _pat_tuple_struct: &mut PatTupleStruct) {}
    fn visit_pat_lit(&mut self, _pat_lit: &mut PatLit) {}
    fn visit_pat_or(&mut self, _pat_or: &mut PatOr) {}
    fn visit_pat_paren(&mut self, _pat_paren: &mut PatParen) {}
    fn visit_pat_path(&mut self, _pat_path: &mut PatPath) {}
    fn visit_pat_struct(&mut self, _pat_struct: &mut PatStruct) {}
    fn visit_pat_type(&mut self, _pat_type: &mut PatType) {}
    fn visit_pat_wild(&mut self) {}
    fn visit_expr(&mut self, _expr: &mut Expr) {}
    fn visit_expr_binary(&mut self, _expr_binary: &mut ExprBinary) {}
    fn visit_expr_unary(&mut self, _expr_unary: &mut ExprUnary) {}
    fn visit_expr_match(&mut self, _expr_match: &mut ExprMatch) {}
    fn visit_expr_ret(&mut self, _expr_return: &mut ExprRet) {}
    fn visit_expr_if(&mut self, _expr_if: &mut ExprIf) {}
    fn visit_expr_index(&mut self, _expr_index: &mut ExprIndex) {}
    fn visit_expr_paren(&mut self, _expr_paren: &mut ExprParen) {}
    fn visit_expr_tuple(&mut self, _expr_tuple: &mut ExprTuple) {}
    fn visit_expr_for_loop(&mut self, _expr_for_loop: &mut ExprForLoop) {}
    fn visit_expr_assign(&mut self, _expr_assign: &mut ExprAssign) {}
    fn visit_expr_group(&mut self, _expr_group: &mut ExprGroup) {}
    fn visit_expr_field(&mut self, _expr_field: &mut ExprField) {}
    fn visit_expr_block(&mut self, _expr_block: &mut ExprBlock) {}
    fn visit_expr_array(&mut self, _expr_array: &mut ExprArray) {}
    fn visit_expr_range(&mut self, _expr_range: &mut ExprRange) {}
    fn visit_expr_path(&mut self, _expr_path: &mut ExprPath) {}
    fn visit_expr_let(&mut self, _expr_let: &mut ExprLet) {}
    fn visit_expr_repeat(&mut self, _expr_repeat: &mut ExprRepeat) {}
    fn visit_expr_struct(&mut self, _expr_struct: &mut ExprStruct) {}
    fn visit_expr_call(&mut self, _expr_call: &mut ExprCall) {}
    fn visit_expr_method_call(&mut self, _expr_method_call: &mut ExprMethodCall) {}
    fn visit_match_arm(&mut self, _arm: &mut Arm) {}
    fn visit_expr_lit(&mut self, _lit: &mut ExprLit) {}
    fn visit_field_value(&mut self, _field_value: &mut FieldValue) {}
    fn visit_field_pat(&mut self, _field_pat: &mut FieldPat) {}
}

pub fn walk_block(visitor: &mut dyn Visitor, block: &mut Block) {
    visitor.visit_block(block);
    for stmt in &mut block.stmts {
        walk_stmt(visitor, stmt);
    }
}

pub fn walk_stmt(visitor: &mut dyn Visitor, stmt: &mut Stmt) {
    visitor.visit_stmt(stmt);
    match &mut stmt.kind {
        StmtKind::Local(local) => {
            walk_local(visitor, local);
        }
        StmtKind::Expr(expr) => {
            walk_expr(visitor, expr);
        }
        StmtKind::Semi(expr) => {
            walk_expr(visitor, expr);
        }
    }
}

pub fn walk_local(visitor: &mut dyn Visitor, local: &mut Local) {
    visitor.visit_local(local);
    walk_pat(visitor, &mut local.pat);
    if let Some(init) = &mut local.init {
        walk_expr(visitor, init);
    }
}

pub fn walk_pat_ident(visitor: &mut dyn Visitor, pat_ident: &mut PatIdent) {
    visitor.visit_pat_ident(pat_ident);
}

pub fn walk_pat_tuple(visitor: &mut dyn Visitor, pat_tuple: &mut PatTuple) {
    visitor.visit_pat_tuple(pat_tuple);
    for pat in &mut pat_tuple.elements {
        walk_pat(visitor, pat);
    }
}

pub fn walk_pat_tuple_struct(visitor: &mut dyn Visitor, pat_tuple_struct: &mut PatTupleStruct) {
    visitor.visit_pat_tuple_struct(pat_tuple_struct);
    walk_path(visitor, &mut pat_tuple_struct.path);
    for pat in &mut pat_tuple_struct.elems {
        walk_pat(visitor, pat);
    }
}

pub fn walk_pat_lit(visitor: &mut dyn Visitor, pat_lit: &mut PatLit) {
    visitor.visit_pat_lit(pat_lit);
}

pub fn walk_pat_or(visitor: &mut dyn Visitor, pat_or: &mut PatOr) {
    visitor.visit_pat_or(pat_or);
    for pat in &mut pat_or.segments {
        walk_pat(visitor, pat);
    }
}

pub fn walk_pat_paren(visitor: &mut dyn Visitor, pat_paren: &mut PatParen) {
    visitor.visit_pat_paren(pat_paren);
    walk_pat(visitor, &mut pat_paren.pat);
}

pub fn walk_pat_path(visitor: &mut dyn Visitor, pat_path: &mut PatPath) {
    visitor.visit_pat_path(pat_path);
    walk_path(visitor, &mut pat_path.path);
}

pub fn walk_pat_struct(visitor: &mut dyn Visitor, pat_struct: &mut PatStruct) {
    visitor.visit_pat_struct(pat_struct);
    walk_path(visitor, &mut pat_struct.path);
    for field in &mut pat_struct.fields {
        walk_field_pat(visitor, field);
    }
}

pub fn walk_field_pat(visitor: &mut dyn Visitor, field_pat: &mut FieldPat) {
    visitor.visit_field_pat(field_pat);
    walk_pat(visitor, &mut field_pat.pat);
}

pub fn walk_pat_type(visitor: &mut dyn Visitor, pat_type: &mut PatType) {
    visitor.visit_pat_type(pat_type);
    walk_pat(visitor, &mut pat_type.pat);
}

pub fn walk_pat_wild(visitor: &mut dyn Visitor) {
    visitor.visit_pat_wild();
}

pub fn walk_pat(visitor: &mut dyn Visitor, pat: &mut Pat) {
    visitor.visit_pat(pat);
    match &mut pat.kind {
        PatKind::Ident(pat) => {
            walk_pat_ident(visitor, pat);
        }
        PatKind::Tuple(tuple) => {
            walk_pat_tuple(visitor, tuple);
        }
        PatKind::TupleStruct(tuple_struct) => {
            walk_pat_tuple_struct(visitor, tuple_struct);
        }
        PatKind::Lit(lit) => {
            walk_pat_lit(visitor, lit);
        }
        PatKind::Or(pat_or) => {
            walk_pat_or(visitor, pat_or);
        }
        PatKind::Paren(pat_paren) => {
            walk_pat_paren(visitor, pat_paren);
        }
        PatKind::Path(path) => {
            walk_pat_path(visitor, path);
        }
        PatKind::Struct(structure) => {
            walk_pat_struct(visitor, structure);
        }
        PatKind::Type(pat_type) => {
            walk_pat_type(visitor, pat_type);
        }
        PatKind::Wild => {
            walk_pat_wild(visitor);
        }
    }
}

pub fn walk_expr_binary(visitor: &mut dyn Visitor, expr_binary: &mut ExprBinary) {
    visitor.visit_expr_binary(expr_binary);
    walk_expr(visitor, &mut expr_binary.lhs);
    walk_expr(visitor, &mut expr_binary.rhs);
}

pub fn walk_expr_unary(visitor: &mut dyn Visitor, expr_unary: &mut ExprUnary) {
    visitor.visit_expr_unary(expr_unary);
    walk_expr(visitor, &mut expr_unary.expr);
}

pub fn walk_expr_match(visitor: &mut dyn Visitor, expr_match: &mut ExprMatch) {
    visitor.visit_expr_match(expr_match);
    walk_expr(visitor, &mut expr_match.expr);
    for arm in &mut expr_match.arms {
        walk_match_arm(visitor, arm);
    }
}

pub fn walk_match_arm(visitor: &mut dyn Visitor, arm: &mut Arm) {
    visitor.visit_match_arm(arm);
    walk_pat(visitor, &mut arm.pattern);
    if let Some(guard) = &mut arm.guard {
        walk_expr(visitor, guard);
    }
    walk_expr(visitor, &mut arm.body);
}

pub fn walk_expr_ret(visitor: &mut dyn Visitor, expr_return: &mut ExprRet) {
    visitor.visit_expr_ret(expr_return);
    if let Some(expr) = &mut expr_return.expr {
        walk_expr(visitor, expr);
    }
}

pub fn walk_expr_if(visitor: &mut dyn Visitor, expr_if: &mut ExprIf) {
    visitor.visit_expr_if(expr_if);
    walk_expr(visitor, &mut expr_if.cond);
    walk_block(visitor, &mut expr_if.then_branch);
    if let Some(else_branch) = &mut expr_if.else_branch {
        walk_expr(visitor, else_branch);
    }
}

pub fn walk_expr_index(visitor: &mut dyn Visitor, expr_index: &mut ExprIndex) {
    visitor.visit_expr_index(expr_index);
    walk_expr(visitor, &mut expr_index.expr);
    walk_expr(visitor, &mut expr_index.index);
}

pub fn walk_expr_lit(visitor: &mut dyn Visitor, lit: &mut ExprLit) {
    visitor.visit_expr_lit(lit);
}

pub fn walk_expr_paren(visitor: &mut dyn Visitor, expr_paren: &mut ExprParen) {
    visitor.visit_expr_paren(expr_paren);
    walk_expr(visitor, &mut expr_paren.expr);
}

pub fn walk_expr_tuple(visitor: &mut dyn Visitor, expr_tuple: &mut ExprTuple) {
    visitor.visit_expr_tuple(expr_tuple);
    for expr in &mut expr_tuple.elements {
        walk_expr(visitor, expr);
    }
}

pub fn walk_expr_for_loop(visitor: &mut dyn Visitor, expr_for_loop: &mut ExprForLoop) {
    visitor.visit_expr_for_loop(expr_for_loop);
    walk_pat(visitor, &mut expr_for_loop.pat);
    walk_expr(visitor, &mut expr_for_loop.expr);
    walk_block(visitor, &mut expr_for_loop.body);
}

pub fn walk_expr_assign(visitor: &mut dyn Visitor, expr_assign: &mut ExprAssign) {
    visitor.visit_expr_assign(expr_assign);
    walk_expr(visitor, &mut expr_assign.lhs);
    walk_expr(visitor, &mut expr_assign.rhs);
}

pub fn walk_expr_group(visitor: &mut dyn Visitor, expr_group: &mut ExprGroup) {
    visitor.visit_expr_group(expr_group);
    walk_expr(visitor, &mut expr_group.expr);
}

pub fn walk_expr_field(visitor: &mut dyn Visitor, expr_field: &mut ExprField) {
    visitor.visit_expr_field(expr_field);
    walk_expr(visitor, &mut expr_field.expr);
}

pub fn walk_expr_block(visitor: &mut dyn Visitor, expr_block: &mut ExprBlock) {
    visitor.visit_expr_block(expr_block);
    walk_block(visitor, &mut expr_block.block);
}

pub fn walk_expr_array(visitor: &mut dyn Visitor, expr_array: &mut ExprArray) {
    visitor.visit_expr_array(expr_array);
    for expr in &mut expr_array.elems {
        walk_expr(visitor, expr);
    }
}

pub fn walk_expr_range(visitor: &mut dyn Visitor, expr_range: &mut ExprRange) {
    visitor.visit_expr_range(expr_range);
    if let Some(start) = &mut expr_range.start {
        walk_expr(visitor, start);
    }
    if let Some(end) = &mut expr_range.end {
        walk_expr(visitor, end);
    }
}

pub fn walk_expr_path(visitor: &mut dyn Visitor, expr_path: &mut ExprPath) {
    visitor.visit_expr_path(expr_path);
    walk_path(visitor, &mut expr_path.path);
}

pub fn walk_expr_let(visitor: &mut dyn Visitor, expr_let: &mut ExprLet) {
    visitor.visit_expr_let(expr_let);
    walk_pat(visitor, &mut expr_let.pattern);
    walk_expr(visitor, &mut expr_let.value);
    walk_expr(visitor, &mut expr_let.body);
}

pub fn walk_expr_repeat(visitor: &mut dyn Visitor, expr_repeat: &mut ExprRepeat) {
    visitor.visit_expr_repeat(expr_repeat);
    walk_expr(visitor, &mut expr_repeat.value);
    walk_expr(visitor, &mut expr_repeat.len);
}

pub fn walk_expr_struct(visitor: &mut dyn Visitor, expr_struct: &mut ExprStruct) {
    visitor.visit_expr_struct(expr_struct);
    walk_path(visitor, &mut expr_struct.path);
    for field in &mut expr_struct.fields {
        walk_field_value(visitor, field);
    }
    if let Some(rest) = &mut expr_struct.rest {
        walk_expr(visitor, rest);
    }
}

pub fn walk_expr_call(visitor: &mut dyn Visitor, expr_call: &mut ExprCall) {
    visitor.visit_expr_call(expr_call);
    walk_path(visitor, &mut expr_call.path);
    for arg in &mut expr_call.args {
        walk_expr(visitor, arg);
    }
}

pub fn walk_expr_method_call(visitor: &mut dyn Visitor, expr_method_call: &mut ExprMethodCall) {
    visitor.visit_expr_method_call(expr_method_call);
    walk_expr(visitor, &mut expr_method_call.receiver);
    for arg in &mut expr_method_call.args {
        walk_expr(visitor, arg);
    }
}

pub fn walk_expr(visitor: &mut dyn Visitor, expr: &mut Expr) {
    visitor.visit_expr(expr);
    match &mut expr.kind {
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

pub fn walk_path(visitor: &mut dyn Visitor, path: &mut Path) {
    visitor.visit_path(path);
    for segment in &mut path.segments {
        walk_path_segment(visitor, segment);
    }
}

pub fn walk_path_segment(visitor: &mut dyn Visitor, path_segment: &mut PathSegment) {
    visitor.visit_path_segment(path_segment);
}

pub fn walk_field_value(visitor: &mut dyn Visitor, field_value: &mut FieldValue) {
    visitor.visit_field_value(field_value);
    walk_expr(visitor, &mut field_value.value);
}
