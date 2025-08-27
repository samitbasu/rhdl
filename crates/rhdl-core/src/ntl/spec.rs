use crate::{
    common::symtab::{Symbol, SymbolKind},
    types::bit_string::BitString,
};

#[derive(Clone, PartialEq, Hash)]
pub enum OpCode {
    Noop,
    // lhs <- arg
    Assign(Assign),
    // lhs <- arg1 op arg2
    Binary(Binary),
    // [lhs.0..lhs.N-1] <- [arg1.0..arg1.N-1] op [arg2.0..arg2.N-1]
    Vector(Vector),
    // lhs <- case [arg1] {pattern0 : arg0, pattern1: arg1, ...}
    Case(Case),
    // Comment
    Comment(String),
    // lhs <- cond ? true_value : false_value
    Select(Select),
    // lhs <- ! arg
    Not(Not),
    // [lhs...] = black_box([arg...])
    BlackBox(BlackBox),
    // lhs <- reduce(arg)
    Unary(Unary),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BlackBox {
    pub lhs: Vec<Wire>,
    pub arg: Vec<Vec<Wire>>,
    pub code: BlackBoxId,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Select {
    pub lhs: Wire,
    pub selector: Wire,
    pub true_case: Wire,
    pub false_case: Wire,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Case {
    pub lhs: Wire,
    pub discriminant: Vec<Wire>,
    pub entries: Vec<(CaseEntry, Wire)>,
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum CaseEntry {
    Literal(BitString),
    WildCard,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Vector {
    pub op: VectorOp,
    pub lhs: Vec<Wire>,
    pub arg1: Vec<Wire>,
    pub arg2: Vec<Wire>,
    pub signed: bool,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Unary {
    pub op: UnaryOp,
    pub lhs: Vec<Wire>,
    pub arg: Vec<Wire>,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum VectorOp {
    Add,
    Sub,
    Mul,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum UnaryOp {
    All,
    Any,
    Neg,
    Xor,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Not {
    pub lhs: Wire,
    pub arg: Wire,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Binary {
    pub op: BinaryOp,
    pub lhs: Wire,
    pub arg1: Wire,
    pub arg2: Wire,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum BinaryOp {
    Xor,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Assign {
    pub lhs: Wire,
    pub rhs: Wire,
}

pub fn assign(lhs: Wire, rhs: Wire) -> OpCode {
    OpCode::Assign(Assign { lhs, rhs })
}

#[derive(Hash, Eq, Ord, PartialOrd, PartialEq, Copy, Clone, Default)]
pub struct WireKind {}

impl SymbolKind for WireKind {
    const NAME: &'static str = "w";
}

pub type Wire = Symbol<WireKind>;

#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct BlackBoxId(usize);

impl std::fmt::Debug for BlackBoxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl BlackBoxId {
    pub(crate) fn new(x: usize) -> Self {
        Self(x)
    }
    pub(crate) fn offset(self, offset: usize) -> Self {
        Self(self.0 + offset)
    }

    pub(crate) fn raw(self) -> usize {
        self.0
    }
}
