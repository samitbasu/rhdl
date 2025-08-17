#[derive(Clone, Debug)]
pub struct ModuleList(pub Vec<ModuleDef>);

pub fn module_list(modules: Vec<ModuleDef>) -> ModuleList {
    ModuleList(modules)
}

use std::ops::RangeInclusive;
#[derive(Clone, Debug)]
pub enum HDLKind {
    Wire,
    Reg,
}

pub fn wire() -> HDLKind {
    HDLKind::Wire
}

pub fn reg() -> HDLKind {
    HDLKind::Reg
}

#[derive(Clone, Debug)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

pub fn input() -> Direction {
    Direction::Input
}

pub fn output() -> Direction {
    Direction::Output
}

pub fn inout() -> Direction {
    Direction::Inout
}

#[derive(Clone, Debug)]
pub enum SignedWidth {
    Signed(RangeInclusive<u32>),
    Unsigned(RangeInclusive<u32>),
}

pub fn signed(range: RangeInclusive<u32>) -> SignedWidth {
    SignedWidth::Signed(range)
}

pub fn unsigned(range: RangeInclusive<u32>) -> SignedWidth {
    SignedWidth::Unsigned(range)
}

#[derive(Clone, Debug)]
pub struct Declaration {
    pub kind: HDLKind,
    pub signed_width: SignedWidth,
    pub name: String,
}

pub fn declaration(kind: HDLKind, signed_width: SignedWidth, name: &str) -> Declaration {
    Declaration {
        kind,
        signed_width,
        name: name.to_string(),
    }
}

#[derive(Clone, Debug)]
pub struct Port {
    pub direction: Direction,
    pub decl: Declaration,
}

pub fn port(direction: Direction, decl: Declaration) -> Port {
    Port { direction, decl }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub target: String,
    pub local: Box<Expr>,
}

pub fn connection(target: &str, local: Expr) -> Connection {
    Connection {
        target: target.to_string(),
        local: Box::new(local),
    }
}

#[derive(Debug, Clone)]
pub enum Sensitivity {
    PosEdge(String),
    NegEdge(String),
    Signal(String),
    Star,
}

pub fn pos_edge(signal: &str) -> Sensitivity {
    Sensitivity::PosEdge(signal.to_string())
}

pub fn neg_edge(signal: &str) -> Sensitivity {
    Sensitivity::NegEdge(signal.to_string())
}

pub fn signal(signal: &str) -> Sensitivity {
    Sensitivity::Signal(signal.to_string())
}

