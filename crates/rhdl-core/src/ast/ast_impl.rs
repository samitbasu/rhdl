use crate::{
    DigitalSignature, Kind, RHDLError, ast::SpannedSource, kernel::KernelFnKind,
    rhif::spec::Member, types::typed_bits::TypedBits,
};
use anyhow::anyhow;
use rhdl_span::MetaDB;

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

impl From<u32> for NodeId {
    fn from(id: u32) -> Self {
        NodeId(id)
    }
}

impl std::fmt::Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "N{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, PartialOrd, Eq, Ord)]
pub struct SourceLocation {
    pub func: FunctionId,
    pub node: NodeId,
}

impl From<(FunctionId, NodeId)> for SourceLocation {
    fn from((func, node): (FunctionId, NodeId)) -> Self {
        Self { func, node }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, Hash)]
pub enum StmtKind {
    Local(Box<Local>),
    Expr(Box<Expr>),
    Semi(Box<Expr>),
}

#[derive(Debug, Clone, Hash)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, Clone, Hash)]
pub struct Local {
    pub id: NodeId,
    pub pat: Box<Pat>,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Hash)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
}

#[derive(Debug, Clone, Hash)]
pub struct PathSegment {
    pub ident: &'static str,
    pub arguments: Vec<&'static str>,
}

#[derive(Debug, Clone, Hash, Default)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

#[derive(Debug, Clone, Hash)]
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

#[derive(Debug, Clone, Hash)]
pub struct PatIdent {
    pub name: &'static str,
    pub mutable: bool,
}

#[derive(Debug, Clone, Hash)]
pub struct PatTuple {
    pub elements: Vec<Box<Pat>>,
}

#[derive(Debug, Clone, Hash)]
pub struct PatSlice {
    pub elems: Vec<Box<Pat>>,
}

#[derive(Debug, Clone, Hash)]
pub struct PatTupleStruct {
    pub path: Box<Path>,
    pub elems: Vec<Box<Pat>>,
}

#[derive(Debug, Clone, Hash)]
pub struct PatLit {
    pub lit: Box<ExprLit>,
}

#[derive(Debug, Clone, Hash)]
pub struct PatOr {
    pub segments: Vec<Box<Pat>>,
}

#[derive(Debug, Clone, Hash)]
pub struct PatParen {
    pub pat: Box<Pat>,
}

#[derive(Debug, Clone, Hash)]
pub struct PatPath {
    pub path: Box<Path>,
}

#[derive(Debug, Clone, Hash)]
pub struct PatStruct {
    pub path: Box<Path>,
    pub fields: Vec<Box<FieldPat>>,
    pub rest: bool,
}

#[derive(Debug, Clone, Hash)]
pub struct PatType {
    pub pat: Box<Pat>,
    pub kind: Kind,
}

#[derive(Debug, Clone, Hash)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, Hash)]
pub enum ExprKind {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Match(ExprMatch),
    Ret(ExprRet),
    If(ExprIf),
    IfLet(ExprIfLet),
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
    Try(ExprTry),
    Cast(ExprCast),
}

