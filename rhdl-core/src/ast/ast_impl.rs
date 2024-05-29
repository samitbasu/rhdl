use crate::{
    kernel::KernelFnKind, rhif::spec::Member, types::typed_bits::TypedBits, DigitalSignature, Kind,
};

// Modeled after rustc's AST

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    pub const fn new(id: u32) -> Self {
        NodeId(id)
    }
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl std::fmt::Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "N{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Local(Box<Local>),
    Expr(Box<Expr>),
    Semi(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct Local {
    pub id: NodeId,
    pub pat: Box<Pat>,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
}

#[derive(Debug, Clone)]
pub struct PathSegment {
    pub ident: &'static str,
    pub arguments: Vec<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct PatIdent {
    pub name: &'static str,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct PatTuple {
    pub elements: Vec<Box<Pat>>,
}

#[derive(Debug, Clone)]
pub struct PatSlice {
    pub elems: Vec<Box<Pat>>,
}

#[derive(Debug, Clone)]
pub struct PatTupleStruct {
    pub path: Box<Path>,
    pub elems: Vec<Box<Pat>>,
    pub signature: DigitalSignature,
}

#[derive(Debug, Clone)]
pub struct PatLit {
    pub lit: Box<ExprLit>,
}

#[derive(Debug, Clone)]
pub struct PatOr {
    pub segments: Vec<Box<Pat>>,
}

#[derive(Debug, Clone)]
pub struct PatParen {
    pub pat: Box<Pat>,
}

#[derive(Debug, Clone)]
pub struct PatPath {
    pub path: Box<Path>,
}

#[derive(Debug, Clone)]
pub struct PatStruct {
    pub path: Box<Path>,
    pub fields: Vec<Box<FieldPat>>,
    pub rest: bool,
}

#[derive(Debug, Clone)]
pub struct PatType {
    pub pat: Box<Pat>,
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
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
    Bits(ExprBits),
}

#[derive(Debug, Clone)]
pub struct ExprType {
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprMatch {
    pub expr: Box<Expr>,
    pub arms: Vec<Box<Arm>>,
}

#[derive(Debug, Clone)]
pub struct ExprRet {
    pub expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprIf {
    pub cond: Box<Expr>,
    pub then_branch: Box<Block>,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprParen {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprTuple {
    pub elements: Vec<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprForLoop {
    pub pat: Box<Pat>,
    pub expr: Box<Expr>,
    pub body: Box<Block>,
}

#[derive(Debug, Clone)]
pub struct ExprAssign {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprGroup {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprField {
    pub expr: Box<Expr>,
    pub member: Member,
}

#[derive(Debug, Clone)]
pub struct ExprBlock {
    pub block: Box<Block>,
}

#[derive(Debug, Clone)]
pub struct ExprArray {
    pub elems: Vec<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprRange {
    pub start: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub end: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprPath {
    pub path: Box<Path>,
}

#[derive(Debug, Clone)]
pub struct ExprLet {
    pub pattern: Box<Pat>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprRepeat {
    pub value: Box<Expr>,
    pub len: i64,
}

#[derive(Debug, Clone)]
pub struct ExprStruct {
    pub path: Box<Path>,
    pub fields: Vec<Box<FieldValue>>,
    pub rest: Option<Box<Expr>>,
    pub template: TypedBits,
    pub variant: Kind,
    pub discriminant: TypedBits,
}

#[derive(Debug, Clone)]
pub struct ExprBits {
    pub kind: BitsKind,
    pub arg: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum BitsKind {
    Unsigned,
    Signed,
}

#[derive(Debug, Clone)]
pub struct ExprCall {
    pub path: Box<Path>,
    pub args: Vec<Box<Expr>>,
    pub signature: Option<DigitalSignature>,
    pub code: Option<KernelFnKind>,
}

#[derive(Debug, Clone)]
pub struct ExprMethodCall {
    pub receiver: Box<Expr>,
    pub args: Vec<Box<Expr>>,
    pub method: &'static str,
}

#[derive(Debug, Clone)]
pub struct FieldValue {
    pub member: Member,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, Copy)]
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

impl BinOp {
    pub fn is_self_assign(&self) -> bool {
        matches!(
            self,
            BinOp::AddAssign
                | BinOp::SubAssign
                | BinOp::MulAssign
                | BinOp::BitXorAssign
                | BinOp::BitAndAssign
                | BinOp::BitOrAssign
                | BinOp::ShlAssign
                | BinOp::ShrAssign
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub struct Arm {
    pub id: NodeId,
    pub kind: ArmKind,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum ArmKind {
    Wild,
    Constant(ArmConstant),
    Enum(ArmEnum),
}

#[derive(Debug, Clone)]
pub struct ArmConstant {
    pub value: ExprLit,
}

#[derive(Debug, Clone)]
pub struct ArmEnum {
    pub pat: Box<Pat>,
    pub template: TypedBits,
    pub payload_kind: Kind,
}

#[derive(Debug, Clone)]
pub enum ExprLit {
    TypedBits(ExprTypedBits),
    Int(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct ExprTypedBits {
    pub path: Box<Path>,
    pub value: TypedBits,
}

#[derive(Debug, Clone)]
pub enum RangeLimits {
    HalfOpen,
    Closed,
}

#[derive(Debug, Clone)]
pub struct FieldPat {
    pub member: Member,
    pub pat: Box<Pat>,
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq, Default)]
pub struct FunctionId(u64);

impl From<u64> for FunctionId {
    fn from(id: u64) -> Self {
        FunctionId(id)
    }
}

impl std::fmt::Debug for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FnID({:x})", self.0)
    }
}

impl std::fmt::LowerHex for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct KernelFn {
    pub id: NodeId,
    pub name: &'static str,
    pub inputs: Vec<Box<Pat>>,
    pub ret: Kind,
    pub body: Box<Block>,
    pub fn_id: FunctionId,
}
