use crate::Kind;

// Modeled after rustc's AST

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeId(u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        NodeId(id)
    }
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub id: Option<NodeId>,
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
    pub id: Option<NodeId>,
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct Local {
    pub id: Option<NodeId>,
    pub pat: Box<Pat>,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Pat {
    pub id: Option<NodeId>,
    pub kind: PatKind,
}

#[derive(Debug, Clone)]
pub struct PathSegment {
    pub ident: String,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub id: Option<NodeId>,
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
    Struct(PatStruct),
    Type(PatType),
    Wild,
}

#[derive(Debug, Clone)]
pub struct PatIdent {
    pub name: String,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct PatTuple {
    pub elements: Vec<Box<Pat>>,
}

#[derive(Debug, Clone)]
pub struct PatTupleStruct {
    pub path: Box<Path>,
    pub elems: Vec<Box<Pat>>,
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
    pub id: Option<NodeId>,
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
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprRepeat {
    pub value: Box<Expr>,
    pub len: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprStruct {
    pub path: Box<Path>,
    pub fields: Vec<Box<FieldValue>>,
    pub rest: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprCall {
    pub path: Box<Path>,
    pub args: Vec<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprMethodCall {
    pub receiver: Box<Expr>,
    pub args: Vec<Box<Expr>>,
    pub method: String,
}

#[derive(Debug, Clone)]
pub struct FieldValue {
    pub member: Member,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub struct Arm {
    pub pattern: Box<Pat>,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprLit {
    Int(String),
    Bool(bool),
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
