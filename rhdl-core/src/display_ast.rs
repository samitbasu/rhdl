use std::fmt::{Display, Formatter};

use crate::ast::*;

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

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for stmt in &self.stmts {
            writeln!(f, "{}", stmt)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Display for Local {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(init) = self.init.as_ref() {
            write!(f, "let {} = {}", self.pat, init)
        } else {
            write!(f, "let {}", self.pat)
        }
    }
}

impl Display for Pat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

fn splice<T: Display>(elems: &[T], sep: &str) -> String {
    elems
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}

impl Display for PatKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PatKind::Ident { name, mutable } => {
                if *mutable {
                    write!(f, "mut ")?;
                }
                write!(f, "{}", name)
            }
            PatKind::Tuple { elements } => {
                write!(f, "({})", splice(elements.as_slice(), ", "))
            }
            PatKind::TupleStruct { path, elems } => {
                write!(f, "{}({})", path, splice(elems.as_slice(), ", "))
            }
            PatKind::Struct { path, fields, rest } => {
                write!(f, "{} {{{}", path, splice(fields.as_slice(), ", "))?;
                if *rest {
                    write!(f, "..")?;
                }
                write!(f, "}}")
            }
            PatKind::Lit { lit } => write!(f, "{}", lit),
            PatKind::Or { segments } => {
                write!(f, "{}", splice(segments.as_slice(), " | "))
            }
            PatKind::Paren { pat } => write!(f, "({})", pat),
            PatKind::Path { path } => write!(f, "{}", path),
            PatKind::Type { pat, kind } => write!(f, "{}: {:?}", pat, kind),
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
        write!(f, "{}", self.kind)
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprKind::Binary { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
            ExprKind::Unary { op, expr } => write!(f, "{}{}", op, expr),
            ExprKind::Match { expr, arms } => {
                write!(f, "match {} {{{}}}", expr, splice(arms.as_slice(), ""))
            }
            ExprKind::Ret { expr } => {
                write!(f, "return ")?;
                if let Some(expr) = expr {
                    write!(f, "{}", expr)?;
                }
                Ok(())
            }
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                write!(f, "if {} ", cond)?;
                write!(f, "{}", then_branch)?;
                if let Some(else_branch) = else_branch {
                    write!(f, " else {}", else_branch)?;
                }
                Ok(())
            }
            ExprKind::Index { expr, index } => write!(f, "{}[{}]", expr, index),
            ExprKind::Lit { lit } => write!(f, "{}", lit),
            ExprKind::Paren { expr } => write!(f, "({})", expr),
            ExprKind::Tuple { elements } => write!(f, "({})", splice(elements.as_slice(), ", ")),
            ExprKind::ForLoop { pat, expr, body } => {
                write!(f, "for {} in {} ", pat, expr)?;
                write!(f, "{}", body)
            }
            ExprKind::Assign { lhs, rhs } => {
                write!(f, "{} = {}", lhs, rhs)
            }
            ExprKind::Group { expr } => {
                write!(f, "({})", expr)
            }
            ExprKind::Field { expr, member } => {
                write!(f, "{}.{}", expr, member)
            }
            ExprKind::Block { block } => {
                write!(f, "{}", block)
            }
            ExprKind::Array { elems } => {
                write!(f, "[{}]", splice(elems.as_slice(), ", "))
            }
            ExprKind::Range { start, limits, end } => {
                if let Some(start) = start {
                    write!(f, "{}", start)?;
                }
                write!(f, "{}", limits)?;
                if let Some(end) = end {
                    write!(f, "{}", end)?;
                }
                Ok(())
            }
            ExprKind::Path { path } => {
                write!(f, "{}", path)
            }
            ExprKind::Let {
                pattern,
                value,
                body,
            } => {
                write!(f, "let {} = {} {}", pattern, value, body)
            }
            ExprKind::Repeat { value, len } => {
                write!(f, "[{}; {}]", value, len)
            }
            ExprKind::Struct { path, fields, rest } => {
                write!(f, "{} {{{}", path, splice(fields.as_slice(), ", "))?;
                if let Some(rest) = rest {
                    write!(f, ".. {}", rest)?;
                }
                write!(f, "}}")
            }
            ExprKind::Call { path, args } => {
                write!(f, "{}({})", path, splice(args.as_slice(), ", "))
            }
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
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
        write!(f, "{}", self.ident)
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

impl Display for RangeLimits {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeLimits::HalfOpen => write!(f, ".."),
            RangeLimits::Closed => write!(f, "..="),
        }
    }
}
