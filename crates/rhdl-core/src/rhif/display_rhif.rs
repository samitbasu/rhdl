use crate::rhdl_core::{
    rhif::spec::{
        AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec,
        FieldValue, Index, Member, OpCode, Repeat, Splice, Struct, Tuple, Unary,
    },
    util::splice,
};

use super::spec::{Retime, Select, Wrap};

impl std::fmt::Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Noop => write!(f, " NOP"),
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                write!(f, " {lhs} <- {arg1} {op:?} {arg2}")
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                write!(f, " {lhs} <- {op:?}{arg1}")
            }
            OpCode::Array(Array { lhs, elements }) => {
                write!(f, " {:?} <- [{}]", lhs, splice(elements, ", "))
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                write!(f, "{lhs} <- {rhs}")
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                path,
                subst,
            }) => {
                write!(f, "{lhs} <- {orig}/{path:?}/{subst}")
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                write!(f, " {lhs} <- {cond} ? {true_value} : {false_value}")
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                write!(f, " {lhs} <- ({})", splice(fields, ", "))
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                write!(f, " {lhs} <- {arg}{path:?}")
            }
            OpCode::Case(Case {
                lhs,
                discriminant: expr,
                table,
            }) => {
                writeln!(f, " {lhs} <- case {expr} {{")?;
                for (cond, val) in table {
                    writeln!(f, "         {cond:?} => {val}")?;
                }
                writeln!(f, " }}")
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                write!(f, " {lhs} <- {:?}({})", id, splice(args, ", "))
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                write!(
                    f,
                    " {:?} <- {} {{ {} {} }}",
                    lhs,
                    template.kind.get_name(),
                    splice(fields, ", "),
                    rest.map(|x| format!("..{x}")).unwrap_or_default(),
                )
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                write!(f, " {lhs} <- [{value}; {len}]")
            }
            OpCode::Comment(s) => write!(f, " # {}", s.trim_end().replace('\n', "\n   # ")),
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => {
                write!(
                    f,
                    " {lhs} <- {}#{:?}({})",
                    template.kind.get_name(),
                    template.discriminant().unwrap(),
                    splice(fields, ", ")
                )
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                if let Some(len) = len {
                    write!(f, " {lhs} <- {arg} as b{len}")
                } else {
                    write!(f, " {lhs} <- {arg} as bits")
                }
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                if let Some(len) = len {
                    write!(f, " {lhs} <- {arg} as s{len}")
                } else {
                    write!(f, " {lhs} <- {arg} as signed")
                }
            }
            OpCode::Resize(Cast { lhs, arg, len }) => {
                if let Some(len) = len {
                    write!(f, " {lhs} <- {arg}.resize::<{len}>")
                } else {
                    write!(f, " {lhs} <- {arg}.resize")
                }
            }
            OpCode::Retime(Retime { lhs, arg, color }) => {
                write!(f, " {lhs} <- {arg} retime {color:?}")
            }
            OpCode::Wrap(Wrap { op, lhs, arg, kind }) => {
                if let Some(kind) = kind {
                    write!(f, " {lhs} <- {op:?}({arg}) wrapped as {kind:?}")
                } else {
                    write!(f, " {lhs} <- {op:?}({arg})")
                }
            }
        }
    }
}

impl std::fmt::Debug for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.member, self.value)
    }
}

impl std::fmt::Debug for CaseArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseArgument::Slot(s) => write!(f, "{s:?}"),
            CaseArgument::Wild => write!(f, "_"),
        }
    }
}

impl std::fmt::Debug for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Member::Named(s) => write!(f, "{s}"),
            Member::Unnamed(i) => write!(f, "{i}"),
        }
    }
}

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
            AluBinary::XAdd => write!(f, "xadd"),
            AluBinary::XSub => write!(f, "xsub"),
            AluBinary::XMul => write!(f, "xmul"),
        }
    }
}

impl std::fmt::Debug for AluUnary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AluUnary::Neg => write!(f, "-"),
            AluUnary::Not => write!(f, "!"),
            AluUnary::All => write!(f, "&"),
            AluUnary::Any => write!(f, "|"),
            AluUnary::Xor => write!(f, "^"),
            AluUnary::Signed => write!(f, "signed "),
            AluUnary::Unsigned => write!(f, "unsigned "),
            AluUnary::Val => write!(f, "val "),
            AluUnary::XExt(diff) => {
                write!(f, "xext<W{diff}> ")
            }
            AluUnary::XShl(diff) => {
                write!(f, "xshl<W{diff}> ")
            }
            AluUnary::XShr(diff) => {
                write!(f, "xshr<W{diff}> ")
            }
            AluUnary::XNeg => write!(f, "xneg "),
            AluUnary::XSgn => write!(f, "xsgn "),
        }
    }
}
