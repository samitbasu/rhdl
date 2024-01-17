use std::fmt::Display;

use crate::{
    rhif::spec::{
        AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Discriminant, Enum,
        Exec, FieldValue, FuncId, Index, Member, OpCode, Repeat, Slot, Splice, Struct, Tuple,
        Unary,
    },
    util::splice,
};

use super::spec::Select;

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                write!(f, " {} <- {} {} {}", lhs, arg1, op, arg2)
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                write!(f, " {} <- {}{}", lhs, op, arg1)
            }
            OpCode::Array(Array { lhs, elements }) => {
                write!(f, " {} <- [{}]", lhs, splice(elements, ", "))
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                write!(f, "{} <- {}", lhs, rhs)
            }
            OpCode::Splice(Splice {
                lhs,
                orig: rhs,
                path,
                subst: arg,
            }) => {
                write!(f, "{} <- {}/[{}]{}", lhs, path, rhs, arg)
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                write!(f, " {} <- {} ? {} : {}", lhs, cond, true_value, false_value)
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                write!(f, " {} <- ({})", lhs, splice(fields, ", "))
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                write!(f, " {} <- {}{}", lhs, arg, path)
            }
            OpCode::Case(Case {
                lhs,
                discriminant: expr,
                table,
            }) => {
                writeln!(f, " {} <- case {} {{", lhs, expr)?;
                for (cond, val) in table {
                    writeln!(f, "         {} => {}", cond, val)?;
                }
                writeln!(f, " }}")
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                write!(f, " {} <- {}({})", lhs, id, splice(args, ", "))
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                write!(
                    f,
                    " {} <- {} {{ {} {} }}",
                    lhs,
                    template.kind.get_name(),
                    splice(fields, ", "),
                    rest.map(|x| format!("..{}", x)).unwrap_or_default(),
                )
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                write!(f, " {} <- [{}; {}]", lhs, value, len)
            }
            OpCode::Comment(s) => write!(f, " # {}", s.trim_end().replace('\n', "\n   # ")),
            OpCode::Discriminant(Discriminant { lhs, arg }) => write!(f, " {} <- #[{}]", lhs, arg),
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => {
                write!(
                    f,
                    " {} <- {}#{}({})",
                    lhs,
                    template.kind.get_name(),
                    template.discriminant().unwrap(),
                    splice(fields, ", ")
                )
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                write!(f, " {} <- {} as b{}", lhs, arg, len)
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                write!(f, " {} <- {} as s{}", lhs, arg, len)
            }
        }
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.member, self.value)
    }
}

impl Display for CaseArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseArgument::Constant(c) => write!(f, "{}", c),
            CaseArgument::Wild => write!(f, "_"),
        }
    }
}

impl Display for FuncId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.0)
    }
}

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Member::Named(s) => write!(f, "{}", s),
            Member::Unnamed(i) => write!(f, "{}", i),
        }
    }
}

impl Display for AluBinary {
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

impl Display for AluUnary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AluUnary::Neg => write!(f, "-"),
            AluUnary::Not => write!(f, "!"),
            AluUnary::All => write!(f, "&"),
            AluUnary::Any => write!(f, "|"),
            AluUnary::Xor => write!(f, "^"),
            AluUnary::Signed => write!(f, "signed "),
            AluUnary::Unsigned => write!(f, "unsigned "),
        }
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Literal(l) => write!(f, "l{}", l),
            // Use 4 spaces for alignment
            Slot::Register(usize) => write!(f, "r{}", usize),
            Slot::Empty => write!(f, "()"),
        }
    }
}
