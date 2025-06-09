use crate::prelude::BitString;

#[derive(Clone, PartialEq, Hash)]
pub enum OpCode {
    Noop,
    // lhs <- arg
    Assign(Assign),
    // lhs <- arg1 op arg2
    Binary(Binary),
    // [lhs.0..lhs.N-1] <- [arg1.0..arg1.N-1] op [arg2.0..arg2.N-1]
    Vector(VectorOp),
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
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BlackBox {
    pub lhs: Vec<RegisterId>,
    pub arg: Vec<Operand>,
    pub code: BlackBoxId,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Dff {
    pub lhs: RegisterId,
    pub arg: Operand,
    pub clock: Operand,
    pub reset: Operand,
    pub reset_value: bool,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct DynamicIndex {
    pub lhs: Vec<RegisterId>,
    pub arg: Vec<Operand>,
    pub offset: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct DynamicSplice {
    pub lhs: Vec<RegisterId>,
    pub arg: Vec<Operand>,
    pub offset: Vec<Operand>,
    pub value: Vec<Operand>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Select {
    pub lhs: RegisterId,
    pub selector: Operand,
    pub true_case: Operand,
    pub false_case: Operand,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Case {
    pub lhs: RegisterId,
    pub discriminant: Vec<Operand>,
    pub entries: Vec<CaseEntry>,
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum CaseEntry {
    Literal(BitString),
    WildCard,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Vector {
    pub op: VectorOp,
    pub lhs: Vec<RegisterId>,
    pub arg1: Vec<Operand>,
    pub arg2: Vec<Operand>,
    pub signed: bool,
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
    BitSelect,
    All,
    Any,
    Neg,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Not {
    pub lhs: RegisterId,
    pub arg: Operand,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Binary {
    pub op: BinaryOp,
    pub lhs: RegisterId,
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
    pub lhs: RegisterId,
    pub rhs: Operand,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Operand {
    Zero,
    One,
    X,
    Register(RegisterId),
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RegisterId(usize, usize);

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct BlackBoxId(usize);
