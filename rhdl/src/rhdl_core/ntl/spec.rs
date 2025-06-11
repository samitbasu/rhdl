use crate::prelude::{BitString, BitX};

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
    // lhs <- rhs[ndx]
    DynamicIndex(DynamicIndex),
    // lhs[ndx] <- rhs/[ndx]=val
    DynamicSplice(DynamicSplice),
    // lhs <- cond ? true_value : false_value
    Select(Select),
    // lhs <- ! arg
    Not(Not),
    // lhs <- DFF(arg)
    Dff(Dff),
    // [lhs...] = black_box([arg...])
    BlackBox(BlackBox),
    // lhs <- reduce(arg)
    Unary(Unary),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BlackBox {
    pub lhs: Vec<Operand>,
    pub arg: Vec<Operand>,
    pub code: BlackBoxId,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Dff {
    pub lhs: Operand,
    pub arg: Operand,
    pub clock: Operand,
    pub reset: Operand,
    pub reset_value: bool,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct DynamicIndex {
    pub lhs: Vec<Operand>,
    pub arg: Vec<Operand>,
    pub offset: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct DynamicSplice {
    pub lhs: Vec<Operand>,
    pub arg: Vec<Operand>,
    pub offset: Vec<Operand>,
    pub value: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Select {
    pub lhs: Operand,
    pub selector: Operand,
    pub true_case: Operand,
    pub false_case: Operand,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Case {
    pub lhs: Operand,
    pub discriminant: Vec<Operand>,
    pub entries: Vec<(CaseEntry, Operand)>,
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum CaseEntry {
    Literal(BitString),
    WildCard,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Vector {
    pub op: VectorOp,
    pub lhs: Vec<Operand>,
    pub arg1: Vec<Operand>,
    pub arg2: Vec<Operand>,
    pub signed: bool,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Unary {
    pub op: UnaryOp,
    pub lhs: Vec<Operand>,
    pub arg: Vec<Operand>,
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
    BitSelect,
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
    pub lhs: Operand,
    pub arg: Operand,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Binary {
    pub op: BinaryOp,
    pub lhs: Operand,
    pub arg1: Operand,
    pub arg2: Operand,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum BinaryOp {
    Xor,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Assign {
    pub lhs: Operand,
    pub rhs: Operand,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Operand {
    Zero,
    One,
    X,
    Register(RegisterId),
}

impl From<BitX> for Operand {
    fn from(x: BitX) -> Operand {
        match x {
            BitX::One => Operand::One,
            BitX::Zero => Operand::Zero,
            BitX::X => Operand::X,
        }
    }
}

impl Operand {
    pub fn reg(&self) -> Option<RegisterId> {
        if let Operand::Register(reg) = self {
            Some(*reg)
        } else {
            None
        }
    }
    pub fn bitx(&self) -> Option<BitX> {
        match self {
            Operand::Zero => Some(BitX::Zero),
            Operand::One => Some(BitX::One),
            Operand::X => Some(BitX::X),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Zero => write!(f, "0"),
            Operand::One => write!(f, "1"),
            Operand::X => write!(f, "X"),
            Operand::Register(rid) => write!(f, "r{}", rid.0),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RegisterId(u32);

impl RegisterId {
    pub(crate) fn new(val: u32) -> Self {
        Self(val)
    }
    pub(crate) fn raw(self) -> u32 {
        self.0
    }
    pub(crate) fn offset(self, offset: u32) -> Self {
        Self(self.0 + offset)
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct BlackBoxId(usize);

impl BlackBoxId {
    pub(crate) fn offset(self, offset: usize) -> Self {
        Self(self.0 + offset)
    }
}
