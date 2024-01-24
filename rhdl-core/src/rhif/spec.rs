// RHDL Intermediate Form (RHIF).
use anyhow::Result;

use crate::{path::Path, DigitalSignature, KernelFnKind, TypedBits};

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    Noop,
    // lhs <- arg1 op arg2
    Binary(Binary),
    // lhs <- op arg1
    Unary(Unary),
    // Select a value based on a condition.
    Select(Select),
    // lhs <- arg[path]
    Index(Index),
    // lhs <- rhs,
    Assign(Assign),
    // lhs <- rhs, where rhs[path] = arg
    Splice(Splice),
    // lhs <- [value; len]
    Repeat(Repeat),
    // lhs <- Struct@path { fields (..rest) }
    Struct(Struct),
    // lhs <- Tuple(fields)
    Tuple(Tuple),
    // ROM table
    Case(Case),
    // lhs = @path(args)
    Exec(Exec),
    // x <- [a, b, c, d]
    Array(Array),
    // x <- tag where tag is the discriminant of the enum.
    Discriminant(Discriminant),
    // x <- enum(discriminant, fields)
    Enum(Enum),
    // x <- a as bits::<len>
    AsBits(Cast),
    // x <- a as signed::<len>
    AsSigned(Cast),
    Comment(String),
}
impl OpCode {
    pub(crate) fn rename_read_register(self, old: Slot, new: Slot) -> OpCode {
        match self {
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => OpCode::Binary(Binary {
                op,
                lhs,
                arg1: arg1.rename(old, new),
                arg2: arg2.rename(old, new),
            }),
            OpCode::Unary(Unary { op, lhs, arg1 }) => OpCode::Unary(Unary {
                op,
                lhs,
                arg1: arg1.rename(old, new),
            }),
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => OpCode::Select(Select {
                lhs,
                cond: cond.rename(old, new),
                true_value: true_value.rename(old, new),
                false_value: false_value.rename(old, new),
            }),
            OpCode::Index(Index { lhs, arg, path }) => OpCode::Index(Index {
                lhs,
                arg: arg.rename(old, new),
                path: path.rename_dyn_slots(old, new),
            }),
            OpCode::Assign(Assign { lhs, rhs }) => {
                let new_rhs = rhs.rename(old, new);
                if new_rhs == lhs {
                    OpCode::Noop
                } else {
                    OpCode::Assign(Assign { lhs, rhs: new_rhs })
                }
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                path,
                subst,
            }) => OpCode::Splice(Splice {
                lhs,
                orig: orig.rename(old, new),
                path: path.rename_dyn_slots(old, new),
                subst: subst.rename(old, new),
            }),
            OpCode::Repeat(Repeat { lhs, value, len }) => OpCode::Repeat(Repeat {
                lhs,
                value: value.rename(old, new),
                len,
            }),
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => OpCode::Struct(Struct {
                lhs,
                fields: fields
                    .into_iter()
                    .map(|f| FieldValue {
                        member: f.member,
                        value: f.value.rename(old, new),
                    })
                    .collect(),
                rest: rest.map(|r| r.rename(old, new)),
                template,
            }),
            OpCode::Tuple(Tuple { lhs, fields }) => OpCode::Tuple(Tuple {
                lhs,
                fields: fields.into_iter().map(|f| f.rename(old, new)).collect(),
            }),
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => OpCode::Case(Case {
                lhs,
                discriminant: discriminant.rename(old, new),
                table: table
                    .into_iter()
                    .map(|(arg, slot)| (arg, slot.rename(old, new)))
                    .collect(),
            }),
            OpCode::Exec(Exec { lhs, id, args }) => OpCode::Exec(Exec {
                lhs,
                id,
                args: args.into_iter().map(|x| x.rename(old, new)).collect(),
            }),
            OpCode::Array(Array { lhs, elements }) => OpCode::Array(Array {
                lhs,
                elements: elements.into_iter().map(|x| x.rename(old, new)).collect(),
            }),
            OpCode::Discriminant(Discriminant { lhs, arg }) => OpCode::Discriminant(Discriminant {
                lhs,
                arg: arg.rename(old, new),
            }),
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => OpCode::Enum(Enum {
                lhs,
                fields: fields
                    .into_iter()
                    .map(|f| FieldValue {
                        member: f.member,
                        value: f.value.rename(old, new),
                    })
                    .collect(),
                template,
            }),
            OpCode::AsBits(Cast { lhs, arg, len }) => OpCode::AsBits(Cast {
                lhs,
                arg: arg.rename(old, new),
                len,
            }),
            OpCode::AsSigned(Cast { lhs, arg, len }) => OpCode::AsSigned(Cast {
                lhs,
                arg: arg.rename(old, new),
                len,
            }),
            _ => self,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub op: AluBinary,
    pub lhs: Slot,
    pub arg1: Slot,
    pub arg2: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub op: AluUnary,
    pub lhs: Slot,
    pub arg1: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub lhs: Slot,
    pub cond: Slot,
    pub true_value: Slot,
    pub false_value: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub lhs: Slot,
    pub arg: Slot,
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub lhs: Slot,
    pub rhs: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Splice {
    pub lhs: Slot,
    pub orig: Slot,
    pub path: Path,
    pub subst: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Repeat {
    pub lhs: Slot,
    pub value: Slot,
    pub len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Slot>,
    pub template: TypedBits,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub lhs: Slot,
    pub discriminant: Slot,
    pub table: Vec<(CaseArgument, Slot)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub lhs: Slot,
    pub elements: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub lhs: Slot,
    pub fields: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exec {
    pub lhs: Slot,
    pub id: FuncId,
    pub args: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaseArgument {
    Constant(TypedBits),
    Wild,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AluBinary {
    Add,
    Sub,
    Mul,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AluUnary {
    Neg,
    Not,
    All,
    Any,
    Xor,
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Slot {
    Literal(usize),
    Register(usize),
    Empty,
}
impl Slot {
    pub fn reg(&self) -> Result<usize> {
        match self {
            Slot::Register(r) => Ok(*r),
            _ => Err(anyhow::anyhow!("Not a register")),
        }
    }
    pub fn is_literal(&self) -> bool {
        matches!(self, Slot::Literal(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Slot::Empty)
    }

    pub(crate) fn is_reg(&self) -> bool {
        matches!(self, Slot::Register(_))
    }

    pub(crate) fn rename(&self, old: Slot, new: Slot) -> Slot {
        if *self == old {
            new
        } else {
            *self
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncId(pub usize);

#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub path: String,
    pub code: KernelFnKind,
    pub signature: DigitalSignature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Discriminant {
    pub lhs: Slot,
    pub arg: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub template: TypedBits,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cast {
    pub lhs: Slot,
    pub arg: Slot,
    pub len: usize,
}
