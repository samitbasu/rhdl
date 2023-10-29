// Given an AST, generate a dot representation of it.

use crate::ast;
use crate::ast::*;
use crate::visit::{self, visit_block, Visitor};
use anyhow::Result;

#[derive(Default)]
struct DotGenerator {
    pub dot: String,
}

fn id(node: Option<NodeId>) -> anyhow::Result<u32> {
    match node {
        Some(id) => Ok(id.as_u32()),
        None => Err(anyhow::anyhow!("Node has no id")),
    }
}

pub fn render_dot(block: &mut ast::Block) -> Result<String> {
    let mut dot = DotGenerator::default();
    dot.dot.push_str("digraph {\n");
    visit_block(&mut dot, block)?;
    dot.dot.push_str("}\n");
    Ok(dot.dot)
}

impl Visitor for DotGenerator {
    fn visit_block(&mut self, blk: &Block) -> Result<()> {
        self.dot
            .push_str(&format!("{} [label=\"Block\"];\n", id(blk.id)?));
        for stmt in &blk.stmts {
            self.dot
                .push_str(&format!("{} -> {};\n", id(blk.id)?, id(stmt.id)?));
        }
        visit::visit_block(self, blk)
    }
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        let child = match &stmt.kind {
            StmtKind::Local(local) => {
                self.dot
                    .push_str(&format!("{} [label=\"Stmt::Local\"];\n", id(stmt.id)?));
                local.id
            }
            StmtKind::Expr(expr) => {
                self.dot
                    .push_str(&format!("{} [label=\"Stmt::Expr\"];\n", id(stmt.id)?));
                expr.id
            }
            StmtKind::Semi(expr) => {
                self.dot
                    .push_str(&format!("{} [label=\"Stmt::Semi\"];\n", id(stmt.id)?));
                expr.id
            }
        };
        self.dot
            .push_str(&format!("{} -> {};\n", id(stmt.id)?, id(child)?));
        visit::visit_stmt(self, stmt)
    }
    fn visit_local(&mut self, local: &Local) -> Result<()> {
        self.dot
            .push_str(&format!("{} [label=\"Local\"];\n", id(local.id)?));
        self.dot
            .push_str(&format!("{} -> {};\n", id(local.id)?, id(local.pat.id)?));
        if let Some(init) = &local.init {
            self.dot
                .push_str(&format!("{} -> {};\n", id(local.id)?, id(init.id)?));
        }
        visit::visit_local(self, local)
    }
    fn visit_path(&mut self, path: &Path) -> Result<()> {
        self.dot.push_str(&format!(
            "{} [label=\"{}\"];\n",
            id(path.id)?,
            path.segments
                .iter()
                .map(|x| x.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        ));
        visit::visit_path(self, path)
    }
    fn visit_pat(&mut self, pat: &Pat) -> Result<()> {
        match &pat.kind {
            PatKind::Path(path) => {
                self.dot
                    .push_str(&format!("{} [label=\"Pat::Path\"];\n", id(pat.id)?));
                self.dot
                    .push_str(&format!("{} -> {};\n", id(pat.id)?, id(path.path.id)?));
            }
            _ => {
                self.dot
                    .push_str(&format!("{} [label=\"Pat\"];\n", id(pat.id)?));
            }
        }
        visit::visit_pat(self, pat)
    }
    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        match &expr.kind {
            ExprKind::Binary(bin) => {
                self.dot
                    .push_str(&format!("{} [label=\"{}\"];\n", id(expr.id)?, bin.op));
                self.dot
                    .push_str(&format!("{} -> {};\n", id(expr.id)?, id(bin.lhs.id)?));
                self.dot
                    .push_str(&format!("{} -> {};\n", id(expr.id)?, id(bin.rhs.id)?));
            }
            ExprKind::Unary(un) => {
                self.dot
                    .push_str(&format!("{} [label=\"{}\"];\n", id(expr.id)?, un.op));
                self.dot
                    .push_str(&format!("{} -> {};\n", id(expr.id)?, id(un.expr.id)?));
            }
            ExprKind::If(if_) => {
                self.dot
                    .push_str(&format!("{} [label=\"if\"];\n", id(expr.id)?));
                self.dot.push_str(&format!(
                    "{} -> {} [label=\"cond\"];\n",
                    id(expr.id)?,
                    id(if_.cond.id)?
                ));
                self.dot.push_str(&format!(
                    "{} -> {} [label=\"then\"];\n",
                    id(expr.id)?,
                    id(if_.then_branch.id)?
                ));
                if let Some(else_) = &if_.else_branch {
                    self.dot.push_str(&format!(
                        "{} -> {} [label=\"else\"];\n",
                        id(expr.id)?,
                        id(else_.id)?
                    ));
                }
            }
            ExprKind::Block(blk) => {
                self.dot
                    .push_str(&format!("{} -> {};\n", id(expr.id)?, id(blk.block.id)?));
            }
            ExprKind::Assign(assign) => {
                self.dot
                    .push_str(&format!("{} [label=\"=\"];\n", id(expr.id)?));
                self.dot
                    .push_str(&format!("{} -> {};\n", id(expr.id)?, id(assign.lhs.id)?));
                self.dot
                    .push_str(&format!("{} -> {};\n", id(expr.id)?, id(assign.rhs.id)?));
            }
            _ => {
                self.dot
                    .push_str(&format!("{} [label=\"{}\"];\n", id(expr.id)?, expr));
            }
        }
        visit::visit_expr(self, expr)
    }
}
