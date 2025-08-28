//! Builder functions for constructing Verilog AST nodes
//!
//! This module provides convenient constructor functions for building Verilog AST structures
//! with idiomatic Rust patterns. Functions follow these conventions:
//! - Expression constructors return `Expr`
//! - Statement constructors (suffixed with `_stmt`) return `StmtKind`
//! - Struct constructors return the specific struct type
//! - Collection parameters use `impl IntoIterator<Item=T>` for flexibility
//! - `impl From` traits provide idiomatic conversions for common cases

use crate::{
    atoms::{NegEdgeSensitivity, PosEdgeSensitivity, Sensitivity, SensitivityList},
    expr::{
        ExprBinary, ExprDynIndexInner, ExprFunction, ExprIndexAddress, ExprReplica,
        ExprReplicaInner, ExprTernary, ExprUnary,
    },
    stmt::{
        Always, Assign, AssignTarget, Block, Case, CaseItem, CaseLine, ConcatAssign, Connection,
        ConstExpr, ContinuousAssign, Delay, DynamicSplice, ElseBranch, FunctionCall, If, Instance,
        LocalParam, NonblockAssign, StmtKind,
    },
    *,
};

// Atom constructors

pub fn bit_range(start: u32, end: u32) -> BitRange {
    BitRange { start, end }
}

pub fn width_spec(bit_range: BitRange) -> WidthSpec {
    WidthSpec { bit_range }
}

pub fn signed_width_spec(width_spec: WidthSpec) -> SignedWidth {
    SignedWidth::Signed(width_spec)
}

pub fn unsigned_width_spec(width_spec: WidthSpec) -> SignedWidth {
    SignedWidth::Unsigned(width_spec)
}

pub fn decl_kind(name: &str, width: Option<SignedWidth>) -> DeclKind {
    DeclKind {
        name: name.to_string(),
        width,
    }
}

pub fn connection(target: &str, local: Expr) -> Connection {
    Connection {
        target: target.to_string(),
        local: Box::new(local),
    }
}

pub fn lit_verilog(width: u32, value: &str) -> LitVerilog {
    LitVerilog {
        width,
        value: value.to_string(),
    }
}

// =============================================================================
// Impl From traits for idiomatic conversions
// =============================================================================

impl From<&str> for Expr {
    fn from(name: &str) -> Self {
        Expr::Ident(name.to_string())
    }
}

impl From<String> for Expr {
    fn from(name: String) -> Self {
        Expr::Ident(name)
    }
}

impl From<LitVerilog> for Expr {
    fn from(value: LitVerilog) -> Self {
        Expr::Constant(value)
    }
}

impl From<StmtKind> for Stmt {
    fn from(kind: StmtKind) -> Self {
        Stmt { kind }
    }
}

impl From<Stmt> for Initial {
    fn from(statement: Stmt) -> Self {
        Initial { statement }
    }
}

impl From<u64> for Delay {
    fn from(length: u64) -> Self {
        Delay { length }
    }
}

impl From<Stmt> for ElseBranch {
    fn from(stmt: Stmt) -> Self {
        ElseBranch {
            stmt: Box::new(stmt),
        }
    }
}

impl From<BitRange> for WidthSpec {
    fn from(bit_range: BitRange) -> Self {
        WidthSpec { bit_range }
    }
}

impl From<WidthSpec> for SignedWidth {
    fn from(width_spec: WidthSpec) -> Self {
        SignedWidth::Unsigned(width_spec)
    }
}

// Declaration constructors

pub fn declaration(kind: HDLKind, signed_width: Option<SignedWidth>, name: &str) -> Declaration {
    Declaration {
        kind,
        signed_width,
        name: name.to_string(),
    }
}

pub fn declaration_list(
    kind: HDLKind,
    signed_width: Option<SignedWidth>,
    items: impl IntoIterator<Item = DeclKind>,
) -> DeclarationList {
    DeclarationList {
        kind,
        signed_width,
        items: items.into_iter().collect(),
    }
}

pub fn port(direction: Direction, decl: Declaration) -> Port {
    Port { direction, decl }
}

pub fn posedge(ident: &str) -> PosEdgeSensitivity {
    PosEdgeSensitivity {
        ident: ident.to_string(),
    }
}

