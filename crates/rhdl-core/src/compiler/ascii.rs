// Use the visitor pattern to walk an AST and
// print it to a string.  The AST is formatted
// using indented text.

use crate::rhdl_core::ast::ast_impl::*;
use crate::rhdl_core::kernel::Kernel;
use crate::rhdl_core::rhif::spec::Member;
use anyhow::Result;

#[derive(Default)]
pub struct AsciiRenderer {
    indent: usize,
    buffer: String,
}

pub fn render_ast_to_string(kernel: &Kernel) -> Result<String> {
    let mut renderer = AsciiRenderer::default();
    renderer.render(kernel.inner())
}

pub fn render_statement_to_string(stmt: &Stmt) -> Result<String> {
    let mut renderer = AsciiRenderer::default();
    renderer.render_stmt(stmt)?;
    Ok(renderer.buffer)
}

impl AsciiRenderer {
    pub fn render(&mut self, ast: &crate::rhdl_core::ast::ast_impl::KernelFn) -> Result<String> {
        self.render_kernel(ast)?;
        Ok(self.buffer.clone())
    }

    fn push(&mut self, s: &str) {
        let lines = s.split('\n');
        for line in lines {
            self.buffer
                .push_str(&format!("{} ", "  ".repeat(self.indent)));
            self.buffer.push_str(line);
            self.buffer.push('\n');
        }
    }

