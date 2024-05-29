use crate::{
    ast::ast_impl::{
        ArmKind, BitsKind, Block, Expr, ExprKind, KernelFn, NodeId, Pat, PatKind, Stmt, StmtKind,
    },
    util::IndentingFormatter,
    Kind,
};
use std::{collections::HashMap, ops::Range};

use super::spec::Member;

#[derive(Clone, Debug)]
pub struct SpannedSource {
    pub source: String,
    pub name: String,
    pub span_map: HashMap<NodeId, Range<usize>>,
}

impl SpannedSource {
    pub fn span(&self, id: NodeId) -> Range<usize> {
        self.span_map[&id].clone()
    }
    pub fn text(&self, id: NodeId) -> &str {
        let span = self.span(id);
        &self.source[span]
    }
    pub fn snippet(&self, id: NodeId) -> &str {
        let span = self.span(id);
        &self.source[span]
    }
}

#[derive(Default)]
struct SpannedSourceBuilder {
    name: String,
    buffer: IndentingFormatter,
    span_map: HashMap<NodeId, Range<usize>>,
}

pub fn build_spanned_source_for_kernel(kernel: &KernelFn) -> SpannedSource {
    let mut builder = SpannedSourceBuilder::default();
    builder.kernel(kernel);
    builder.build()
}

impl SpannedSourceBuilder {
    fn build(self) -> SpannedSource {
        SpannedSource {
            name: self.name,
            source: self.buffer.buffer(),
            span_map: self.span_map,
        }
    }

    fn loc(&self) -> usize {
        self.buffer.location()
    }

    fn push(&mut self, s: &str) {
        self.buffer.write(s);
    }

    fn kernel(&mut self, kernel: &KernelFn) {
        let start = self.loc();
        self.push(&format!("fn {}(", kernel.name));
        for arg in &kernel.inputs {
            self.pattern(arg);
            self.push(", ");
        }
        self.push(") -> ");
        self.kind(&kernel.ret);
        self.push(" ");
        self.block(&kernel.body);
        self.span_map.insert(kernel.id, start..self.loc());
        self.name = kernel.name.into();
    }

    fn pattern(&mut self, pat: &Pat) {
        let start = self.loc();
        match &pat.kind {
            PatKind::Ident(ident) => {
                self.push(ident.name);
            }
            PatKind::Wild => {
                self.push("_");
            }
            PatKind::Lit(lit) => {
                self.push(&format!("{}", lit.lit));
            }
            PatKind::Or(pat) => {
                for segment in &pat.segments {
                    self.pattern(segment);
                    self.push(" | ");
                }
            }
            PatKind::Paren(pat) => {
                self.push("(");
                self.pattern(&pat.pat);
                self.push(")");
            }
            PatKind::Path(pat) => {
                self.push(&format!("{}", pat.path));
            }
            PatKind::Slice(pat) => {
                self.push("[");
                for elem in &pat.elems {
                    self.pattern(elem);
                    self.push(", ");
                }
                self.push("]");
            }
            PatKind::Struct(pat) => {
                self.push(&format!("{}", pat.path));
                self.push(" {");
                for field in &pat.fields {
                    if let Member::Named(name) = &field.member {
                        self.push(&format!("{}: ", name));
                    }
                    self.pattern(&field.pat);
                    self.push(", ");
                }
                self.push("}");
            }
            PatKind::Tuple(pat) => {
                self.push("(");
                for elem in &pat.elements {
                    self.pattern(elem);
                    self.push(", ");
                }
                self.push(")");
            }
            PatKind::TupleStruct(pat) => {
                self.push(&format!("{}", pat.path));
                self.push("(");
                for elem in &pat.elems {
                    self.pattern(elem);
                    self.push(", ");
                }
                self.push(")");
            }
            PatKind::Type(pat) => {
                self.pattern(&pat.pat);
                self.push(": ");
                self.kind(&pat.kind);
            }
        }
        self.span_map.insert(pat.id, start..self.loc());
    }

    fn block(&mut self, block: &Block) {
        let start = self.loc();
        self.push("{\n");
        for stmt in &block.stmts {
            self.stmt(stmt);
        }
        self.push("}\n");
        self.span_map.insert(block.id, start..self.loc());
    }

    fn stmt(&mut self, stmt: &Stmt) {
        let start = self.loc();
        match &stmt.kind {
            StmtKind::Local(local) => {
                self.push("let ");
                self.pattern(&local.pat);
                if let Some(init) = &local.init {
                    self.push(" = ");
                    self.expr(init);
                }
                self.push(";\n");
            }
            StmtKind::Expr(expr) => {
                self.expr(expr);
                self.push("\n");
            }
            StmtKind::Semi(expr) => {
                self.expr(expr);
                self.push(";\n");
            }
        }
        self.span_map.insert(stmt.id, start..self.loc());
    }

    fn kind(&mut self, kind: &Kind) {
        match kind {
            Kind::Empty => self.push("()"),
            Kind::Signed(n) => self.push(&format!("s{}", n)),
            Kind::Bits(n) => self.push(&format!("b{}", n)),
            Kind::Tuple(kinds) => {
                self.push("(");
                for kind in &kinds.elements {
                    self.kind(kind);
                    self.push(", ");
                }
                self.push(")");
            }
            Kind::Array(kind) => {
                self.push("[");
                self.kind(&kind.base);
                self.push("; ");
                self.push(&format!("{}", kind.size));
                self.push("]");
            }
            Kind::Struct(kind) => self.push(&kind.name),
            Kind::Enum(kind) => self.push(&kind.name),
            Kind::Signal(base, color) => {
                self.push("Signal<");
                self.kind(base);
                self.push(", ");
                self.push(&format!("{:?}", color));
                self.push(">");
            }
        }
    }