pub fn negedge(ident: &str) -> NegEdgeSensitivity {
    NegEdgeSensitivity {
        ident: ident.to_string(),
    }
}

pub fn sensitivity_list(elements: impl IntoIterator<Item = Sensitivity>) -> SensitivityList {
    SensitivityList {
        elements: elements.into_iter().collect(),
    }
}

// =============================================================================
// Expression constructor functions
// =============================================================================

/// Creates a repeat expression {count{target}}
pub fn repeat(target: Expr, count: usize) -> Expr {
    Expr::Replica(ExprReplica {
        inner: ExprReplicaInner {
            count: count as u32,
            concatenation: ExprConcat {
                elements: vec![target],
            },
        },
    })
}

/// Creates a single bit index expression target[bit]
pub fn index_bit(target: &str, bit: usize) -> Expr {
    Expr::Index(ExprIndex {
        target: target.to_string(),
        address: ExprIndexAddress {
            msb: Box::new(Expr::Literal(bit as i32)),
            lsb: None,
        },
    })
}

/// Creates a binary expression with an operator
pub fn binary(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary(ExprBinary {
        lhs: Box::new(lhs),
        op,
        rhs: Box::new(rhs),
    })
}

/// Creates a unary expression with an operator
pub fn unary(op: UnaryOp, expr: Expr) -> Expr {
    Expr::Unary(ExprUnary {
        op,
        arg: Box::new(expr),
    })
}

/// Creates a concatenation expression from multiple expressions
pub fn concatenate(args: impl IntoIterator<Item = Expr>) -> Expr {
    Expr::Concat(ExprConcat {
        elements: args.into_iter().collect(),
    })
}

/// Creates an index expression for array/bit access
pub fn index_expr(target: &str, msb: Expr, lsb: Option<Box<Expr>>) -> Expr {
    Expr::Index(ExprIndex {
        target: target.to_string(),
        address: ExprIndexAddress {
            msb: Box::new(msb),
            lsb,
        },
    })
}

/// Creates a ternary expression (condition ? true_expr : false_expr)
pub fn ternary(condition: Expr, true_expr: Expr, false_expr: Expr) -> Expr {
    Expr::Ternary(ExprTernary {
        lhs: Box::new(condition),
        mhs: Box::new(true_expr),
        rhs: Box::new(false_expr),
    })
}

/// Creates a function call expression
pub fn function_expr(name: &str, args: impl IntoIterator<Item = Expr>) -> Expr {
    Expr::Function(ExprFunction {
        name: name.to_string(),
        args: args.into_iter().collect(),
    })
}

/// Creates a parenthesized expression
pub fn paren(expr: Expr) -> Expr {
    Expr::Paren(Box::new(expr))
}

/// Creates a dynamic index expression
pub fn dyn_index(target: &str, base: Expr, op: DynOp, width: Expr) -> Expr {
    Expr::DynIndex(ExprDynIndex {
        target: target.to_string(),
        address: ExprDynIndexInner {
            base: Box::new(base),
            op,
            width: Box::new(width),
        },
    })
}

/// Creates a replica expression {count{expr}}
pub fn replica_expr(count: u32, inner: ExprConcat) -> Expr {
    Expr::Replica(ExprReplica {
        inner: ExprReplicaInner {
            count,
            concatenation: inner,
        },
    })
}

/// Creates a literal integer expression
pub fn literal(value: i32) -> Expr {
    Expr::Literal(value)
}

/// Creates a string expression
pub fn string_expr(value: &str) -> Expr {
    Expr::String(value.to_string())
}

pub fn case_line(item: CaseItem, stmt: Stmt) -> CaseLine {
    CaseLine {
        item,
        stmt: Box::new(stmt),
    }
}

// =============================================================================
// Statement constructor functions
// =============================================================================

/// Creates an assign statement
pub fn assign_stmt(target: AssignTarget, rhs: Expr) -> StmtKind {
    StmtKind::Assign(Assign {
        target,
        rhs: Box::new(rhs),
    })
}

