// RHDL Intermediate Form (RHIF).

use std::ops::Range;

use crate::digital::TypedBits;

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
}

pub struct MatchOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub arms: Vec<MatchArm>,
}

pub struct MatchArm {
    //    pub pattern: MatchPattern,
    pub body: BlockOp,
}

pub struct TupleOp {
    pub lhs: Slot,
    pub fields: Vec<Slot>,
}
pub struct StructOp {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Slot>,
}

pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

pub struct RepeatOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub count: Slot,
}
pub struct FieldOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub member: Member,
}

pub struct AssignOp {
    pub lhs: Slot,
    pub rhs: Slot,
}
pub struct IndexOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub index: Slot,
}

pub struct BlockOp {
    pub lhs: Slot,
    pub body: Vec<OpCode>,
}

pub struct IfOp {
    pub lhs: Option<Slot>,
    pub cond: Slot,
    pub then_branch: BlockOp,
    pub else_branch: Box<OpCode>,
}

pub struct BinaryOp {
    pub op: AluBinary,
    pub lhs: Slot,
    pub arg1: Slot,
    pub arg2: Slot,
}

pub struct UnaryOp {
    pub op: AluUnary,
    pub lhs: Slot,
    pub arg1: Slot,
}

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

pub enum AluUnary {
    Neg,
    Not,
}

pub enum Slot {
    Const(TypedBits),
    Register(u32),
    Reference { reg: u32, range: Range<usize> },
}

pub enum Member {
    Named(String),
    Unnamed(u32),
}
