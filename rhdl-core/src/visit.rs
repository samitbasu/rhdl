// A visitor pattern for the ast

use crate::ast::*;

pub trait Visitor {
    fn visit_block(&mut self, block: &mut Block) {}
    fn visit_stmt(&mut self, stmt: &mut Stmt) {}
    fn visit_local(&mut self, local: &mut Local) {}
    fn visit_pat(&mut self, pat: &mut Pat) {}
    fn visit_path_segment(&mut self, path_segment: &mut PathSegment) {}
    fn visit_path(&mut self, path: &mut Path) {}
    fn visit_pat_ident(&mut self, pat_ident: &mut PatIdent) {}
    fn visit_pat_tuple(&mut self, pat_tuple: &mut PatTuple) {}
    fn visit_pat_tuple_struct(&mut self, pat_tuple_struct: &mut PatTupleStruct) {}
    fn visit_pat_lit(&mut self, pat_lit: &mut PatLit) {}
    fn visit_pat_or(&mut self, pat_or: &mut PatOr) {}
    fn visit_pat_paren(&mut self, pat_paren: &mut PatParen) {}
    fn visit_pat_path(&mut self, pat_path: &mut PatPath) {}
    fn visit_pat_struct(&mut self, pat_struct: &mut PatStruct) {}
    fn visit_pat_type(&mut self, pat_type: &mut PatType) {}
    fn visit_pat_wild(&mut self) {}
    fn visit_expr(&mut self, expr: &mut Expr) {}
    fn visit_expr_binary(&mut self, expr_binary: &mut ExprBinary) {}
    fn visit_expr_unary(&mut self, expr_unary: &mut ExprUnary) {}
    fn visit_expr_match(&mut self, expr_match: &mut ExprMatch) {}
    fn visit_expr_ret(&mut self, expr_return: &mut ExprRet) {}
    fn visit_expr_if(&mut self, expr_if: &mut ExprIf) {}
    fn visit_expr_index(&mut self, expr_index: &mut ExprIndex) {}
    fn visit_expr_paren(&mut self, expr_paren: &mut ExprParen) {}
    fn visit_expr_tuple(&mut self, expr_tuple: &mut ExprTuple) {}
    fn visit_expr_for_loop(&mut self, expr_for_loop: &mut ExprForLoop) {}
    fn visit_expr_assign(&mut self, expr_assign: &mut ExprAssign) {}
    fn visit_expr_group(&mut self, expr_group: &mut ExprGroup) {}
    fn visit_expr_field(&mut self, expr_field: &mut ExprField) {}
    fn visit_expr_block(&mut self, expr_block: &mut ExprBlock) {}
    fn visit_expr_array(&mut self, expr_array: &mut ExprArray) {}
    fn visit_expr_range(&mut self, expr_range: &mut ExprRange) {}
    fn visit_expr_path(&mut self, expr_path: &mut ExprPath) {}
    fn visit_expr_let(&mut self, expr_let: &mut ExprLet) {}
    fn visit_expr_repeat(&mut self, expr_repeat: &mut ExprRepeat) {}
    fn visit_expr_struct(&mut self, expr_struct: &mut ExprStruct) {}
    fn visit_expr_call(&mut self, expr_call: &mut ExprCall) {}
    fn visit_expr_method_call(&mut self, expr_method_call: &mut ExprMethodCall) {}
    fn visit_match_arm(&mut self, arm: &mut Arm) {}
    fn visit_expr_lit(&mut self, lit: &mut ExprLit) {}
}

impl<V: Visitor> Visitor for &mut V {}

pub fn walk_block(mut visitor: impl Visitor, block: &mut Block) {
    visitor.visit_block(block);
    for stmt in &mut block.stmts {
        walk_stmt(visitor, stmt);
    }
}

