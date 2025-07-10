use std::ops::Range;

use crate::rhdl_core::common::symtab::{LiteralId, Symbol};

#[derive(Clone, PartialEq, Hash)]
pub enum OpCode {
    Noop,
    // lhs <- arg
    Assign(Assign),
    // lhs <- arg1 op arg2
    Binary(Binary),
    // lhs <- table[slot]
    Case(Case),
    // lhs <- cast(slot) as signed/unsigned
    Cast(Cast),
    // Comment
    Comment(String),
    // lhs <- {{ r1, r2, ... }}
    Concat(Concat),
    // lhs <- arg[bit_range]
    Index(Index),
    // lhs <- cond ? true_value : false_value
    Select(Select),
    // lhs <- arg; lhs[bit_range] <- value
    Splice(Splice),
    // lhs <- op arg1
    Unary(Unary),
}

pub type Operand = Symbol;

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

impl From<AluBinary> for crate::rhdl_core::rhif::spec::AluBinary {
    fn from(op: AluBinary) -> Self {
        match op {
            AluBinary::Add => crate::rhdl_core::rhif::spec::AluBinary::Add,
            AluBinary::Sub => crate::rhdl_core::rhif::spec::AluBinary::Sub,
            AluBinary::Mul => crate::rhdl_core::rhif::spec::AluBinary::Mul,
            AluBinary::BitXor => crate::rhdl_core::rhif::spec::AluBinary::BitXor,
            AluBinary::BitAnd => crate::rhdl_core::rhif::spec::AluBinary::BitAnd,
            AluBinary::BitOr => crate::rhdl_core::rhif::spec::AluBinary::BitOr,
            AluBinary::Shl => crate::rhdl_core::rhif::spec::AluBinary::Shl,
            AluBinary::Shr => crate::rhdl_core::rhif::spec::AluBinary::Shr,
            AluBinary::Eq => crate::rhdl_core::rhif::spec::AluBinary::Eq,
            AluBinary::Lt => crate::rhdl_core::rhif::spec::AluBinary::Lt,
            AluBinary::Le => crate::rhdl_core::rhif::spec::AluBinary::Le,
            AluBinary::Ne => crate::rhdl_core::rhif::spec::AluBinary::Ne,
            AluBinary::Ge => crate::rhdl_core::rhif::spec::AluBinary::Ge,
            AluBinary::Gt => crate::rhdl_core::rhif::spec::AluBinary::Gt,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Binary {
    pub op: AluBinary,
    pub lhs: Operand,
    pub arg1: Operand,
    pub arg2: Operand,
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

impl From<AluUnary> for crate::rhdl_core::rhif::spec::AluUnary {
    fn from(op: AluUnary) -> Self {
        match op {
            AluUnary::Neg => crate::rhdl_core::rhif::spec::AluUnary::Neg,
            AluUnary::Not => crate::rhdl_core::rhif::spec::AluUnary::Not,
            AluUnary::All => crate::rhdl_core::rhif::spec::AluUnary::All,
            AluUnary::Any => crate::rhdl_core::rhif::spec::AluUnary::Any,
            AluUnary::Xor => crate::rhdl_core::rhif::spec::AluUnary::Xor,
            AluUnary::Signed => crate::rhdl_core::rhif::spec::AluUnary::Signed,
            AluUnary::Unsigned => crate::rhdl_core::rhif::spec::AluUnary::Unsigned,
            AluUnary::Val => crate::rhdl_core::rhif::spec::AluUnary::Val,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Unary {
    pub op: AluUnary,
    pub lhs: Operand,
    pub arg1: Operand,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Select {
    pub lhs: Operand,
    pub cond: Operand,
    pub true_value: Operand,
    pub false_value: Operand,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Concat {
    pub lhs: Operand,
    pub args: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Index {
    pub lhs: Operand,
    pub arg: Operand,
    pub bit_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Splice {
    pub lhs: Operand,
    pub orig: Operand,
    pub bit_range: Range<usize>,
    pub value: Operand,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Assign {
    pub lhs: Operand,
    pub rhs: Operand,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum CaseArgument {
    Literal(LiteralId),
    Wild,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Case {
    pub lhs: Operand,
    pub discriminant: Operand,
    pub table: Vec<(CaseArgument, Operand)>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Cast {
    pub lhs: Operand,
    pub arg: Operand,
    pub len: usize,
    pub kind: CastKind,
}

#[derive(Debug, Clone, PartialEq, Copy, Hash)]
pub enum CastKind {
    Signed,
    Unsigned,
    Resize,
}
