use std::fmt::Display;

use crate::{digital_fn::DigitalSignature, kernel::KernelFnKind, typed_bits::TypedBits, Kind};
use serde::{Deserialize, Serialize};

// Modeled after rustc's AST

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct NodeId(Option<u32>);

impl NodeId {
    pub fn new(id: u32) -> Self {
        NodeId(Some(id))
    }
    // Panics for invalid node IDs!
    pub fn as_u32(self) -> u32 {
        self.0.unwrap()
    }
    pub fn is_invalid(&self) -> bool {
        self.0 == INVALID_NODE_ID.0
    }
}

pub const INVALID_NODE_ID: NodeId = NodeId(None);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(x) = self.0 {
            write!(f, "N{x}")
        } else {
            write!(f, "N<invalid>")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StmtKind {
    Local(Box<Local>),
    Expr(Box<Expr>),
    Semi(Box<Expr>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Local {
    pub id: NodeId,
    pub pat: Box<Pat>,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    pub ident: String,
    pub arguments: Vec<Box<Expr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatKind {
    Ident(PatIdent),
    Tuple(PatTuple),
    TupleStruct(PatTupleStruct),
    Lit(PatLit),
    Or(PatOr),
    Paren(PatParen),
    Path(PatPath),
    Slice(PatSlice),
    Struct(PatStruct),
    Type(PatType),
    Wild,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatIdent {
    pub name: String,
    pub mutable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatTuple {
    pub elements: Vec<Box<Pat>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatSlice {
    pub elems: Vec<Box<Pat>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatTupleStruct {
    pub path: Box<Path>,
    pub elems: Vec<Box<Pat>>,
    pub signature: DigitalSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatLit {
    pub lit: Box<ExprLit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatOr {
    pub segments: Vec<Box<Pat>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatParen {
    pub pat: Box<Pat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatPath {
    pub path: Box<Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatStruct {
    pub path: Box<Path>,
    pub fields: Vec<Box<FieldPat>>,
    pub rest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatType {
    pub pat: Box<Pat>,
    pub kind: Kind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExprKind {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Match(ExprMatch),
    Ret(ExprRet),
    If(ExprIf),
    Index(ExprIndex),
    Lit(ExprLit),
    Paren(ExprParen),
    Tuple(ExprTuple),
    ForLoop(ExprForLoop),
    Assign(ExprAssign),
    Group(ExprGroup),
    Field(ExprField),
    Block(ExprBlock),
    Array(ExprArray),
    Range(ExprRange),
    Path(ExprPath),
    Let(ExprLet),
    Repeat(ExprRepeat),
    Struct(ExprStruct),
    Call(ExprCall),
    MethodCall(ExprMethodCall),
    Type(ExprType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprType {
    pub kind: Kind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprBinary {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprMatch {
    pub expr: Box<Expr>,
    pub arms: Vec<Box<Arm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprRet {
    pub expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprIf {
    pub cond: Box<Expr>,
    pub then_branch: Box<Block>,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprParen {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprTuple {
    pub elements: Vec<Box<Expr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprForLoop {
    pub pat: Box<Pat>,
    pub expr: Box<Expr>,
    pub body: Box<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprAssign {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprGroup {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprField {
    pub expr: Box<Expr>,
    pub member: Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprBlock {
    pub block: Box<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprArray {
    pub elems: Vec<Box<Expr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprRange {
    pub start: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub end: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprPath {
    pub path: Box<Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprLet {
    pub pattern: Box<Pat>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprRepeat {
    pub value: Box<Expr>,
    pub len: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprStruct {
    pub path: Box<Path>,
    pub fields: Vec<Box<FieldValue>>,
    pub rest: Option<Box<Expr>>,
    pub template: TypedBits,
    pub variant: Kind,
    pub discriminant: TypedBits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprCall {
    pub path: Box<Path>,
    pub args: Vec<Box<Expr>>,
    pub signature: DigitalSignature,
    pub code: KernelFnKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprMethodCall {
    pub receiver: Box<Expr>,
    pub args: Vec<Box<Expr>>,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    pub member: Member,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinOp {
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
    AddAssign,
    SubAssign,
    MulAssign,
    BitXorAssign,
    BitAndAssign,
    BitOrAssign,
    ShlAssign,
    ShrAssign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arm {
    pub id: NodeId,
    pub kind: ArmKind,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArmKind {
    Wild,
    Constant(ArmConstant),
    Enum(ArmEnum),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmConstant {
    pub value: ExprLit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmEnum {
    pub pat: Box<Pat>,
    pub template: TypedBits,
    pub payload_kind: Kind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExprLit {
    TypedBits(ExprTypedBits),
    Int(String),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprTypedBits {
    pub path: Box<Path>,
    pub value: TypedBits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RangeLimits {
    HalfOpen,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPat {
    pub member: Member,
    pub pat: Box<Pat>,
}

#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Default,
)]
pub struct FunctionId(u64);

impl From<u64> for FunctionId {
    fn from(id: u64) -> Self {
        FunctionId(id)
    }
}

impl std::fmt::Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FnID({:x})", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelFn {
    pub id: NodeId,
    pub name: String,
    pub inputs: Vec<Box<Pat>>,
    pub ret: Kind,
    pub body: Box<Block>,
    pub fn_id: FunctionId,
}