pub fn walk_stmt(mut visitor: impl Visitor, stmt: &mut Stmt) {
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

pub fn walk_local(mut visitor: impl Visitor, local: &mut Local) {
    visitor.visit_local(local);
    walk_pat(visitor, &mut local.pat);
    if let Some(init) = &mut local.init {
        walk_expr(visitor, init);
    }
}

pub fn walk_pat_ident(mut visitor: impl Visitor, pat_ident: &mut PatIdent) {
    visitor.visit_pat_ident(pat_ident);
}

pub fn walk_pat_tuple(mut visitor: impl Visitor, pat_tuple: &mut PatTuple) {
    visitor.visit_pat_tuple(pat_tuple);
    for pat in &mut pat_tuple.elements {
        walk_pat(visitor, pat);
    }
}

pub fn walk_pat_tuple_struct(mut visitor: impl Visitor, pat_tuple_struct: &mut PatTupleStruct) {
    visitor.visit_pat_tuple_struct(pat_tuple_struct);
    walk_path(visitor, &mut pat_tuple_struct.path);
    for pat in &mut pat_tuple_struct.elems {
        walk_pat(visitor, pat);
    }
}

pub fn walk_pat_lit(mut visitor: impl Visitor, pat_lit: &mut PatLit) {
    visitor.visit_pat_lit(pat_lit);
}

pub fn walk_pat_or(mut visitor: impl Visitor, pat_or: &mut PatOr) {
    visitor.visit_pat_or(pat_or);
    for pat in &mut pat_or.segments {
        walk_pat(visitor, pat);
    }
}

pub fn walk_pat_paren(mut visitor: impl Visitor, pat_paren: &mut PatParen) {
    visitor.visit_pat_paren(pat_paren);
    walk_pat(visitor, &mut pat_paren.pat);
}

pub fn walk_pat_path(mut visitor: impl Visitor, pat_path: &mut PatPath) {
    visitor.visit_pat_path(pat_path);
    walk_path(visitor, &mut pat_path.path);
}

pub fn walk_pat_struct(mut visitor: impl Visitor, pat_struct: &mut PatStruct) {
    visitor.visit_pat_struct(pat_struct);
    walk_path(visitor, &mut pat_struct.path);
    for field in &mut pat_struct.fields {
        walk_field_pat(visitor, field);
    }
}

pub fn walk_pat_type(mut visitor: impl Visitor, pat_type: &mut PatType) {
    visitor.visit_pat_type(pat_type);
    walk_pat(visitor, &mut pat_type.pat);
}

pub fn walk_pat_wild(mut visitor: impl Visitor) {
    visitor.visit_pat_wild();
}

pub fn walk_pat(mut visitor: impl Visitor, pat: &mut Pat) {
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

pub fn walk_expr_binary(mut visitor: impl Visitor, expr_binary: &mut ExprBinary) {
    visitor.visit_expr_binary(expr_binary);
    walk_expr(visitor, &mut expr_binary.lhs);
    walk_expr(visitor, &mut expr_binary.rhs);
}

pub fn walk_expr_unary(mut visitor: impl Visitor, expr_unary: &mut ExprUnary) {
    visitor.visit_expr_unary(expr_unary);
    walk_expr(visitor, &mut expr_unary.expr);
}

pub fn walk_expr_match(mut visitor: impl Visitor, expr_match: &mut ExprMatch) {
    visitor.visit_expr_match(expr_match);
    walk_expr(visitor, &mut expr_match.expr);
    for arm in &mut expr_match.arms {
        walk_match_arm(visitor, arm);
    }
}

pub fn walk_match_arm(mut visitor: impl Visitor, arm: &mut Arm) {
    visitor.visit_match_arm(arm);
    walk_pat(visitor, &mut arm.pattern);
    if let Some(guard) = &mut arm.guard {
        walk_expr(visitor, guard);
    }
    walk_expr(visitor, &mut arm.body);
}

pub fn walk_expr_ret(mut visitor: impl Visitor, expr_return: &mut ExprRet) {
    visitor.visit_expr_ret(expr_return);
    if let Some(expr) = &mut expr_return.expr {
        walk_expr(visitor, expr);
    }
}

pub fn walk_expr(mut visitor: impl Visitor, expr: &mut Expr) {
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