    fn expr(&mut self, expr: &Expr) {
        let start = self.loc();
        match &expr.kind {
            ExprKind::Array(expr) => {
                self.push("[");
                for elem in &expr.elems {
                    self.expr(elem);
                    self.push(", ");
                }
                self.push("]");
            }
            ExprKind::Binary(expr) => {
                self.expr(&expr.lhs);
                self.push(&format!(" {} ", expr.op));
                self.expr(&expr.rhs);
            }
            ExprKind::Assign(expr) => {
                self.expr(&expr.lhs);
                self.push(" = ");
                self.expr(&expr.rhs);
            }
            ExprKind::Block(expr) => {
                self.block(&expr.block);
            }
            ExprKind::Call(expr) => {
                self.push(&format!("{}", expr.path));
                self.push("(");
                for arg in &expr.args {
                    self.expr(arg);
                    self.push(", ");
                }
                self.push(")");
            }
            ExprKind::Field(expr) => {
                self.expr(&expr.expr);
                self.push(&format!(".{:?}", expr.member));
            }
            ExprKind::ForLoop(expr) => {
                self.push("for ");
                self.pattern(&expr.pat);
                self.push(" in ");
                self.expr(&expr.expr);
                self.push(" ");
                self.block(&expr.body);
            }
            ExprKind::Group(expr) => {
                self.expr(&expr.expr);
            }
            ExprKind::If(expr) => {
                self.push("if ");
                self.expr(&expr.cond);
                self.push(" ");
                self.block(&expr.then_branch);
                if let Some(else_) = &expr.else_branch {
                    self.push("else ");
                    self.expr(else_);
                }
            }
            ExprKind::Index(expr) => {
                self.expr(&expr.expr);
                self.push("[");
                self.expr(&expr.index);
                self.push("]");
            }
            ExprKind::Let(expr) => {
                self.push("let ");
                self.pattern(&expr.pattern);
                self.push(" = ");
                self.expr(&expr.value);
            }
            ExprKind::Lit(expr) => {
                self.push(&format!("{}", expr));
            }
            ExprKind::Match(expr) => {
                self.push("match ");
                self.expr(&expr.expr);
                self.push(" {\n");
                for arm in &expr.arms {
                    match &arm.kind {
                        ArmKind::Wild => self.push("_"),
                        ArmKind::Constant(constant) => {
                            self.push(&format!("{}", constant.value));
                        }
                        ArmKind::Enum(enum_arm) => {
                            self.pattern(&enum_arm.pat);
                        }
                    }
                    self.push(" => ");
                    self.expr(&arm.body);
                    self.push(",\n");
                }
                self.push("}");
            }
            ExprKind::MethodCall(expr) => {
                self.expr(&expr.receiver);
                self.push(&format!(".{}", expr.method));
                self.push("(");
                for arg in &expr.args {
                    self.expr(arg);
                    self.push(", ");
                }
                self.push(")");
            }
            ExprKind::Paren(expr) => {
                self.push("(");
                self.expr(&expr.expr);
                self.push(")");
            }
            ExprKind::Path(expr) => {
                self.push(&format!("{}", expr.path));
            }
            ExprKind::Range(expr) => {
                if let Some(start) = &expr.start {
                    self.expr(start);
                }
                self.push(" .. ");
                if let Some(end) = &expr.end {
                    self.expr(end);
                }
            }
            ExprKind::Repeat(expr) => {
                self.push("[");
                self.expr(&expr.value);
                self.push(&format!("; {}]", expr.len));
            }
            ExprKind::Ret(expr) => {
                self.push("return ");
                if let Some(expr) = &expr.expr {
                    self.expr(expr);
                }
            }
            ExprKind::Struct(expr) => {
                self.push(&format!("{}", expr.path));
                self.push(" {");
                for field in &expr.fields {
                    if let Member::Named(name) = &field.member {
                        self.push(&format!("{}: ", name));
                    }
                    self.expr(&field.value);
                    self.push(", ");
                }
                if let Some(rest) = &expr.rest {
                    self.push("..");
                    self.expr(rest);
                }
                self.push("}");
            }
            ExprKind::Tuple(expr) => {
                self.push("(");
                for elem in &expr.elements {
                    self.expr(elem);
                    self.push(", ");
                }
                self.push(")");
            }
            ExprKind::Unary(expr) => {
                self.push(&format!("{}", expr.op));
                self.expr(&expr.expr);
            }
            ExprKind::Type(expr) => {
                self.kind(&expr.kind);
            }
            ExprKind::Bits(bits) => {
                let func_name = match bits.kind {
                    BitsKind::Unsigned => "bits",
                    BitsKind::Signed => "signed",
                };
                self.push(&format!("{}(", func_name));
                self.expr(&bits.arg);
                self.push(")");
            }
        }
        self.span_map.insert(expr.id, start..self.loc());
        eprintln!("Span for expression {:?}: {:?}", expr.id, start..self.loc());
    }
}
