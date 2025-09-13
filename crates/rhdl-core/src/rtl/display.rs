use crate::{rtl::spec::CastKind, util::splice};

use super::spec::{
    AluBinary, AluUnary, Assign, Binary, Case, Cast, Concat, Index, OpCode, Select, Splice, Unary,
};

impl std::fmt::Debug for AluBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AluBinary::Add => write!(f, "+"),
            AluBinary::Sub => write!(f, "-"),
            AluBinary::Mul => write!(f, "*"),
            AluBinary::BitAnd => write!(f, "&"),
            AluBinary::BitOr => write!(f, "|"),
            AluBinary::BitXor => write!(f, "^"),
            AluBinary::Shl => write!(f, "<<"),
            AluBinary::Shr => write!(f, ">>"),
            AluBinary::Eq => write!(f, "=="),
            AluBinary::Ne => write!(f, "!="),
            AluBinary::Lt => write!(f, "<"),
            AluBinary::Le => write!(f, "<="),
            AluBinary::Gt => write!(f, ">"),
            AluBinary::Ge => write!(f, ">="),
        }
    }
}

impl std::fmt::Debug for AluUnary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AluUnary::Neg => "-",
                AluUnary::Not => "!",
                AluUnary::All => "&",
                AluUnary::Any => "|",
                AluUnary::Xor => "^",
                AluUnary::Signed => "signed ",
                AluUnary::Unsigned => "unsigned ",
                AluUnary::Val => "val",
            }
        )
    }
}

impl std::fmt::Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Noop => {
                write!(f, "Noop")
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                write!(f, " {lhs} <- {rhs}")
            }
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                write!(f, " {lhs} <- {arg1} {op:?} {arg2}")
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                writeln!(f, " {lhs} <- case {discriminant} {{")?;
                for (cond, val) in table {
                    writeln!(f, "         {cond:?} => {val}")?;
                }
                write!(f, "}}")
            }
            OpCode::Cast(Cast {
                lhs,
                arg,
                len,
                kind,
            }) => {
                write!(
                    f,
                    " {} <- {} as {}{}",
                    lhs,
                    arg,
                    match kind {
                        CastKind::Signed => "s",
                        CastKind::Unsigned => "b",
                        CastKind::Resize => "x",
                    },
                    len
                )
            }
            OpCode::Comment(comment) => {
                for line in comment.lines() {
                    writeln!(f, "// {line}")?;
                }
                Ok(())
            }
            OpCode::Concat(Concat { lhs, args }) => {
                write!(f, " {} <- {{ {} }}", lhs, splice(args, ", "))
            }
            OpCode::Index(Index {
                lhs,
                arg,
                bit_range,
                path,
            }) => {
                write!(f, " {lhs} <- {arg}[{bit_range:?}] // [{path:?}]")
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                write!(f, " {lhs} <- {cond} ? {true_value} : {false_value}")
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                bit_range,
                value,
                path,
            }) => {
                write!(f, " {lhs} <- {orig}/{bit_range:?}/{value} // [{path:?}]")
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                write!(f, " {lhs} <- {op:?}{arg1}")
            }
        }
    }
}
