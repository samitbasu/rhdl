use crate::{ast::*, Kind};

// Constructor functions
pub fn binary_expr(op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Binary { op, lhs, rhs },
    })
}

pub fn unary_expr(op: UnOp, expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Unary { op, expr },
    })
}

pub fn assign_expr(lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Assign { lhs, rhs },
    })
}

pub fn lit_expr(lit: ExprLit) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Lit { lit: Box::new(lit) },
    })
}

pub fn struct_expr(
    path: Box<Path>,
    fields: Vec<Box<FieldValue>>,
    rest: Option<Box<Expr>>,
) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Struct { path, fields, rest },
    })
}

pub fn if_expr(
    cond: Box<Expr>,
    then_branch: Box<Block>,
    else_branch: Option<Box<Expr>>,
) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::If {
            cond,
            then_branch,
            else_branch,
        },
    })
}

pub fn let_expr(pattern: Box<Pat>, value: Box<Expr>, body: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Let {
            pattern,
            value,
            body,
        },
    })
}

pub fn path_segment(ident: String) -> PathSegment {
    PathSegment { id: None, ident }
}

pub fn path(segments: Vec<PathSegment>) -> Box<Path> {
    Box::new(Path { segments })
}

pub fn path_expr(path: Box<Path>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Path { path },
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
        kind: ExprKind::Field { expr, member },
    })
}

pub fn field_value(member: Member, value: Box<Expr>) -> Box<FieldValue> {
    Box::new(FieldValue { member, value })
}

pub fn match_expr(expr: Box<Expr>, arms: Vec<Box<Arm>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Match { expr, arms },
    })
}

pub fn range_expr(
    start: Option<Box<Expr>>,
    limits: RangeLimits,
    end: Option<Box<Expr>>,
) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Range { start, limits, end },
    })
}

pub fn paren_expr(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Paren { expr },
    })
}

pub fn group_expr(expr: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Group { expr },
    })
}

pub fn tuple_expr(elements: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Tuple { elements },
    })
}

pub fn repeat_expr(value: Box<Expr>, len: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Repeat { value, len },
    })
}

pub fn for_expr(pat: Box<Pat>, expr: Box<Expr>, body: Box<Block>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::ForLoop { pat, expr, body },
    })
}

pub fn call_expr(path: Box<Path>, args: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Call { path, args },
    })
}

pub fn array_expr(elems: Vec<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Array { elems },
    })
}

pub fn index_expr(expr: Box<Expr>, index: Box<Expr>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Index { expr, index },
    })
}

pub fn method_expr(receiver: Box<Expr>, args: Vec<Box<Expr>>, method: String) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::MethodCall {
            receiver,
            args,
            method,
        },
    })
}
pub fn return_expr(expr: Option<Box<Expr>>) -> Box<Expr> {
    Box::new(Expr {
        id: None,
        kind: ExprKind::Ret { expr },
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
        kind: PatKind::Lit { lit: Box::new(lit) },
    })
}

pub fn type_pat(pat: Box<Pat>, kind: Kind) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Type { pat, kind },
    })
}

pub fn struct_pat(path: Box<Path>, fields: Vec<Box<FieldPat>>, rest: bool) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Struct { path, fields, rest },
    })
}

pub fn path_pat(path: Box<Path>) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Path { path },
    })
}

pub fn tuple_pat(elems: Vec<Box<Pat>>) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Tuple { elements: elems },
    })
}

pub fn tuple_struct_pat(path: Box<Path>, elems: Vec<Box<Pat>>) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::TupleStruct { path, elems },
    })
}

pub fn ident_pat(name: String, mutable: bool) -> Box<Pat> {
    Box::new(Pat {
        id: None,
        kind: PatKind::Ident { name, mutable },
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
        kind: ExprKind::Block { block },
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
