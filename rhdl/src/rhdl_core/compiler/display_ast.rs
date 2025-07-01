use std::fmt::{Display, Formatter};

use crate::rhdl_core::{
    Kind, ast::ast_impl::*, kernel::Kernel, rhif::spec::Member, util::IndentingFormatter,
};
use anyhow::Result;

#[derive(Default)]
pub struct PrettyPrinter {
    buffer: IndentingFormatter,
}

fn _pretty_print_kernel(kernel: &Kernel) -> Result<String> {
    let mut printer = PrettyPrinter {
        buffer: Default::default(),
    };
    printer._print_kernel(kernel)?;
    let buffer = printer.buffer;
    Ok(buffer.buffer())
}

pub fn pretty_print_statement(stmt: &Stmt) -> Result<String> {
    let mut printer = PrettyPrinter {
        buffer: Default::default(),
    };
    printer.print_stmt(stmt)?;
    let buffer = printer.buffer;
    Ok(buffer.buffer())
}

impl PrettyPrinter {
    fn _print_kernel(&mut self, kernel: &Kernel) -> Result<()> {
        self.push(&format!("fn {}(", kernel.inner().name));
        for arg in &kernel.inner().inputs {
            self.print_pattern(arg)?;
            self.push(", ");
        }
        self.push(") -> ");
        self.print_kind(&kernel.inner().ret)?;
        self.push(" ");
        self.print_block(&kernel.inner().body)?;
        Ok(())
    }
    fn push(&mut self, s: &str) {
        self.buffer.write(s);
    }
    fn print_pattern(&mut self, pat: &Pat) -> Result<()> {
        match &pat.kind {
            PatKind::Ident(ident) => {
                self.push(ident.name);
            }
            PatKind::Wild => {
                self.push("_");
            }
            PatKind::Lit(lit) => {
                self.push(&format!("{:?}", lit.lit));
            }
            PatKind::Or(pat) => {
                for segment in &pat.segments {
                    self.print_pattern(segment)?;
                    self.push(" | ");
                }
            }
            PatKind::Paren(pat) => {
                self.push("(");
                self.print_pattern(&pat.pat)?;
                self.push(")");
            }
            PatKind::Path(pat) => {
                self.push(&format!("{}", pat.path));
            }
            PatKind::Slice(pat) => {
                self.push("[");
                for elem in &pat.elems {
                    self.print_pattern(elem)?;
                    self.push(", ");
                }
                self.push("]");
            }
            PatKind::Struct(pat) => {
                self.push(&format!("{}", pat.path));
                self.push(" {");
                for field in &pat.fields {
                    if let Member::Named(name) = &field.member {
                        self.push(&format!("{name}: "));
                    }
                    self.print_pattern(&field.pat)?;
                    self.push(", ");
                }
                self.push("}");
            }
            PatKind::Tuple(pat) => {
                self.push("(");
                for elem in &pat.elements {
                    self.print_pattern(elem)?;
                    self.push(", ");
                }
                self.push(")");
            }
            PatKind::TupleStruct(pat) => {
                self.push(&format!("{}", pat.path));
                self.push("(");
                for elem in &pat.elems {
                    self.print_pattern(elem)?;
                    self.push(", ");
                }
                self.push(")");
            }
            PatKind::Type(pat) => {
                self.print_pattern(&pat.pat)?;
                self.push(": ");
                self.print_kind(&pat.kind)?;
            }
        }
        Ok(())
    }
    fn print_block(&mut self, block: &Block) -> Result<()> {
        self.push("{\n");
        for stmt in &block.stmts {
            self.print_stmt(stmt)?;
        }
        self.push("}\n");
        Ok(())
    }
    fn print_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match &stmt.kind {
            StmtKind::Local(local) => {
                self.push("let ");
                self.print_pattern(&local.pat)?;
                if let Some(init) = &local.init {
                    self.push(" = ");
                    self.print_expr(init)?;
                }
                self.push(";\n");
            }
            StmtKind::Expr(expr) => {
                self.print_expr(expr)?;
                self.push("\n");
            }
            StmtKind::Semi(expr) => {
                self.print_expr(expr)?;
                self.push(";\n");
            }
        }
        Ok(())
    }
    fn print_kind(&mut self, kind: &Kind) -> Result<()> {
        match kind {
            Kind::Empty => self.push("()"),
            Kind::Signed(n) => self.push(&format!("s{n}")),
            Kind::Bits(n) => self.push(&format!("b{n}")),
            Kind::Tuple(kinds) => {
                self.push("(");
                for kind in &kinds.elements {
                    self.print_kind(kind)?;
                    self.push(", ");
                }
                self.push(")");
            }
            Kind::Array(kind) => {
                self.push("[");
                self.print_kind(&kind.base)?;
                self.push("; ");
                self.push(&format!("{}", kind.size));
                self.push("]");
            }
            Kind::Struct(kind) => {
                self.push(&kind.name);
            }
            Kind::Enum(kind) => {
                self.push(&kind.name);
            }
            Kind::Signal(base, color) => {
                self.print_kind(base)?;
                self.push(&format!("@{color:?}"));
            }
        }
        Ok(())
    }
    fn print_expr(&mut self, expr: &Expr) -> Result<()> {
        match &expr.kind {
            ExprKind::Array(expr) => {
                self.push("[");
                for elem in &expr.elems {
                    self.print_expr(elem)?;
                    self.push(", ");
                }
                self.push("]");
            }
            ExprKind::Binary(expr) => {
                self.print_expr(&expr.lhs)?;
                self.push(&format!(" {} ", expr.op));
                self.print_expr(&expr.rhs)?;
            }
            ExprKind::Assign(expr) => {
                self.print_expr(&expr.lhs)?;
                self.push(" = ");
                self.print_expr(&expr.rhs)?;
            }
            ExprKind::Block(expr) => {
                self.print_block(&expr.block)?;
            }
            ExprKind::Call(expr) => {
                self.push(&format!("{}(", expr.path));
                for (ndx, arg) in expr.args.iter().enumerate() {
                    self.print_expr(arg)?;
                    if ndx < expr.args.len() - 1 {
                        self.push(", ");
                    }
                }
                self.push(")");
            }
            ExprKind::Field(expr) => {
                self.print_expr(&expr.expr)?;
                self.push(&format!(".{}", expr.member));
            }
            ExprKind::ForLoop(expr) => {
                self.push("for ");
                self.print_pattern(&expr.pat)?;
                self.push(" in ");
                self.print_expr(&expr.expr)?;
                self.push(" ");
                self.print_block(&expr.body)?;
            }
            ExprKind::Group(expr) => {
                self.print_expr(&expr.expr)?;
            }
            ExprKind::If(expr) => {
                self.push("if ");
                self.print_expr(&expr.cond)?;
                self.push(" ");
                self.print_block(&expr.then_branch)?;
                if let Some(else_branch) = &expr.else_branch {
                    self.push(" else ");
                    self.print_expr(else_branch)?;
                }
            }
            ExprKind::IfLet(expr) => {
                self.push("if let ");
                match &expr.kind {
                    ArmKind::Wild => self.push("_"),
                    ArmKind::Constant(constant) => {
                        self.push(&format!("const {:?}", constant.value))
                    }
                    ArmKind::Enum(enum_arm) => {
                        self.print_pattern(&enum_arm.pat)?;
                        self.push(&format!("#{:?}", enum_arm.discriminant));
                    }
                }
                self.push(" = ");
                self.print_expr(&expr.test)?;
                self.print_block(&expr.then_block)?;
                if let Some(else_branch) = &expr.else_branch {
                    self.push(" else ");
                    self.print_expr(else_branch)?;
                }
            }
            ExprKind::Index(expr) => {
                self.print_expr(&expr.expr)?;
                self.push("[");
                self.print_expr(&expr.index)?;
                self.push("]");
            }
            ExprKind::Let(expr) => {
                self.push("let ");
                self.print_pattern(&expr.pattern)?;
                self.push(" = ");
                self.print_expr(&expr.value)?;
            }
            ExprKind::Lit(expr) => {
                self.push(&format!("{expr:?}"));
            }
            ExprKind::Match(expr) => {
                self.push("match ");
                self.print_expr(&expr.expr)?;
                self.push(" {\n");
                for arm in &expr.arms {
                    match &arm.kind {
                        ArmKind::Wild => self.push("_"),
                        ArmKind::Constant(constant) => {
                            self.push(&format!("const {:?}", constant.value))
                        }
                        ArmKind::Enum(enum_arm) => {
                            self.print_pattern(&enum_arm.pat)?;
                            self.push(&format!("#{:?}", enum_arm.discriminant));
                        }
                    }
                    self.push(" => ");
                    self.print_expr(&arm.body)?;
                    self.push(",\n");
                }
                self.push("}");
            }
            ExprKind::MethodCall(expr) => {
                self.print_expr(&expr.receiver)?;
                self.push(&format!(".{}", expr.method));
                self.push("(");
                for arg in &expr.args {
                    self.print_expr(arg)?;
                    self.push(", ");
                }
                self.push(")");
            }
            ExprKind::Paren(expr) => {
                self.push("(");
                self.print_expr(&expr.expr)?;
                self.push(")");
            }
            ExprKind::Path(expr) => {
                self.push(&format!("{}", expr.path));
            }
            ExprKind::Range(expr) => {
                if let Some(start) = &expr.start {
                    self.print_expr(start)?;
                }
                self.push(&format!("{}", expr.limits));
                if let Some(end) = &expr.end {
                    self.print_expr(end)?;
                }
            }
            ExprKind::Repeat(expr) => {
                self.push("[");
                self.print_expr(&expr.value)?;
                self.push(&format!("; {}]", expr.len));
            }
            ExprKind::Ret(expr) => {
                self.push("return ");
                if let Some(expr) = &expr.expr {
                    self.print_expr(expr)?;
                }
            }
            ExprKind::Struct(expr) => {
                self.push(&format!("{}", expr.path));
                self.push(&format!("/* {} */", expr.template.kind.get_name()));
                self.push(" {");
                for field in &expr.fields {
                    if let Member::Named(name) = &field.member {
                        self.push(&format!("{name}: "));
                    }
                    self.print_expr(&field.value)?;
                    self.push(", ");
                }
                if let Some(rest) = &expr.rest {
                    self.push("..");
                    self.print_expr(rest)?;
                }
                self.push("}");
            }
            ExprKind::Tuple(expr) => {
                self.push("(");
                for elem in &expr.elements {
                    self.print_expr(elem)?;
                    self.push(", ");
                }
                self.push(")");
            }
            ExprKind::Try(tri) => {
                self.print_expr(&tri.expr)?;
                self.push("?");
            }
            ExprKind::Unary(expr) => {
                self.push(&format!("{}", expr.op));
                self.print_expr(&expr.expr)?;
            }
            ExprKind::Type(ty) => self.push(&format!("<{}>", ty.kind.get_name())),
            ExprKind::Bits(bits) => {
                self.push(&format!(
                    "{}(",
                    if matches!(bits.kind, BitsKind::Unsigned) {
                        "bits"
                    } else {
                        "signed"
                    }
                ));
                self.print_expr(&bits.arg)?;
                self.push(")");
            }
            ExprKind::Cast(cast) => {
                self.print_expr(&cast.expr)?;
                self.push(&format!(" as b{}", cast.len));
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ExprLit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprLit::Int(int) => write!(f, "{int}"),
            ExprLit::Bool(bool) => write!(f, "{bool}"),
            ExprLit::TypedBits(ty) => write!(f, "{}", ty.code),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let segments = self
            .segments
            .iter()
            .map(|segment| segment.ident)
            .collect::<Vec<_>>();
        write!(f, "{}", segments.join("::"))
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Ge => write!(f, ">="),
            BinOp::Gt => write!(f, ">"),
            BinOp::AddAssign => write!(f, "+="),
            BinOp::SubAssign => write!(f, "-="),
            BinOp::MulAssign => write!(f, "*="),
            BinOp::BitXorAssign => write!(f, "^="),
            BinOp::BitAndAssign => write!(f, "&="),
            BinOp::BitOrAssign => write!(f, "|="),
            BinOp::ShlAssign => write!(f, "<<="),
            BinOp::ShrAssign => write!(f, ">>="),
        }
    }
}

impl Display for UnOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
        }
    }
}

impl Display for Member {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Member::Named(name) => write!(f, "{name}"),
            Member::Unnamed(index) => write!(f, "{index}"),
        }
    }
}

impl Display for RangeLimits {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeLimits::HalfOpen => write!(f, ".."),
            RangeLimits::Closed => write!(f, "..="),
        }
    }
}
