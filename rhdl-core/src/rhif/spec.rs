// RHDL Intermediate Form (RHIF).
use anyhow::Result;

use crate::{
    ast_builder::BinOp,
    kernel::{ExternalKernelDef, Kernel},
    path::Path,
    DigitalSignature, Kind, TypedBits,
};

#[derive(Debug, Clone, PartialEq)]
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
    // x <- C::sig(a)
    AsKind(KindCast),
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
pub struct Select {
    pub lhs: Slot,
    pub cond: Slot,
    pub true_value: Slot,
    pub false_value: Slot,
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
    pub lhs: Slot,
    pub discriminant: Slot,
    pub table: Vec<(CaseArgument, Slot)>,
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
    Slot(Slot),
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

    pub(crate) fn rename(&self, old: Slot, new: Slot) -> Slot {
        if *self == old {
            new
        } else {
            *self
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncId(pub usize);

#[derive(Debug, Clone)]
pub enum ExternalFunctionCode {
    Kernel(Kernel),
    Extern(ExternalKernelDef),
}

impl std::fmt::Display for ExternalFunctionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalFunctionCode::Kernel(kernel) => {
                write!(
                    f,
                    "kernel {name} {fn_id}",
                    name = kernel.inner().name,
                    fn_id = kernel.inner().fn_id
                )
            }
            ExternalFunctionCode::Extern(extern_kernel) => {
                write!(f, "extern {}", extern_kernel.name)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub path: String,
    pub code: ExternalFunctionCode,
    pub signature: DigitalSignature,
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

#[derive(Debug, Clone, PartialEq)]
pub struct KindCast {
    pub lhs: Slot,
    pub arg: Slot,
    pub kind: Kind,
}
