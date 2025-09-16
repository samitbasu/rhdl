use std::ops::Range;

use crate::{
    common::symtab::{LiteralId, Symbol, SymbolKind},
    types::path::Path,
};

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

#[derive(Hash, Eq, Ord, PartialOrd, PartialEq, Copy, Clone, Default)]
pub struct OperandKind {}

impl SymbolKind for OperandKind {
    const NAME: &'static str = "o";
}

pub type Operand = Symbol<OperandKind>;

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

impl From<AluBinary> for crate::rhif::spec::AluBinary {
    fn from(op: AluBinary) -> Self {
        match op {
            AluBinary::Add => crate::rhif::spec::AluBinary::Add,
            AluBinary::Sub => crate::rhif::spec::AluBinary::Sub,
            AluBinary::Mul => crate::rhif::spec::AluBinary::Mul,
            AluBinary::BitXor => crate::rhif::spec::AluBinary::BitXor,
            AluBinary::BitAnd => crate::rhif::spec::AluBinary::BitAnd,
            AluBinary::BitOr => crate::rhif::spec::AluBinary::BitOr,
            AluBinary::Shl => crate::rhif::spec::AluBinary::Shl,
            AluBinary::Shr => crate::rhif::spec::AluBinary::Shr,
            AluBinary::Eq => crate::rhif::spec::AluBinary::Eq,
            AluBinary::Lt => crate::rhif::spec::AluBinary::Lt,
            AluBinary::Le => crate::rhif::spec::AluBinary::Le,
            AluBinary::Ne => crate::rhif::spec::AluBinary::Ne,
            AluBinary::Ge => crate::rhif::spec::AluBinary::Ge,
            AluBinary::Gt => crate::rhif::spec::AluBinary::Gt,
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

impl From<AluUnary> for crate::rhif::spec::AluUnary {
    fn from(op: AluUnary) -> Self {
        match op {
            AluUnary::Neg => crate::rhif::spec::AluUnary::Neg,
            AluUnary::Not => crate::rhif::spec::AluUnary::Not,
            AluUnary::All => crate::rhif::spec::AluUnary::All,
            AluUnary::Any => crate::rhif::spec::AluUnary::Any,
            AluUnary::Xor => crate::rhif::spec::AluUnary::Xor,
            AluUnary::Signed => crate::rhif::spec::AluUnary::Signed,
            AluUnary::Unsigned => crate::rhif::spec::AluUnary::Unsigned,
            AluUnary::Val => crate::rhif::spec::AluUnary::Val,
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
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Splice {
    pub lhs: Operand,
    pub orig: Operand,
    pub bit_range: Range<usize>,
    pub value: Operand,
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Assign {
    pub lhs: Operand,
    pub rhs: Operand,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum CaseArgument {
    Literal(LiteralId<OperandKind>),
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