    fn render_block(&mut self, block: &crate::rhdl_core::ast::ast_impl::Block) -> Result<()> {
        self.push(&format!("block {:?}", block.id));
        self.indent += 1;
        for stmt in &block.stmts {
            self.render_stmt(stmt)?;
        }
        self.indent -= 1;
        Ok(())
    }
    fn render_stmt(&mut self, stmt: &crate::rhdl_core::ast::ast_impl::Stmt) -> Result<()> {
        self.push(&format!("stmt {:?}", stmt.id,));
        self.indent += 1;
        match &stmt.kind {
            StmtKind::Local(local) => {
                self.push(&format!("local {:?}", local.id,));
                self.indent += 1;
                self.render_pat(&local.pat)?;
                if let Some(init) = &local.init {
                    self.push("init");
                    self.render_expr(init)?;
                }
                self.indent -= 1;
            }
            StmtKind::Expr(expr) => {
                self.push("expr");
                self.indent += 1;
                self.render_expr(expr)?;
                self.indent -= 1;
            }
            StmtKind::Semi(expr) => {
                self.push("semi");
                self.indent += 1;
                self.render_expr(expr)?;
                self.indent -= 1;
            }
        }
        self.indent -= 1;
        Ok(())
    }
    fn render_member(&mut self, member: &Member) -> Result<()> {
        match member {
            Member::Named(name) => {
                self.push(&format!("member named {name}"));
            }
            Member::Unnamed(index) => {
                self.push(&format!("member unnamed {index}"));
            }
        }
        Ok(())
    }
    fn render_arm(&mut self, arm: &Arm) -> Result<()> {
        self.push(&format!("arm {:?}", arm.id));
        self.indent += 1;
        match &arm.kind {
            ArmKind::Wild => {
                self.push("_");
            }
            ArmKind::Constant(constant) => {
                self.push(&format!("{:?}", constant.value));
            }
            ArmKind::Enum(enum_arm) => {
                self.render_pat(&enum_arm.pat)?;
                self.push(&format!("#{:?}", &enum_arm.discriminant));
            }
        }
        self.push(" => ");
        self.render_expr(&arm.body)?;
        self.indent -= 1;
        Ok(())
    }
    fn render_expr(&mut self, expr: &Expr) -> Result<()> {
        self.push(&format!("expr {:?}", expr.id,));
        self.indent += 1;

        match &expr.kind {
            ExprKind::Binary(binary) => {
                self.push(&format!("binary {:?}", binary.op));
                self.indent += 1;
                self.render_expr(&binary.lhs)?;
                self.render_expr(&binary.rhs)?;
                self.indent -= 1;
            }
            ExprKind::Unary(unary) => {
                self.push(&format!("unary {:?}", unary.op));
                self.indent += 1;
                self.render_expr(&unary.expr)?;
                self.indent -= 1;
            }
            ExprKind::Array(array) => {
                self.push("array");
                self.indent += 1;
                for expr in &array.elems {
                    self.render_expr(expr)?;
                }
                self.indent -= 1;
            }
            ExprKind::If(if_expr) => {
                self.push("if");
                self.indent += 1;
                self.render_expr(&if_expr.cond)?;
                self.render_block(&if_expr.then_branch)?;
                if let Some(else_expr) = &if_expr.else_branch {
                    self.render_expr(else_expr)?;
                }
                self.indent -= 1;
            }
            ExprKind::Lit(lit) => {
                self.push(&format!("lit {lit:?}"));
            }
            ExprKind::Path(path) => {
                self.push(&format!("path {path:?}"));
            }
            ExprKind::Assign(assign) => {
                self.push("assign");
                self.indent += 1;
                self.render_expr(&assign.lhs)?;
                self.render_expr(&assign.rhs)?;
                self.indent -= 1;
            }
            ExprKind::Call(call) => {
                self.push("call");
                self.indent += 1;

                self.push(&format!("path {:?}", call.path));
                for arg in &call.args {
                    self.render_expr(arg)?;
                }
                self.indent -= 1;
            }
            ExprKind::Index(index) => {
                self.push("index");
                self.indent += 1;
                self.render_expr(&index.expr)?;
                self.render_expr(&index.index)?;
                self.indent -= 1;
            }
            ExprKind::Field(field) => {
                self.push("field");
                self.indent += 1;
                self.render_expr(&field.expr)?;
                self.render_member(&field.member)?;
                self.indent -= 1;
            }
            ExprKind::MethodCall(method_call) => {
                self.push(&format!("method_call {}", method_call.method));
                self.indent += 1;
                self.render_expr(&method_call.receiver)?;
                for arg in &method_call.args {
                    self.render_expr(arg)?;
                }
                self.indent -= 1;
            }
            ExprKind::Block(block) => {
                self.render_block(&block.block)?;
            }
            ExprKind::Paren(paren) => {
                self.render_expr(&paren.expr)?;
            }
            ExprKind::Group(group) => {
                self.render_expr(&group.expr)?;
            }
            ExprKind::Tuple(tuple) => {
                self.push("tuple");
                self.indent += 1;
                for expr in &tuple.elements {
                    self.render_expr(expr)?;
                }
                self.indent -= 1;
            }
            ExprKind::Ret(ret) => {
                self.push("ret");
                self.indent += 1;
                if let Some(expr) = &ret.expr {
                    self.render_expr(expr)?;
                }
                self.indent -= 1;
            }
            ExprKind::Let(let_) => {
                self.push("let");
                self.indent += 1;

                self.render_pat(&let_.pattern)?;
                self.render_expr(&let_.value)?;
                self.indent -= 1;
            }
            ExprKind::Match(match_) => {
                self.push("match");
                self.indent += 1;
                self.render_expr(&match_.expr)?;
                for arm in &match_.arms {
                    self.render_arm(arm)?;
                }
                self.indent -= 1;
            }
            ExprKind::Struct(struct_) => {
                self.push("struct");
                self.indent += 1;
                self.push(&format!("path {:?}", struct_.path));
                for field in &struct_.fields {
                    self.push(&format!("field {:?}", field.member));
                    self.indent += 1;
                    self.render_expr(&field.value)?;
                    self.indent -= 1;
                }
                self.indent -= 1;
            }
            ExprKind::ForLoop(for_loop) => {
                self.push("for_loop");
                self.indent += 1;
                self.render_pat(&for_loop.pat)?;
                self.render_expr(&for_loop.expr)?;
                self.render_block(&for_loop.body)?;
                self.indent -= 1;
            }
            ExprKind::Range(range) => {
                self.push("range");
                self.indent += 1;
                self.push("start");
                if let Some(start) = &range.start {
                    self.render_expr(start)?;
                }
                self.push("end");
                if let Some(end) = &range.end {
                    self.render_expr(end)?;
                }
                self.indent -= 1;
            }
            _ => {
                self.push(&format!("unhandled {:?}", expr.kind));
            }
        }
        self.indent -= 1;
        Ok(())
    }
    fn render_pat(&mut self, pat: &Pat) -> Result<()> {
        self.push(&format!("pat {:?}", pat.id,));
        self.indent += 1;

        match &pat.kind {
            PatKind::Ident(ident) => {
                self.push(&format!("ident {ident:?}"));
            }
            PatKind::Lit(lit) => {
                self.push(&format!("lit {lit:?}"));
            }
            PatKind::Path(path) => {
                self.push(&format!("path {path:?}"));
            }
            PatKind::Struct(struct_) => {
                self.push(&format!("struct {struct_:?}"));
            }
            PatKind::TupleStruct(tuple) => {
                self.push("tuple_struct ");
                self.push(&format!("path {:?}", tuple.path));
                self.indent += 1;
                for pat in &tuple.elems {
                    self.render_pat(pat)?;
                }
                self.indent -= 1;
            }
            PatKind::Tuple(tuple) => {
                self.push(&format!("tuple {tuple:?}"));
            }
            PatKind::Wild => {
                self.push("wild");
            }
            PatKind::Type(ty) => {
                self.push(&format!("type {ty:?}"));
            }
            _ => {
                self.push(&format!("unhandled {:?}", pat.kind));
            }
        }
        self.indent -= 1;
        Ok(())
    }
    fn render_kernel(&mut self, kernel: &KernelFn) -> Result<()> {
        self.push(&format!("kernel {:?}", kernel.id,));
        self.indent += 1;
        for input in &kernel.inputs {
            self.render_pat(input)?;
        }
        self.render_block(&kernel.body)?;
        self.indent -= 1;
        Ok(())
    }
}
