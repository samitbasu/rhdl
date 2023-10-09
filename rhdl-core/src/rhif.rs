// RHDL Intermediate Form (RHIF).

use anyhow::Result;

use crate::ast::ExprLit;

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // x <- a op b
    Binary(BinaryOp),
    // x <- op a
    Unary(UnaryOp),
    // return a
    Return(Option<Slot>),
    // if cond { then_branch } else { else_branch }
    If(IfOp),
    // x <- {block}
    Block(BlockOp),
    // x <- a[i]
    Index(IndexOp),
    // x <- a
    Copy(CopyOp),
    // *x <- a
    Assign(AssignOp),
    // x <- a.field
    Field(FieldOp),
    // x <- [a; count]
    Repeat(RepeatOp),
    // x <- Struct { fields }
    Struct(StructOp),
    // x <- Tuple(fields)
    Tuple(TupleOp),
    // x <- match a { arms }
    Match(MatchOp),
    // x = &a
    Ref(RefOp),
    // x = &a.field
    FieldRef(FieldRefOp),
    // x = &a[i]
    IndexRef(IndexRefOp),
    // Jump
    Call(BlockId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopyOp {
    pub lhs: Slot,
    pub rhs: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefOp {
    pub lhs: Slot,
    pub arg: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexRefOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub index: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldRefOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub member: Member,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    //    pub pattern: MatchPattern,
    pub body: BlockOp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleOp {
    pub lhs: Slot,
    pub fields: Vec<Slot>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct StructOp {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub count: Slot,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FieldOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub member: Member,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignOp {
    pub lhs: Slot,
    pub rhs: Slot,
}
#[derive(Debug, Clone, PartialEq)]
pub struct IndexOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub index: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockOp {
    pub lhs: Slot,
    pub body: Vec<OpCode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfOp {
    pub lhs: Slot,
    pub cond: Slot,
    pub then_branch: BlockId,
    pub else_branch: BlockId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOp {
    pub op: AluBinary,
    pub lhs: Slot,
    pub arg1: Slot,
    pub arg2: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOp {
    pub op: AluUnary,
    pub lhs: Slot,
    pub arg1: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AluBinary {
    Add,
    Sub,
    Mul,
    And,
    Or,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Slot {
    Literal(ExprLit),
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);
