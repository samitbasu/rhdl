// RHDL Intermediate Form (RHIF).
use anyhow::Result;

use crate::{path::Path, DigitalSignature, KernelFnKind, TypedBits};

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // lhs <- arg1 op arg2
    Binary(Binary),
    // lhs <- op arg1
    Unary(Unary),
    // lhs <- if cond { then_branch } else { else_branch }
    If(If),
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
    // Jump to block
    Block(BlockId),
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
pub struct If {
    pub lhs: Slot,
    pub cond: Slot,
    pub then_branch: BlockId,
    pub else_branch: BlockId,
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
    pub discriminant: Slot,
    pub table: Vec<(CaseArgument, BlockId)>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncId(pub usize);

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub ops: Vec<OpCode>,
}

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
