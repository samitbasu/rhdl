use std::ops::Range;

use crate::rhif::spec::{AluBinary, AluUnary, FuncId};

#[derive(Clone, PartialEq)]
pub enum OpCode {
    // lhs <- unsigned(slot)
    AsBits(Cast),
    // lhs <- arg
    Assign(Assign),
    // lhs <- signed(slot)
    AsSigned(Cast),
    // lhs <- arg1 op arg2
    Binary(Binary),
    // lhs <- table[slot]
    Case(Case),
    // Comment
    Comment(String),
    // lhs <- {{ r1, r2, ... }}
    Concat(Concat),
    // lhs <- arg[base_offset + arg * stride +: len]
    DynamicIndex(DynamicIndex),
    // lhs <- arg; lhs[base_offset + arg * stride +: len] <- value
    DynamicSplice(DynamicSplice),
    // lhs <- func(arg)
    Exec(Exec),
    // lhs <- arg[bit_range]
    Index(Index),
    // lhs <- cond ? true_value : false_value
    Select(Select),
    // lhs <- arg; lhs[bit_range] <- value
    Splice(Splice),
    // lhs <- op arg1
    Unary(Unary),
}

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Operand {
    Literal(usize),
    Register(usize),
}

impl std::fmt::Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operand::Literal(l) => write!(f, "l{}", l),
            Operand::Register(r) => write!(f, "r{}", r),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub op: AluBinary,
    pub lhs: Operand,
    pub arg1: Operand,
    pub arg2: Operand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub op: AluUnary,
    pub lhs: Operand,
    pub arg1: Operand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub lhs: Operand,
    pub cond: Operand,
    pub true_value: Operand,
    pub false_value: Operand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Concat {
    pub lhs: Operand,
    pub args: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicIndex {
    pub lhs: Operand,
    pub arg: Operand,
    pub base_offset: usize,
    pub stride: usize,
    pub len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicSplice {
    pub lhs: Operand,
    pub arg: Operand,
    pub base_offset: usize,
    pub stride: usize,
    pub len: usize,
    pub value: Operand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub lhs: Operand,
    pub arg: Operand,
    pub bit_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Splice {
    pub lhs: Operand,
    pub orig: Operand,
    pub bit_range: Range<usize>,
    pub value: Operand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub lhs: Operand,
    pub rhs: Operand,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CaseArgument {
    Literal(usize),
    Wild,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub lhs: Operand,
    pub discriminant: Operand,
    pub table: Vec<(CaseArgument, Operand)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cast {
    pub lhs: Operand,
    pub arg: Operand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exec {
    pub lhs: Operand,
    pub id: FuncId,
    pub args: Vec<Operand>,
}
