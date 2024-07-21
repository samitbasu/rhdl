use crate::util::splice;

use super::spec::{
    Assign, Binary, Case, Cast, Concat, DynamicIndex, DynamicSplice, Exec, Index, OpCode, Select,
    Splice, Unary,
};

impl std::fmt::Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                write!(f, " {:?} <- {:?} as b{}", lhs, arg, len)
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                write!(f, " {:?} <- {:?}", lhs, rhs)
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                write!(f, " {:?} <- {:?} as s{}", lhs, arg, len)
            }
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                write!(f, " {:?} <- {:?} {:?} {:?}", lhs, arg1, op, arg2)
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                writeln!(f, " {:?} <- case {:?} {{", lhs, discriminant)?;
                for (cond, val) in table {
                    writeln!(f, "         {:?} => {:?}", cond, val)?;
                }
                write!(f, "}}")
            }
            OpCode::Comment(comment) => {
                write!(f, "// {}", comment)
            }
            OpCode::Concat(Concat { lhs, args }) => {
                write!(f, " {:?} <- {{ {} }}", lhs, splice(args, ", "))
            }
            OpCode::DynamicIndex(DynamicIndex {
                lhs,
                arg,
                offset,
                len,
            }) => {
                write!(f, " {:?} <- {:?}[{:?} +: {:?}]", lhs, arg, offset, len)
            }
            OpCode::DynamicSplice(DynamicSplice {
                lhs,
                arg,
                offset,
                len,
                value,
            }) => {
                write!(
                    f,
                    " {lhs:?} <- {arg:?}; {lhs:?}[{offset:?} +: {len}] <- {value:?}"
                )
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                write!(f, " {:?} <- {:?}({})", lhs, id, splice(args, ", "))
            }
            OpCode::Index(Index {
                lhs,
                arg,
                bit_range,
            }) => {
                write!(f, " {:?} <- {:?}[{:?}]", lhs, arg, bit_range)
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                write!(
                    f,
                    " {:?} <- {:?} ? {:?} : {:?}",
                    lhs, cond, true_value, false_value
                )
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                bit_range,
                value,
            }) => {
                write!(f, " {:?} <- {:?}/{:?}/{:?}", lhs, value, bit_range, orig)
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                write!(f, " {:?} <- {:?}{:?}", lhs, op, arg1)
            }
        }
    }
}
