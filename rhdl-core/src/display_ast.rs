use std::fmt::{Display, Formatter};

use crate::ast::*;

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Local(local) => write!(f, "{}", local),
            Stmt::Expr(expr) => write!(f, "{}", expr),
            Stmt::Semi(expr) => write!(f, "{};", expr),
        }
    }
}

impl Display for ExprStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.expr, self.text)
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for stmt in &self.0 {
            writeln!(f, "{}", stmt)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Display for Local {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.value.as_ref() {
            write!(f, "let {} = {}", self.pattern, value)
        } else {
            write!(f, "let {}", self.pattern)
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Ident(ident) => write!(f, "{}", ident.name),
            Pattern::Tuple(patterns) => {
                write!(f, "(")?;
                for pattern in patterns.iter() {
                    write!(f, "{}, ", pattern)?;
                }
                write!(f, ")")
            }
            Pattern::TupleStruct(pat) => {
                write!(f, "{}", pat.path)?;
                write!(f, "(")?;
                for pattern in &pat.elems {
                    write!(f, "{}, ", pattern)?;
                }
                write!(f, ")")
            }
            Pattern::Lit(lit) => write!(f, "{}", lit),
            Pattern::Paren(pattern) => write!(f, "({})", pattern),
            Pattern::Or(patterns) => {
                write!(f, "(")?;
                for (i, pattern) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", pattern)?;
                }
                write!(f, ")")
            }
            Pattern::Path(path) => write!(f, "{}", path),
            Pattern::Struct(structure) => write!(f, "{}", structure),
            Pattern::Type(type_) => write!(f, "{}", type_),
        }
    }
}

impl Display for PatternType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.pattern, self.kind)
    }
}

impl Display for PatternIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.mutable {
            write!(f, "mut ")?;
        }
        write!(f, "{}", self.name)
    }
}

impl Display for PatternStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{", self.path)?;
        for field in self.fields.iter() {
            write!(f, "{}, ", field)?;
        }
        if self.rest {
            write!(f, "..")?;
        }
        write!(f, "}}")
    }
}

impl Display for FieldPat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.member, self.pat)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary(binary) => write!(f, "{}", binary),
            Expr::Unary(unary) => write!(f, "{}", unary),
            Expr::Match(match_) => write!(f, "{}", match_),
            Expr::Return(None) => write!(f, "return"),
            Expr::Return(Some(expr)) => write!(f, "return {}", expr),
            Expr::If(if_) => write!(f, "{}", if_),
            Expr::Index(index) => write!(f, "{}", index),
            Expr::Lit(lit) => write!(f, "{}", lit),
            Expr::Paren(expr) => write!(f, "({})", expr),
            Expr::Tuple(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            Expr::ForLoop(for_loop) => write!(f, "{}", for_loop),
            Expr::While(while_) => write!(f, "{}", while_),
            Expr::Assign(assign) => write!(f, "{}", assign),
            Expr::Group(expr) => write!(f, "({})", expr),
            Expr::Field(field) => write!(f, "{}", field),
            Expr::Block(block) => write!(f, "{}", block),
            Expr::Array(array) => write!(f, "{}", array),
            Expr::Range(range) => write!(f, "{}", range),
            Expr::Path(path) => write!(f, "<<{}>>", path),
            Expr::Let(let_) => write!(f, "{}", let_),
            Expr::Repeat(repeat) => write!(f, "{}", repeat),
            Expr::Struct(struct_) => write!(f, "{}", struct_),
            Expr::Call(call) => write!(f, "{}", call),
            Expr::MethodCall(method_call) => write!(f, "{}", method_call),
        }
    }
}

impl Display for ExprMethodCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}(", self.receiver, self.method)?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")
    }
}

impl Display for ExprStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{", self.path)?;
        for field in self.fields.iter() {
            write!(f, "{}, ", field)?;
        }
        if let Some(rest) = &self.rest {
            write!(f, "..{}", rest)?;
        }
        write!(f, "}}")
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.member, self.value)
    }
}

impl Display for ExprWhile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "while {} {}", self.cond, self.body)
    }
}

impl Display for ExprRepeat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}; {}]", self.value, self.len)
    }
}

impl Display for ExprCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.path)?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")
    }
}

impl Display for ExprLet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "if let {} = {} {}", self.pattern, self.value, self.body)
    }
}

impl Display for ExprPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.path.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            write!(f, "{}", segment)?;
        }
        Ok(())
    }
}

impl Display for ExprArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for elem in self.elems.iter() {
            write!(f, "{},", elem)?;
        }
        write!(f, "]")
    }
}

impl Display for ExprField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.expr, self.member)
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

impl Display for ExprAssign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.lhs, self.rhs)
    }
}

impl Display for ExprBinary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
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

impl Display for ExprUnary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.op, self.expr)
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

impl Display for ExprIf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} ", self.cond)?;
        write!(f, "{}", self.then_branch)?;
        if let Some(else_branch) = &self.else_branch {
            write!(f, " else {}", else_branch)?;
        }
        Ok(())
    }
}

impl Display for ExprMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "match {} ", self.expr)?;
        write!(f, "{{")?;
        for arm in self.arms.iter() {
            write!(f, "{}", arm)?;
        }
        write!(f, "}}")
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

impl Display for ExprLit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprLit::Int(int) => write!(f, "{}", int),
            ExprLit::Bool(bool) => write!(f, "{}", bool),
        }
    }
}

impl Display for ExprIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.expr, self.index)
    }
}

impl Display for ExprForLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "for {} in {} ", self.pat, self.expr)?;
        write!(f, "{}", self.body)
    }
}

impl Display for ExprRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(start) = &self.start {
            write!(f, "{}", start)?;
        }
        write!(f, "{}", self.limits)?;
        if let Some(end) = &self.end {
            write!(f, "{}", end)?;
        }
        Ok(())
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
