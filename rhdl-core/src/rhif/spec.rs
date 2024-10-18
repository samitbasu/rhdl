// RHDL Intermediate Form (RHIF).
use anyhow::Result;

use crate::{types::path::Path, Color, TypedBits};

#[derive(Clone, PartialEq, Hash)]
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
    // x <- enum(discriminant, fields)
    Enum(Enum),
    // x <- a as bits::<len>
    AsBits(Cast),
    // x <- a as signed::<len>
    AsSigned(Cast),
    // x <- a.cast::<len>
    Resize(Cast),
    // x <- C::sig(a)
    Retime(Retime),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Binary {
    pub op: AluBinary,
    pub lhs: Slot,
    pub arg1: Slot,
    pub arg2: Slot,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Unary {
    pub op: AluUnary,
    pub lhs: Slot,
    pub arg1: Slot,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Select {
    pub lhs: Slot,
    pub cond: Slot,
    pub true_value: Slot,
    pub false_value: Slot,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Index {
    pub lhs: Slot,
    pub arg: Slot,
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Assign {
    pub lhs: Slot,
    pub rhs: Slot,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Splice {
    pub lhs: Slot,
    pub orig: Slot,
    pub path: Path,
    pub subst: Slot,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Repeat {
    pub lhs: Slot,
    pub value: Slot,
    pub len: u64,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Struct {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Slot>,
    pub template: TypedBits,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Case {
    pub lhs: Slot,
    pub discriminant: Slot,
    pub table: Vec<(CaseArgument, Slot)>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Array {
    pub lhs: Slot,
    pub elements: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Tuple {
    pub lhs: Slot,
    pub fields: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Exec {
    pub lhs: Slot,
    pub id: FuncId,
    pub args: Vec<Slot>,
}

#[derive(Clone, PartialEq, Hash)]
pub enum CaseArgument {
    Slot(Slot),
    Wild,
}

#[derive(Clone, PartialEq, Hash)]
pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

#[derive(Clone, Copy, PartialEq, Hash)]
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

impl AluBinary {
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            AluBinary::Eq
                | AluBinary::Lt
                | AluBinary::Le
                | AluBinary::Ne
                | AluBinary::Ge
                | AluBinary::Gt
        )
    }

    pub(crate) fn is_shift(&self) -> bool {
        matches!(self, AluBinary::Shl | AluBinary::Shr)
    }
}

#[derive(Clone, Copy, PartialEq, Hash)]
pub enum AluUnary {
    Neg,
    Not,
    All,
    Any,
    Xor,
    Signed,
    Unsigned,
    Val,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Slot {
    Literal(LiteralId),
    Register(RegisterId),
    Empty,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct LiteralId(pub usize);

impl From<LiteralId> for Slot {
    fn from(val: LiteralId) -> Self {
        Slot::Literal(val)
    }
}

impl std::fmt::Debug for LiteralId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "l{}", self.0)
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RegisterId(pub usize);

impl From<RegisterId> for Slot {
    fn from(val: RegisterId) -> Self {
        Slot::Register(val)
    }
}

impl std::fmt::Debug for RegisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

impl std::fmt::Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Literal(l) => write!(f, "{:?}", l),
            Slot::Register(r) => write!(f, "{:?}", r),
            Slot::Empty => write!(f, "()"),
        }
    }
}

impl Slot {
    pub fn reg(&self) -> Result<RegisterId> {
        match self {
            Slot::Register(r) => Ok(*r),
            _ => Err(anyhow::anyhow!("Not a register")),
        }
    }
    pub fn as_literal(self) -> Result<LiteralId> {
        match self {
            Slot::Literal(l) => Ok(l),
            _ => Err(anyhow::anyhow!("Not a literal")),
        }
    }
    pub fn as_reg(self) -> Result<RegisterId> {
        match self {
            Slot::Register(r) => Ok(r),
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

#[derive(Clone, PartialEq, Hash)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncId(usize);

impl std::fmt::Debug for FuncId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fid({})", self.0)
    }
}

impl From<usize> for FuncId {
    fn from(id: usize) -> Self {
        FuncId(id)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Enum {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub template: TypedBits,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Cast {
    pub lhs: Slot,
    pub arg: Slot,
    pub len: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Retime {
    pub lhs: Slot,
    pub arg: Slot,
    pub color: Option<Color>,
}
