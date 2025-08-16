use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
pub struct ModuleList(pub Vec<ModuleDef>);

#[derive(Clone, Debug)]
pub enum HDLKind {
    Wire,
    Reg,
}

#[derive(Clone, Debug)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

#[derive(Clone, Debug)]
pub enum SignedWidth {
    Signed(RangeInclusive<u32>),
    Unsigned(RangeInclusive<u32>),
}

#[derive(Clone, Debug)]
pub struct Declaration {
    pub kind: HDLKind,
    pub width: SignedWidth,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub direction: Direction,
    pub decl: Declaration,
}

#[derive(Default, Debug, Clone)]
pub enum Stmt {
    If(If),
    Always(Always),
    Case(Case),
    LocalParam(LocalParam),
    Block(Vec<Stmt>),
    ContinuousAssign(Assign),
    FunctionCall(FunctionCall),
    NonblockAssign(Assign),
    Assign(Assign),
    Instance(Instance),
    Splice(Splice),
    DynamicSplice(DynamicSplice),
    Delay(u32),
    ConcatAssign(ConcatAssign),
    #[default]
    /// Required because the parser for if/else uses it as a placeholder
    Noop,
}

#[derive(Debug, Clone)]
pub struct ConcatAssign {
    pub target: Vec<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct DynamicSplice {
    pub target: String,
    pub base: Box<Expr>,
    pub width: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Splice {
    pub target: String,
    pub msb: Box<Expr>,
    pub lsb: Option<Box<Expr>>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub target: String,
    pub local: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub module: String,
    pub instance: String,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub target: String,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct LocalParam {
    pub target: String,
    pub rhs: LitVerilog,
}

#[derive(Debug, Clone, Copy)]
pub enum BitX {
    Zero,
    One,
    X,
}

#[derive(Debug, Clone)]
pub struct LitVerilog {
    pub width: u32,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum CaseItem {
    Literal(LitVerilog),
    Wild,
}

#[derive(Debug, Clone)]
pub struct CaseLine {
    pub item: CaseItem,
    pub stmt: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub discriminant: Box<Expr>,
    pub lines: Vec<CaseLine>,
}

#[derive(Debug, Clone)]
pub enum Sensitivity {
    PosEdge(String),
    NegEdge(String),
    Signal(String),
    Star,
}

#[derive(Debug, Clone)]
pub struct Always {
    pub sensitivity: Vec<Sensitivity>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Expr>,
    pub true_stmt: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub width: SignedWidth,
    pub name: String,
    pub args: Vec<Port>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Statement(Stmt),
    Declaration(Declaration),
    FunctionDef(FunctionDef),
    Initial(Stmt),
}

#[derive(Clone, Debug)]
pub struct ModuleDef {
    pub name: String,
    pub args: Vec<Port>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Constant(LitVerilog),
    Literal(i32),
    String(String),
    Ident(String),
    Paren(Box<Expr>),
    Ternary(ExprTernary),
    Concat(Vec<Expr>),
    Replica(ExprReplica),
    Index(ExprIndex),
    DynIndex(ExprDynIndex),
    Function(ExprFunction),
}

#[derive(Debug, Clone)]
pub struct ExprDynIndex {
    pub target: String,
    pub base: Box<Expr>,
    pub width: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprIndex {
    pub target: String,
    pub msb: Box<Expr>,
    pub lsb: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprTernary {
    pub lhs: Box<Expr>,
    pub mhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
    pub lhs: Box<Expr>,
    pub op: BinaryOp,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprUnary {
    pub op: UnaryOp,
    pub arg: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprReplica {
    pub count: usize,
    pub concatenation: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprFunction {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Shl,
    SignedRightShift,
    Shr,
    ShortAnd,
    ShortOr,
    CaseEq,
    CaseNe,
    Ne,
    Eq,
    Ge,
    Le,
    Gt,
    Lt,
    Plus,
    Minus,
    And,
    Or,
    Xor,
    Mod,
    Mul,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus,
    Minus,
    Bang,
    Not,
    And,
    Or,
    Xor,
}