pub fn star() -> Sensitivity {
    Sensitivity::Star
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

pub fn binary_shl() -> BinaryOp {
    BinaryOp::Shl
}

pub fn binary_signed_right_shift() -> BinaryOp {
    BinaryOp::SignedRightShift
}

pub fn binary_shr() -> BinaryOp {
    BinaryOp::Shr
}

pub fn binary_short_and() -> BinaryOp {
    BinaryOp::ShortAnd
}

pub fn binary_short_or() -> BinaryOp {
    BinaryOp::ShortOr
}

pub fn binary_case_eq() -> BinaryOp {
    BinaryOp::CaseEq
}

pub fn binary_case_ne() -> BinaryOp {
    BinaryOp::CaseNe
}

pub fn binary_ne() -> BinaryOp {
    BinaryOp::Ne
}

pub fn binary_eq() -> BinaryOp {
    BinaryOp::Eq
}

pub fn binary_ge() -> BinaryOp {
    BinaryOp::Ge
}

pub fn binary_le() -> BinaryOp {
    BinaryOp::Le
}

pub fn binary_gt() -> BinaryOp {
    BinaryOp::Gt
}

pub fn binary_lt() -> BinaryOp {
    BinaryOp::Lt
}

pub fn binary_plus() -> BinaryOp {
    BinaryOp::Plus
}

pub fn binary_minus() -> BinaryOp {
    BinaryOp::Minus
}

pub fn binary_and() -> BinaryOp {
    BinaryOp::And
}

pub fn binary_or() -> BinaryOp {
    BinaryOp::Or
}

pub fn binary_xor() -> BinaryOp {
    BinaryOp::Xor
}

pub fn binary_mod() -> BinaryOp {
    BinaryOp::Mod
}

pub fn binary_mul() -> BinaryOp {
    BinaryOp::Mul
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

pub fn unary_plus() -> UnaryOp {
    UnaryOp::Plus
}

pub fn unary_minus() -> UnaryOp {
    UnaryOp::Minus
}

pub fn unary_bang() -> UnaryOp {
    UnaryOp::Bang
}

pub fn unary_not() -> UnaryOp {
    UnaryOp::Not
}

pub fn unary_and() -> UnaryOp {
    UnaryOp::And
}

pub fn unary_or() -> UnaryOp {
    UnaryOp::Or
}

pub fn unary_xor() -> UnaryOp {
    UnaryOp::Xor
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

pub fn if_stmt(condition: Expr, true_stmt: Stmt, else_branch: Option<Stmt>) -> Stmt {
    Stmt::If(If {
        condition: Box::new(condition),
        true_stmt: Box::new(true_stmt),
        else_branch: else_branch.map(Box::new),
    })
}

pub fn always_stmt(sensitivity: Vec<Sensitivity>, body: Stmt) -> Stmt {
    Stmt::Always(Always {
        sensitivity,
        body: Box::new(body),
    })
}

pub fn case_stmt(discriminant: Expr, lines: Vec<CaseLine>) -> Stmt {
    Stmt::Case(Case {
        discriminant: Box::new(discriminant),
        lines,
    })
}

pub fn local_param_stmt(target: &str, rhs: LitVerilog) -> Stmt {
    Stmt::LocalParam(LocalParam {
        target: target.to_string(),
        rhs,
    })
}

pub fn block_stmt(stmts: Vec<Stmt>) -> Stmt {
    Stmt::Block(stmts)
}

pub fn continuous_assign_stmt(target: &str, rhs: Expr) -> Stmt {
    Stmt::ContinuousAssign(Assign {
        target: target.to_string(),
        rhs: Box::new(rhs),
    })
}

pub fn function_call_stmt(name: &str, args: Vec<Expr>) -> Stmt {
    Stmt::FunctionCall(FunctionCall {
        name: name.to_string(),
        args,
    })
}

pub fn nonblock_assign_stmt(target: &str, rhs: Expr) -> Stmt {
    Stmt::NonblockAssign(Assign {
        target: target.to_string(),
        rhs: Box::new(rhs),
    })
}

pub fn assign_stmt(target: &str, rhs: Expr) -> Stmt {
    Stmt::Assign(Assign {
        target: target.to_string(),
        rhs: Box::new(rhs),
    })
}

pub fn instance_stmt(module: &str, instance: &str, connections: Vec<Connection>) -> Stmt {
    Stmt::Instance(Instance {
        module: module.to_string(),
        instance: instance.to_string(),
        connections,
    })
}

pub fn splice_stmt(target: &str, msb: Expr, lsb: Option<Expr>, rhs: Expr) -> Stmt {
    Stmt::Splice(Splice {
        target: target.to_string(),
        msb: Box::new(msb),
        lsb: lsb.map(Box::new),
        rhs: Box::new(rhs),
    })
}

pub fn dynamic_splice_stmt(target: &str, base: Expr, width: Expr, rhs: Expr) -> Stmt {
    Stmt::DynamicSplice(DynamicSplice {
        target: target.to_string(),
        base: Box::new(base),
        width: Box::new(width),
        rhs: Box::new(rhs),
    })
}

pub fn delay_stmt(delay: u32) -> Stmt {
    Stmt::Delay(delay)
}

pub fn concat_assign_stmt(target: Vec<Expr>, rhs: Expr) -> Stmt {
    Stmt::ConcatAssign(ConcatAssign {
        target,
        rhs: Box::new(rhs),
    })
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

#[derive(Debug, Clone)]
pub struct LitVerilog {
    pub width: u32,
    pub value: String,
}

pub fn lit_verilog(width: u32, value: &str) -> LitVerilog {
    LitVerilog {
        width,
        value: value.to_string(),
    }
}

#[derive(Debug, Clone)]
pub enum CaseItem {
    Literal(LitVerilog),
    Wild,
}

pub fn case_item_literal(lit: LitVerilog) -> CaseItem {
    CaseItem::Literal(lit)
}

pub fn case_item_wild() -> CaseItem {
    CaseItem::Wild
}

#[derive(Debug, Clone)]
pub struct CaseLine {
    pub item: CaseItem,
    pub stmt: Box<Stmt>,
}

pub fn case_line(item: CaseItem, stmt: Stmt) -> CaseLine {
    CaseLine {
        item,
        stmt: Box::new(stmt),
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub discriminant: Box<Expr>,
    pub lines: Vec<CaseLine>,
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

pub fn function_def(
    width: SignedWidth,
    name: &str,
    args: Vec<Port>,
    items: Vec<Item>,
) -> FunctionDef {
    FunctionDef {
        width,
        name: name.to_string(),
        args,
        items,
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Statement(Stmt),
    Declaration(Declaration),
    FunctionDef(FunctionDef),
    Initial(Stmt),
}

pub fn stmt_item(stmt: Stmt) -> Item {
    Item::Statement(stmt)
}

pub fn declaration_item(decl: Declaration) -> Item {
    Item::Declaration(decl)
}

pub fn function_def_item(func: FunctionDef) -> Item {
    Item::FunctionDef(func)
}

pub fn initial_item(initial: Stmt) -> Item {
    Item::Initial(initial)
}

#[derive(Clone, Debug)]
pub struct ModuleDef {
    pub name: String,
    pub args: Vec<Port>,
    pub items: Vec<Item>,
}

pub fn module_def(name: &str, args: Vec<Port>, items: Vec<Item>) -> ModuleDef {
    ModuleDef {
        name: name.to_string(),
        args,
        items,
    }
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

pub fn binary_expr(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
    Expr::Binary(ExprBinary {
        lhs: Box::new(lhs),
        op,
        rhs: Box::new(rhs),
    })
}

pub fn unary_expr(op: UnaryOp, arg: Expr) -> Expr {
    Expr::Unary(ExprUnary {
        op,
        arg: Box::new(arg),
    })
}

pub fn constant_expr(lit: LitVerilog) -> Expr {
    Expr::Constant(lit)
}

pub fn literal_expr(value: i32) -> Expr {
    Expr::Literal(value)
}

pub fn string_expr(value: &str) -> Expr {
    Expr::String(value.to_string())
}

pub fn ident_expr(name: &str) -> Expr {
    Expr::Ident(name.to_string())
}

pub fn paren_expr(expr: Expr) -> Expr {
    Expr::Paren(Box::new(expr))
}

pub fn ternary_expr(condition: Expr, true_expr: Expr, false_expr: Expr) -> Expr {
    Expr::Ternary(ExprTernary {
        lhs: Box::new(condition),
        mhs: Box::new(true_expr),
        rhs: Box::new(false_expr),
    })
}

pub fn concat_expr(exprs: Vec<Expr>) -> Expr {
    Expr::Concat(exprs)
}

pub fn replica_expr(count: usize, concatenation: Vec<Expr>) -> Expr {
    Expr::Replica(ExprReplica {
        count,
        concatenation,
    })
}

pub fn index_expr(target: &str, msb: Expr, lsb: Option<Expr>) -> Expr {
    Expr::Index(ExprIndex {
        target: target.to_string(),
        msb: Box::new(msb),
        lsb: lsb.map(Box::new),
    })
}

pub fn dyn_index_expr(target: &str, base: Expr, width: Expr) -> Expr {
    Expr::DynIndex(ExprDynIndex {
        target: target.to_string(),
        base: Box::new(base),
        width: Box::new(width),
    })
}

pub fn function_expr(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Function(ExprFunction {
        name: name.to_string(),
        args,
    })
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