/// Creates a function call statement
pub fn function_call_stmt(name: &str, args: impl IntoIterator<Item = Expr>) -> StmtKind {
    StmtKind::FunctionCall(FunctionCall {
        name: name.to_string(),
        args: args.into_iter().collect(),
    })
}

/// Creates a delay statement
pub fn delay_stmt(length: u64) -> StmtKind {
    StmtKind::Delay(Delay { length })
}

/// Creates a nonblocking assign statement
pub fn nonblock_assign_stmt(target: AssignTarget, rhs: Expr) -> StmtKind {
    StmtKind::NonblockAssign(NonblockAssign {
        target,
        rhs: Box::new(rhs),
    })
}

/// Creates a case statement
pub fn case_stmt(discriminant: Expr, lines: impl IntoIterator<Item = CaseLine>) -> StmtKind {
    StmtKind::Case(Case {
        discriminant: Box::new(discriminant),
        lines: lines.into_iter().collect(),
    })
}

/// Creates a local parameter declaration
pub fn local_param_stmt(target: &str, rhs: ConstExpr) -> StmtKind {
    StmtKind::LocalParam(LocalParam {
        target: target.to_string(),
        rhs,
    })
}

/// Creates a continuous assign statement
pub fn continuous_assign_stmt(target: AssignTarget, rhs: Expr) -> StmtKind {
    StmtKind::ContinuousAssign(ContinuousAssign {
        assign: Assign {
            target,
            rhs: Box::new(rhs),
        },
    })
}

/// Creates a block statement (begin...end)
pub fn block_stmt(body: impl IntoIterator<Item = Stmt>) -> StmtKind {
    StmtKind::Block(Block {
        body: body.into_iter().collect(),
    })
}

/// Creates an instance statement for module instantiation
pub fn instance_stmt(
    module: &str,
    instance_name: &str,
    connections: impl IntoIterator<Item = Connection>,
) -> StmtKind {
    StmtKind::Instance(Instance {
        module: module.to_string(),
        instance: instance_name.to_string(),
        connections: connections.into_iter().collect(),
    })
}

/// Creates a concat assign statement
pub fn concat_assign_stmt(target: ExprConcat, rhs: Expr) -> StmtKind {
    StmtKind::ConcatAssign(ConcatAssign {
        target,
        rhs: Box::new(rhs),
    })
}

/// Creates a dynamic splice statement
pub fn dynamic_splice_stmt(lhs: ExprDynIndex, rhs: Expr) -> StmtKind {
    StmtKind::DynamicSplice(DynamicSplice {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

/// Creates an if statement  
pub fn if_stmt(condition: Expr, true_stmt: Stmt, else_branch: Option<ElseBranch>) -> StmtKind {
    StmtKind::If(If {
        condition: Box::new(condition),
        true_stmt: Box::new(true_stmt),
        else_branch,
    })
}

/// Creates an always statement
pub fn always_stmt(sensitivity: SensitivityList, body: Stmt) -> StmtKind {
    StmtKind::Always(Always {
        sensitivity,
        body: Box::new(body),
    })
}

/// Creates a no-operation statement
pub fn noop_stmt() -> StmtKind {
    StmtKind::Noop
}

pub fn function_call(name: &str, args: impl IntoIterator<Item = Expr>) -> FunctionCall {
    FunctionCall {
        name: name.to_string(),
        args: args.into_iter().collect(),
    }
}

// Module functions

pub fn module_list(modules: impl IntoIterator<Item = ModuleDef>) -> ModuleList {
    ModuleList {
        modules: modules.into_iter().collect(),
    }
}

pub fn module_def(
    name: &str,
    args: impl IntoIterator<Item = Port>,
    items: impl IntoIterator<Item = Item>,
) -> ModuleDef {
    ModuleDef {
        name: name.to_string(),
        args: args.into_iter().collect(),
        items: items.into_iter().collect(),
    }
}

pub fn function_def(
    signed_width: SignedWidth,
    name: &str,
    args: impl IntoIterator<Item = Port>,
    items: impl IntoIterator<Item = Item>,
) -> FunctionDef {
    FunctionDef {
        signed_width,
        name: name.to_string(),
        args: args.into_iter().collect(),
        items: items.into_iter().collect(),
    }
}
