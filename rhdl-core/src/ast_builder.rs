use crate::digital::TypedBits;
use crate::digital_fn::DigitalSignature;
use crate::kernel::KernelFnKind;
use crate::{ast::*, Kind};

// Constructor functions
pub fn binary_expr(op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Binary(ExprBinary { op, lhs, rhs }),
    })
}

pub fn unary_expr(op: UnOp, expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Unary(ExprUnary { op, expr }),
    })
}

pub fn assign_expr(lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Assign(ExprAssign { lhs, rhs }),
    })
}

pub fn lit_expr(lit: ExprLit) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Lit(lit),
    })
}

pub fn struct_expr(
    path: Box<Path>,
    fields: Vec<Box<FieldValue>>,
    rest: Option<Box<Expr>>,
    kind: Kind,
) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Struct(ExprStruct {
            path,
            fields,
            rest,
            kind,
        }),
    })
}

pub fn if_expr(
    cond: Box<Expr>,
    then_branch: Box<Block>,
    else_branch: Option<Box<Expr>>,
) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::If(ExprIf {
            cond,
            then_branch,
            else_branch,
        }),
    })
}

pub fn let_expr(pattern: Box<Pat>, value: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Let(ExprLet { pattern, value }),
    })
}

pub fn path_arguments_none() -> Vec<Box<Expr>> {
    vec![]
}

pub fn path_arguments_angle_bracketed(args: Vec<Box<Expr>>) -> Vec<Box<Expr>> {
    args
}

pub fn generic_argument_const(expr: Box<Expr>) -> Box<Expr> {
    expr
}

pub fn generic_argument_type(kind: Kind) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Type(ExprType { kind }),
    })
}

pub fn path_segment(ident: String, arguments: Vec<Box<Expr>>) -> PathSegment {
    PathSegment { ident, arguments }
}

pub fn path(segments: Vec<PathSegment>) -> Box<Path> {
    Box::new(Path { segments })
}

pub fn path_expr(path: Box<Path>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Path(ExprPath { path }),
    })
}

pub fn arm(pattern: Box<Pat>, guard: Option<Box<Expr>>, body: Box<Expr>) -> Box<Arm> {
    Box::new(Arm {
        pattern,
        guard,
        body,
    })
}

pub fn field_expr(expr: Box<Expr>, member: Member) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Field(ExprField { expr, member }),
    })
}

pub fn field_value(member: Member, value: Box<Expr>) -> Box<FieldValue> {
    Box::new(FieldValue { member, value })
}

pub fn match_expr(expr: Box<Expr>, arms: Vec<Box<Arm>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Match(ExprMatch { expr, arms }),
    })
}

pub fn range_expr(
    start: Option<Box<Expr>>,
    limits: RangeLimits,
    end: Option<Box<Expr>>,
) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Range(ExprRange { start, limits, end }),
    })
}

pub fn paren_expr(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Paren(ExprParen { expr }),
    })
}

pub fn group_expr(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Group(ExprGroup { expr }),
    })
}

pub fn tuple_expr(elements: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Tuple(ExprTuple { elements }),
    })
}

pub fn repeat_expr(value: Box<Expr>, len: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Repeat(ExprRepeat { value, len }),
    })
}

pub fn for_expr(pat: Box<Pat>, expr: Box<Expr>, body: Box<Block>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::ForLoop(ExprForLoop { pat, expr, body }),
    })
}

pub fn call_expr(
    path: Box<Path>,
    args: Vec<Box<Expr>>,
    signature: DigitalSignature,
    code: Option<KernelFnKind>,
) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Call(ExprCall {
            path,
            args,
            signature,
            code,
        }),
    })
}

pub fn array_expr(elems: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Array(ExprArray { elems }),
    })
}

pub fn index_expr(expr: Box<Expr>, index: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Index(ExprIndex { expr, index }),
    })
}

pub fn method_expr(receiver: Box<Expr>, args: Vec<Box<Expr>>, method: String) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::MethodCall(ExprMethodCall {
            receiver,
            args,
            method,
        }),
    })
}
pub fn return_expr(expr: Option<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Ret(ExprRet { expr }),
    })
}

pub fn field_pat(member: Member, pat: Box<Pat>) -> Box<FieldPat> {
    Box::new(FieldPat { member, pat })
}

pub fn wild_pat() -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Wild,
    })
}

pub fn lit_pat(lit: ExprLit) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Lit(PatLit { lit: Box::new(lit) }),
    })
}

pub fn type_pat(pat: Box<Pat>, kind: Kind) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Type(PatType { pat, kind }),
    })
}

pub fn struct_pat(path: Box<Path>, fields: Vec<Box<FieldPat>>, rest: bool) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Struct(PatStruct { path, fields, rest }),
    })
}

pub fn path_pat(path: Box<Path>) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Path(PatPath { path }),
    })
}

pub fn slice_pat(elems: Vec<Box<Pat>>) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Slice(PatSlice { elems }),
    })
}

pub fn tuple_pat(elems: Vec<Box<Pat>>) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Tuple(PatTuple { elements: elems }),
    })
}

pub fn tuple_struct_pat(path: Box<Path>, elems: Vec<Box<Pat>>) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::TupleStruct(PatTupleStruct { path, elems }),
    })
}

pub fn ident_pat(name: String, mutable: bool) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Ident(PatIdent { name, mutable }),
    })
}

pub fn local_stmt(pat: Box<Pat>, init: Option<Box<Expr>>) -> Box<Stmt> {
    Box::new(Stmt {
        id: None,
        kind: StmtKind::Local(Box::new(Local {
            id: None,
            pat,
            init,
        })),
    })
}

pub fn semi_stmt(expr: Box<Expr>) -> Box<Stmt> {
    Box::new(Stmt {
        id: None,
        kind: StmtKind::Semi(expr),
    })
}

pub fn expr_stmt(expr: Box<Expr>) -> Box<Stmt> {
    Box::new(Stmt {
        id: None,
        kind: StmtKind::Expr(expr),
    })
}

pub fn block_expr(block: Box<Block>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Block(ExprBlock { block }),
    })
}

pub fn block(stmts: Vec<Box<Stmt>>) -> Box<Block> {
    Box::new(Block { id: None, stmts })
}

pub fn range_limits_half_open() -> RangeLimits {
    RangeLimits::HalfOpen
}

pub fn range_limits_closed() -> RangeLimits {
    RangeLimits::Closed
}

pub fn member_named(name: String) -> Member {
    Member::Named(name)
}

pub fn member_unnamed(index: u32) -> Member {
    Member::Unnamed(index)
}

pub use crate::ast::BinOp;
pub use crate::ast::UnOp;

pub fn expr_lit_int(value: String) -> ExprLit {
    ExprLit::Int(value)
}

pub fn expr_lit_bool(value: bool) -> ExprLit {
    ExprLit::Bool(value)
}

pub fn kernel_fn(name: &str, inputs: Vec<Box<Pat>>, ret: Kind, body: Box<Block>) -> KernelFnKind {
    KernelFnKind::Kernel(Box::new(KernelFn {
        id: None,
        name: name.into(),
        inputs,
        ret,
        body,
    }))
}

pub fn expr_typed_bits(path: Box<Path>, value: TypedBits) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Lit(ExprLit::TypedBits(ExprTypedBits { path, value })),
    })
}
