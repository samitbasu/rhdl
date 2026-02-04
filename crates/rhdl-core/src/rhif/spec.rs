use internment::Intern;

// RHDL Intermediate Form (RHIF).
use crate::{
    Color, Kind, TypedBits,
    ast::ast_impl::WrapOp,
    common::symtab::{Symbol, SymbolKind},
    types::path::Path,
};

#[derive(Clone, PartialEq, Eq, Hash)]
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
    // x <- wrap(a), where wrap is either Result::Ok, Result::Err, Option::Some, Option::None
    Wrap(Wrap),
}

impl OpCode {
    pub fn lhs(&self) -> Option<Slot> {
        match self {
            OpCode::Noop => None,
            OpCode::Binary(Binary { lhs, .. })
            | OpCode::Unary(Unary { lhs, .. })
            | OpCode::Select(Select { lhs, .. })
            | OpCode::Index(Index { lhs, .. })
            | OpCode::Assign(Assign { lhs, .. })
            | OpCode::Splice(Splice { lhs, .. })
            | OpCode::Repeat(Repeat { lhs, .. })
            | OpCode::Struct(Struct { lhs, .. })
            | OpCode::Tuple(Tuple { lhs, .. })
            | OpCode::Case(Case { lhs, .. })
            | OpCode::Exec(Exec { lhs, .. })
            | OpCode::Array(Array { lhs, .. })
            | OpCode::Enum(Enum { lhs, .. })
            | OpCode::AsBits(Cast { lhs, .. })
            | OpCode::AsSigned(Cast { lhs, .. })
            | OpCode::Resize(Cast { lhs, .. })
            | OpCode::Retime(Retime { lhs, .. })
            | OpCode::Wrap(Wrap { lhs, .. }) => Some(*lhs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wrap {
    pub op: WrapOp,
    pub lhs: Slot,
    pub arg: Slot,
    pub kind: Option<Kind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binary {
    pub op: AluBinary,
    pub lhs: Slot,
    pub arg1: Slot,
    pub arg2: Slot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unary {
    pub op: AluUnary,
    pub lhs: Slot,
    pub arg1: Slot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Select {
    pub lhs: Slot,
    pub cond: Slot,
    pub true_value: Slot,
    pub false_value: Slot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Index {
    pub lhs: Slot,
    pub arg: Slot,
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assign {
    pub lhs: Slot,
    pub rhs: Slot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Splice {
    pub lhs: Slot,
    pub orig: Slot,
    pub path: Path,
    pub subst: Slot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Repeat {
    pub lhs: Slot,
    pub value: Slot,
    pub len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Struct {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Slot>,
    pub template: TypedBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Case {
    pub lhs: Slot,
    pub discriminant: Slot,
    pub table: Vec<(CaseArgument, Slot)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
    pub lhs: Slot,
    pub elements: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tuple {
    pub lhs: Slot,
    pub fields: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Exec {
    pub lhs: Slot,
    pub id: FuncId,
    pub args: Vec<Slot>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CaseArgument {
    Slot(Slot),
    Wild,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
    XAdd,
    XSub,
    XMul,
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AluUnary {
    Neg,
    Not,
    All,
    Any,
    Xor,
    Signed,
    Unsigned,
    Val,
    XExt(usize),
    XShl(usize),
    XShr(usize),
    XNeg,
    XSgn,
}

#[derive(Hash, Eq, Ord, PartialOrd, PartialEq, Copy, Clone, Default)]
pub struct SlotKind {}

impl SymbolKind for SlotKind {
    const NAME: &'static str = "s";
}

pub type Slot = Symbol<SlotKind>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Member {
    Named(Intern<String>),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Enum {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub template: TypedBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cast {
    pub lhs: Slot,
    pub arg: Slot,
    pub len: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Retime {
    pub lhs: Slot,
    pub arg: Slot,
    pub color: Option<Color>,
}
