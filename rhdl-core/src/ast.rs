use crate::Kind;

// Modeled after rustc's AST

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeId(u32);

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
    pub id: Option<NodeId>,
    pub ident: String,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

#[derive(Debug, Clone)]
pub enum PatKind {
    Ident {
        name: String,
        mutable: bool,
    },
    Tuple {
        elements: Vec<Box<Pat>>,
    },
    TupleStruct {
        path: Box<Path>,
        elems: Vec<Box<Pat>>,
    },
    Lit {
        lit: Box<ExprLit>,
    },
    Or {
        segments: Vec<Box<Pat>>,
    },
    Paren {
        pat: Box<Pat>,
    },
    Path {
        path: Box<Path>,
    },
    Struct {
        path: Box<Path>,
        fields: Vec<Box<FieldPat>>,
        rest: bool,
    },
    Type {
        pat: Box<Pat>,
        kind: Kind,
    },
    Wild,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: Option<NodeId>,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<Box<Arm>>,
    },
    Ret {
        expr: Option<Box<Expr>>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Block>,
        else_branch: Option<Box<Expr>>,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },
    Lit {
        lit: Box<ExprLit>,
    },
    Paren {
        expr: Box<Expr>,
    },
    Tuple {
        elements: Vec<Box<Expr>>,
    },
    ForLoop {
        pat: Box<Pat>,
        expr: Box<Expr>,
        body: Box<Block>,
    },
    Assign {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Group {
        expr: Box<Expr>,
    },
    Field {
        expr: Box<Expr>,
        member: Member,
    },
    Block {
        block: Box<Block>,
    },
    Array {
        elems: Vec<Box<Expr>>,
    },
    Range {
        start: Option<Box<Expr>>,
        limits: RangeLimits,
        end: Option<Box<Expr>>,
    },
    Path {
        path: Box<Path>,
    },
    Let {
        pattern: Box<Pat>,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Repeat {
        value: Box<Expr>,
        len: Box<Expr>,
    },
    Struct {
        path: Box<Path>,
        fields: Vec<Box<FieldValue>>,
        rest: Option<Box<Expr>>,
    },
    Call {
        path: Box<Path>,
        args: Vec<Box<Expr>>,
    },
    MethodCall {
        receiver: Box<Expr>,
        args: Vec<Box<Expr>>,
        method: String,
    },
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
