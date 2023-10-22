// A visitor pattern for the ast

use crate::ast::*;

pub trait Visitor {
    fn visit_block(&mut self, block: &mut Block) {}
    fn visit_stmt(&mut self, stmt: &mut Stmt) {}
    fn visit_stmt_local(&mut self, local: &mut Local) {}
    fn visit_stmt_expr(&mut self, expr: &mut Expr) {}
    fn visit_stmt_semi(&mut self, expr: &mut Expr) {}
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
    fn visit_expr(&mut self, expr: &mut Expr) {}
    fn visit_arm(&mut self, arm: &mut Arm) {}
    fn visit_lit(&mut self, lit: &mut ExprLit) {}
}

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

pub fn walk_pat(mut visitor: impl Visitor, pat: &mut Pat) {
    visitor.visit_pat(pat);
    match &mut pat.kind {
        PatKind::Ident { .. } => {}
        PatKind::Path { path } => {
            walk_path(visitor, path);
        }
        PatKind::Tuple { elements } => {
            for elem in elements {
                walk_pat(visitor, elem);
            }
        }
        PatKind::TupleStruct { path, elems } => {
            visitor.visit_path(path);
            for elem in elems {
                visitor.visit_pat(elem)
            }
            walk_path(visitor, path);
            for elem in elems {
                walk_pat(visitor, elem);
            }
        }
        PatKind::Lit { lit } => {
            visitor.visit_lit(lit);
        }
        PatKind::Struct { path, fields, rest } => {
            visitor.visit_path(path);
            for field in fields {
                walk_pat_field(visitor, field);
            }
        }
        PatKind::Type { pat, kind } => {
            walk_pat(visitor, pat);
        }
    }
}