#[derive(Debug, Clone, Hash)]
pub struct ExprCast {
    pub expr: Box<Expr>,
    pub len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprTry {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprType {
    pub kind: Kind,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprBinary {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprMatch {
    pub expr: Box<Expr>,
    pub arms: Vec<Box<Arm>>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprRet {
    pub expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprIf {
    pub cond: Box<Expr>,
    pub then_branch: Box<Block>,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprIfLet {
    pub test: Box<Expr>,
    pub kind: ArmKind,
    pub then_block: Box<Block>,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprParen {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprTuple {
    pub elements: Vec<Box<Expr>>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprForLoop {
    pub pat: Box<Pat>,
    pub expr: Box<Expr>,
    pub body: Box<Block>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprAssign {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprGroup {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprField {
    pub expr: Box<Expr>,
    pub member: Member,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprBlock {
    pub block: Box<Block>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprArray {
    pub elems: Vec<Box<Expr>>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprRange {
    pub start: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub end: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprPath {
    pub path: Box<Path>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprLet {
    pub pattern: Box<Pat>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprRepeat {
    pub value: Box<Expr>,
    pub len: i64,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprStruct {
    pub path: Box<Path>,
    pub fields: Vec<Box<FieldValue>>,
    pub rest: Option<Box<Expr>>,
    pub template: TypedBits,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprBits {
    pub kind: BitsKind,
    pub len: Option<usize>,
    pub arg: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub enum BitsKind {
    Unsigned,
    Signed,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprCall {
    pub path: Box<Path>,
    pub args: Vec<Box<Expr>>,
    pub signature: Option<DigitalSignature>,
    pub code: Option<KernelFnKind>,
}

#[derive(Debug, Clone, Hash)]
pub struct ExprMethodCall {
    pub receiver: Box<Expr>,
    pub args: Vec<Box<Expr>>,
    pub method: &'static str,
    pub turbo: Option<usize>,
}

#[derive(Debug, Clone, Hash)]
pub struct FieldValue {
    pub member: Member,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, Copy, Hash)]
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

#[derive(Debug, Clone, Copy, Hash)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Hash)]
pub struct Arm {
    pub id: NodeId,
    pub kind: ArmKind,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, Hash)]
pub enum ArmKind {
    Wild,
    Constant(ArmConstant),
    Enum(ArmEnum),
}

#[derive(Debug, Clone, Hash)]
pub struct ArmConstant {
    pub value: ExprLit,
}

#[derive(Debug, Clone, Hash)]
pub struct ArmEnum {
    pub pat: Box<Pat>,
    pub discriminant: TypedBits,
}

#[derive(Clone, Hash)]
pub enum ExprLit {
    TypedBits(ExprTypedBits),
    Int(String),
    Bool(bool),
    Empty,
}

#[derive(Clone, Hash)]
pub struct ExprTypedBits {
    pub path: Box<Path>,
    pub value: TypedBits,
    pub code: String,
}

#[derive(Debug, Clone, Hash)]
pub enum RangeLimits {
    HalfOpen,
    Closed,
}

#[derive(Debug, Clone, Hash)]
pub struct FieldPat {
    pub member: Member,
    pub pat: Box<Pat>,
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum KernelFlags {
    AllowWeakPartial,
}

#[derive(Debug, Clone, Hash)]
pub struct KernelFn {
    pub id: NodeId,
    pub name: &'static str,
    pub inputs: Vec<Box<Pat>>,
    pub ret: Kind,
    pub body: Box<Block>,
    pub fn_id: FunctionId,
    pub text: Option<&'static str>,
    pub meta_db: MetaDB,
    pub flags: Vec<KernelFlags>,
}

impl KernelFn {
    pub fn sources(&self) -> Result<SpannedSource, RHDLError> {
        let Some(filename) = self.text else {
            return Err(anyhow!("Kernel function has no source text").into());
        };
        let source = std::fs::read_to_string(filename)
            .map_err(|err| anyhow!("Failed to read source file {}: {}", filename, err))?;
        let span_map = self
            .meta_db
            .iter()
            .map(|(id, meta)| {
                let node_id = NodeId::new(*id);
                let start_col = meta.span.start_col;
                let start_line = meta.span.start_line;
                let end_col = meta.span.end_col;
                let end_line = meta.span.end_line;
                let start_source_offset =
                    miette::SourceOffset::from_location(&source, start_line, start_col + 1);
                let end_source_offset =
                    miette::SourceOffset::from_location(&source, end_line, end_col + 1);
                (
                    node_id,
                    start_source_offset.offset()..end_source_offset.offset(),
                )
            })
            .collect();
        Ok(SpannedSource {
            source,
            name: self.name.into(),
            span_map,
            fallback: self.id,
            filename: filename.into(),
            function_id: self.fn_id,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum WrapOp {
    Ok,
    Err,
    Some,
    None,
}
