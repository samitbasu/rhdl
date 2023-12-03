use std::fmt::{Display, Formatter};

use crate::{
    ast::*,
    infer_types::id_to_var,
    kernel::Kernel,
    ty::{self, Ty},
    unify::UnifyContext,
    util::{splice, IndentingFormatter},
    Kind,
};
use anyhow::Result;

pub struct PrettyPrinter<'a> {
    buffer: IndentingFormatter,
    ty: &'a UnifyContext,
}

pub fn pretty_print_kernel(kernel: &Kernel, ty: &UnifyContext) -> Result<String> {
    let mut printer = PrettyPrinter {
        buffer: Default::default(),
        ty,
    };
    printer.print_kernel(kernel)?;
    let buffer = printer.buffer;
    Ok(buffer.buffer())
}

pub fn pretty_print_statement(stmt: &Stmt, ty: &UnifyContext) -> Result<String> {
    let mut printer = PrettyPrinter {
        buffer: Default::default(),
        ty,
    };
    printer.print_stmt(stmt)?;
    let buffer = printer.buffer;
    Ok(buffer.buffer())
}

impl<'a> PrettyPrinter<'a> {
    pub fn print_kernel(&mut self, kernel: &Kernel) -> Result<()> {
        self.push(&format!("fn {}(", kernel.ast.name));
        for arg in &kernel.ast.inputs {
            self.print_pattern(arg)?;
            self.push(", ");
        }
        self.push(") -> ");
        self.print_kind(&kernel.ast.ret)?;
        self.push(" ");
        self.print_block(&kernel.ast.body)?;
        Ok(())
    }
    fn push(&mut self, s: &str) {
        self.buffer.write(s);
    }
    fn print_pattern(&mut self, pat: &Pat) -> Result<()> {
        let term = self.ty.apply(id_to_var(pat.id)?);
        match &pat.kind {
            PatKind::Ident(ident) => {
                self.push(&ident.name);
            }
            PatKind::Wild => {
                self.push("_");
            }
            PatKind::Lit(lit) => {
                self.push(&format!("{}", lit.lit));
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
                        self.push(&format!("{}: ", name));
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
            PatKind::Const(pat) => {
                self.push(&format!("const {}: ", pat.name));
                self.print_type(&term)?;
                self.push(&format!("{}", pat.lit));
            }
        }
        self.push(" /* ");
        self.print_type(&term)?;
        self.push(" */");
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
            Kind::Signed(n) => self.push(&format!("s{}", n)),
            Kind::Bits(n) => self.push(&format!("b{}", n)),
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
            _ => todo!(),
        }
        Ok(())
    }
    fn print_type(&mut self, term: &Ty) -> Result<()> {
        match term {
            Ty::Var(_var) => {
                self.push("??");
            }
            Ty::Array(ty) => {
                self.push("[");
                for t in ty {
                    self.print_type(t)?;
                    self.push(", ");
                }
                self.push("]");
            }
            Ty::Const(ty) => match ty {
                ty::Bits::Empty => self.push("()"),
                ty::Bits::Signed(n) => self.push(&format!("s{}", n)),
                ty::Bits::Unsigned(n) => self.push(&format!("b{}", n)),
                ty::Bits::I128 => self.push("i128"),
                ty::Bits::U128 => self.push("u128"),
                ty::Bits::Usize => self.push("usize"),
            },
            Ty::Integer => self.push("int"),
            Ty::Enum(ty) | Ty::Struct(ty) => {
                self.push(&ty.name);
            }
            Ty::Ref(_) => todo!(),
            Ty::Tuple(ty) => {
                self.push("(");
                for t in ty {
                    self.print_type(t)?;
                    self.push(", ");
                }
                self.push(")");
            }
        }
        Ok(())
    }
    fn print_expr(&mut self, expr: &Expr) -> Result<()> {
        let my_id = id_to_var(expr.id)?;
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
                let term = self.ty.apply(id_to_var(expr.lhs.id)?);
                self.print_expr(&expr.lhs)?;
                self.push(" /*");
                self.print_type(&term)?;
                self.push("*/");
                self.push(" = ");
                self.print_expr(&expr.rhs)?;
            }
            ExprKind::Block(expr) => {
                self.print_block(&expr.block)?;
            }
            ExprKind::Call(expr) => {
                self.push(&format!("{}", expr.path));
                let term = self.ty.apply(my_id);
                self.push("<");
                self.print_type(&term)?;
                self.push(">(");
                for arg in &expr.args {
                    self.print_expr(arg)?;
                    self.push(", ");
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
                self.push(&format!("{}", expr));
            }
            ExprKind::Match(expr) => {
                self.push("match ");
                self.print_expr(&expr.expr)?;
                self.push(" {\n");
                for arm in &expr.arms {
                    self.print_pattern(&arm.pattern)?;
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
                self.push("; ");
                self.print_expr(&expr.len)?;
                self.push("]");
            }
            ExprKind::Ret(expr) => {
                self.push("return ");
                if let Some(expr) = &expr.expr {
                    self.print_expr(expr)?;
                }
            }
            ExprKind::Struct(expr) => {
                self.push(&format!("{}", expr.path));
                self.push(&format!("/* {} */", expr.kind.get_name()));
                self.push(" {");
                for field in &expr.fields {
                    if let Member::Named(name) = &field.member {
                        self.push(&format!("{}: ", name));
                    }
                    self.print_expr(&field.value)?;
                    self.push(", ");
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
            ExprKind::Unary(expr) => {
                self.push(&format!("{}", expr.op));
                self.print_expr(&expr.expr)?;
            }
            ExprKind::Type(ty) => self.push(&format!("<{}>", ty.kind.get_name())),
        }
        Ok(())
    }
}
/*
impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl Display for StmtKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StmtKind::Local(local) => write!(f, "{}", local),
            StmtKind::Expr(expr) => write!(f, "{}", expr),
            StmtKind::Semi(expr) => write!(f, "{};", expr),
        }
    }
}

fn id(node: Option<NodeId>) -> String {
    if let Some(id) = node {
        format!("<{}>", id.as_u32())
    } else {
        String::new()
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{ {}", id(self.id))?;
        for stmt in &self.stmts {
            writeln!(f, "{}{}", id(stmt.id), stmt)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Display for Local {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(init) = self.init.as_ref() {
            write!(f, "let{} {} = {}", id(self.id), self.pat, init)
        } else {
            write!(f, "let{} {}", id(self.id), self.pat)
        }
    }
}

impl Display for Pat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", id(self.id), self.kind)
    }
}

impl Display for PatKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PatKind::Ident(PatIdent { name, mutable }) => {
                if *mutable {
                    write!(f, "mut ")?;
                }
                write!(f, "{}", name)
            }
            PatKind::Tuple(PatTuple { elements }) => {
                write!(f, "({})", splice(elements.as_slice(), ", "))
            }
            PatKind::Slice(PatSlice { elems }) => {
                write!(f, "[{}]", splice(elems.as_slice(), ", "))
            }
            PatKind::TupleStruct(PatTupleStruct { path, elems }) => {
                write!(f, "{}({})", path, splice(elems.as_slice(), ", "))
            }
            PatKind::Struct(PatStruct { path, fields, rest }) => {
                write!(f, "{} {{{}", path, splice(fields.as_slice(), ", "))?;
                if *rest {
                    write!(f, "..")?;
                }
                write!(f, "}}")
            }
            PatKind::Lit(PatLit { lit }) => write!(f, "{}", lit),
            PatKind::Or(PatOr { segments }) => {
                write!(f, "{}", splice(segments.as_slice(), " | "))
            }
            PatKind::Paren(PatParen { pat }) => write!(f, "({})", pat),
            PatKind::Path(PatPath { path }) => write!(f, "{}", path),
            PatKind::Type(PatType { pat, kind }) => write!(f, "{}: {:?}", pat, kind),
            PatKind::Wild => write!(f, "_"),
        }
    }
}

impl Display for FieldPat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.member, self.pat)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", id(self.id), self.kind)
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprKind::Binary(ExprBinary { op, lhs, rhs }) => write!(f, "{} {} {}", lhs, op, rhs),
            ExprKind::Unary(ExprUnary { op, expr }) => write!(f, "{}{}", op, expr),
            ExprKind::Match(ExprMatch { expr, arms }) => {
                write!(f, "match {} {{{}}}", expr, splice(arms.as_slice(), ""))
            }
            ExprKind::Ret(ExprRet { expr }) => {
                write!(f, "return ")?;
                if let Some(expr) = expr {
                    write!(f, "{}", expr)?;
                }
                Ok(())
            }
            ExprKind::If(ExprIf {
                cond,
                then_branch,
                else_branch,
            }) => {
                write!(f, "if {} ", cond)?;
                write!(f, "{}", then_branch)?;
                if let Some(else_branch) = else_branch {
                    write!(f, " else {}", else_branch)?;
                }
                Ok(())
            }
            ExprKind::Index(ExprIndex { expr, index }) => write!(f, "{}[{}]", expr, index),
            ExprKind::Lit(lit) => write!(f, "{}", lit),
            ExprKind::Paren(ExprParen { expr }) => write!(f, "({})", expr),
            ExprKind::Tuple(ExprTuple { elements }) => {
                write!(f, "({})", splice(elements.as_slice(), ", "))
            }
            ExprKind::ForLoop(ExprForLoop { pat, expr, body }) => {
                write!(f, "for {} in {} ", pat, expr)?;
                write!(f, "{}", body)
            }
            ExprKind::Assign(ExprAssign { lhs, rhs }) => {
                write!(f, "{} = {}", lhs, rhs)
            }
            ExprKind::Group(ExprGroup { expr }) => {
                write!(f, "({})", expr)
            }
            ExprKind::Field(ExprField { expr, member }) => {
                write!(f, "{}.{}", expr, member)
            }
            ExprKind::Block(ExprBlock { block }) => {
                write!(f, "{}", block)
            }
            ExprKind::Array(ExprArray { elems }) => {
                write!(f, "[{}]", splice(elems.as_slice(), ", "))
            }
            ExprKind::Range(ExprRange { start, limits, end }) => {
                if let Some(start) = start {
                    write!(f, "{}", start)?;
                }
                write!(f, "{}", limits)?;
                if let Some(end) = end {
                    write!(f, "{}", end)?;
                }
                Ok(())
            }
            ExprKind::Path(ExprPath { path }) => {
                write!(f, "{}", path)
            }
            ExprKind::Let(ExprLet { pattern, value }) => {
                write!(f, "let {} = {}", pattern, value)
            }
            ExprKind::Repeat(ExprRepeat { value, len }) => {
                write!(f, "[{}; {}]", value, len)
            }
            ExprKind::Struct(ExprStruct { path, fields, rest }) => {
                write!(f, "{} {{{}", path, splice(fields.as_slice(), ", "))?;
                if let Some(rest) = rest {
                    write!(f, ".. {}", rest)?;
                }
                write!(f, "}}")
            }
            ExprKind::Call(ExprCall { path, args }) => {
                write!(f, "{}({})", path, splice(args.as_slice(), ", "))
            }
            ExprKind::MethodCall(ExprMethodCall {
                receiver,
                method,
                args,
            }) => {
                write!(
                    f,
                    "{}.{}({})",
                    receiver,
                    method,
                    splice(args.as_slice(), ", ")
                )
            }
        }
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.member, self.value)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", splice(&self.segments, "::"))
    }
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)?;
        if !self.arguments.is_empty() {
            write!(f, "<{}>", splice(&self.arguments, ", "))?;
        }
        Ok(())
    }
}



impl Display for Arm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pattern)?;
        if let Some(guard) = &self.guard {
            write!(f, " if {}", guard)?;
        }
        write!(f, " => {}", self.body)
    }
}



impl Display for KernelFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}(", self.name)?;
        write!(f, "{}", splice(&self.inputs, ", "))?;
        write!(f, ") -> {:?}", self.ret)?;
        write!(f, "{}", self.body)
    }
}
*/

impl Display for ExprLit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprLit::Int(int) => write!(f, "{}", int),
            ExprLit::Bool(bool) => write!(f, "{}", bool),
            ExprLit::TypedBits(ty) => write!(f, "<typed_bits {}>", ty.path),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let segments = self
            .segments
            .iter()
            .map(|segment| segment.ident.as_str())
            .collect::<Vec<_>>();
        write!(f, "{}", splice(&segments, "::"))
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
            Member::Named(name) => write!(f, "{}", name),
            Member::Unnamed(index) => write!(f, "{}", index),
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
