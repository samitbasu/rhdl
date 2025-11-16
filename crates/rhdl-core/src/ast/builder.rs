//! AST Builder functions
//!
//! In order to isolate the data structures used to represent the AST from
//! the rest of the code base, we provide a set of builder functions that
//! can be used to construct the various AST nodes.  This allows us to change
//! the underlying data structures without affecting the rest of the code base.
//!
//! The proc macro uses these builder functions to construct the AST for
//! the kernel functions.
use std::hash::{Hash, Hasher};

use rhdl_span::MetaDB;

pub use crate::ast::ast_impl::BinOp;
pub use crate::ast::ast_impl::UnOp;
use crate::kernel::KernelFnKind;
use crate::rhif::spec::Member;
use crate::types::typed_bits::TypedBits;
use crate::{Color, Digital, DigitalSignature};
use crate::{Kind, ast::ast_impl::*};

/// Build a binary expression node
///
/// `lhs op rhs`
pub fn binary_expr(id: NodeId, op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Binary(ExprBinary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }),
    }
}

/// Build a unary expression node
///
/// `op expr`
pub fn unary_expr(id: NodeId, op: UnOp, expr: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Unary(ExprUnary {
            op,
            expr: Box::new(expr),
        }),
    }
}

/// Build an assignment expression node
///
/// `lhs = rhs`
pub fn assign_expr(id: NodeId, lhs: Expr, rhs: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Assign(ExprAssign {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }),
    }
}

/// Build a literal expression node
///
/// `lit`
pub fn lit_expr(id: NodeId, lit: ExprLit) -> Expr {
    Expr {
        id,
        kind: ExprKind::Lit(lit),
    }
}

/// Build a struct expression node
///
/// `
/// Path {
///     field1: value1,
///     field2: value2,
///     ..rest
/// }
/// `
pub fn struct_expr(
    id: NodeId,
    path: Path,
    fields: Vec<FieldValue>,
    rest: Option<Expr>,
    template: TypedBits,
) -> Expr {
    Expr {
        id,
        kind: ExprKind::Struct(ExprStruct {
            path,
            fields,
            rest: rest.map(Box::new),
            template,
        }),
    }
}

/// Build an if expression node
///
/// `
/// if cond { then_branch } else { else_branch }
/// `
pub fn if_expr(id: NodeId, cond: Expr, then_branch: Block, else_branch: Option<Expr>) -> Expr {
    Expr {
        id,
        kind: ExprKind::If(ExprIf {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }),
    }
}

/// Build an if-let expression node
///
/// `
/// if let pattern = test { then_block } else { else_branch }
/// `
pub fn if_let_expr(
    id: NodeId,
    test: Expr,
    kind: ArmKind,
    then_block: Block,
    else_branch: Option<Expr>,
) -> Expr {
    Expr {
        id,
        kind: ExprKind::IfLet(ExprIfLet {
            test: Box::new(test),
            kind,
            then_block: Box::new(then_block),
            else_branch: else_branch.map(Box::new),
        }),
    }
}

/// Build an empty path arguments list (no generic arguments)
pub fn path_arguments_none() -> Vec<&'static str> {
    vec![]
}

/// Build a path segment
pub fn path_segment(ident: &'static str, arguments: Vec<&'static str>) -> PathSegment {
    PathSegment { ident, arguments }
}

/// Build a path
pub fn path(segments: Vec<PathSegment>) -> Path {
    Path { segments }
}

/// Build a path expression node
pub fn path_expr(id: NodeId, path: Path) -> Expr {
    Expr {
        id,
        kind: ExprKind::Path(ExprPath { path }),
    }
}

/// Build an arm kind representing a wildcard pattern
pub fn arm_kind_wild() -> ArmKind {
    ArmKind::Wild
}

/// Build an arm kind representing a constant pattern
pub fn arm_kind_constant(value: ExprLit) -> ArmKind {
    ArmKind::Constant(ArmConstant { value })
}

/// Build an arm kind representing an enum pattern
pub fn arm_kind_enum(pat: Pat, discriminant: TypedBits) -> ArmKind {
    ArmKind::Enum(ArmEnum { pat, discriminant })
}

/// Build an arm kind that matches `None`
pub fn arm_kind_none(id: NodeId) -> ArmKind {
    ArmKind::Enum(ArmEnum {
        pat: wild_pat(id),
        discriminant: false.typed_bits(),
    })
}

/// Build an arm with the given kind and body
///
/// `kind => body`
pub fn arm(id: NodeId, kind: ArmKind, body: Expr) -> Arm {
    Arm {
        id,
        kind,
        body: Box::new(body),
    }
}

/// Build a field expression node
///
/// `expr.member`
pub fn field_expr(id: NodeId, expr: Expr, member: Member) -> Expr {
    Expr {
        id,
        kind: ExprKind::Field(ExprField {
            expr: Box::new(expr),
            member,
        }),
    }
}

/// Build a field value node
///
/// `member: value`
pub fn field_value(member: Member, value: Expr) -> FieldValue {
    FieldValue {
        member,
        value: Box::new(value),
    }
}

/// Build a match expression node
/// `
/// match expr {
///     arm1,
///     arm2,
///     ...
/// }
/// `
pub fn match_expr(id: NodeId, expr: Expr, arms: Vec<Arm>) -> Expr {
    Expr {
        id,
        kind: ExprKind::Match(ExprMatch {
            expr: Box::new(expr),
            arms,
        }),
    }
}

/// Build a range expression node
///
/// `start..end` or `..end` or `start..` or `start..=end` etc.
pub fn range_expr(id: NodeId, start: Option<Expr>, limits: RangeLimits, end: Option<Expr>) -> Expr {
    Expr {
        id,
        kind: ExprKind::Range(ExprRange {
            start: start.map(Box::new),
            limits,
            end: end.map(Box::new),
        }),
    }
}

/// Build a parenthesized expression node
///
/// `(expr)`
pub fn paren_expr(id: NodeId, expr: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Paren(ExprParen {
            expr: Box::new(expr),
        }),
    }
}

/// Build a grouped expression node
///
/// `{ expr }`
pub fn group_expr(id: NodeId, expr: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Group(ExprGroup {
            expr: Box::new(expr),
        }),
    }
}

/// Build a tuple expression node
///
/// `(elem1, elem2, ...)`
pub fn tuple_expr(id: NodeId, elements: Vec<Expr>) -> Expr {
    Expr {
        id,
        kind: ExprKind::Tuple(ExprTuple { elements }),
    }
}

/// Build a repeat expression node
///
/// `[value; len]`
pub fn repeat_expr(id: NodeId, value: Expr, len: i64) -> Expr {
    Expr {
        id,
        kind: ExprKind::Repeat(ExprRepeat {
            value: Box::new(value),
            len,
        }),
    }
}

/// Build a for loop expression node
///
/// `for pat in expr { body }`
pub fn for_expr(id: NodeId, pat: Pat, expr: Expr, body: Block) -> Expr {
    Expr {
        id,
        kind: ExprKind::ForLoop(ExprForLoop {
            pat,
            expr: Box::new(expr),
            body: Box::new(body),
        }),
    }
}

/// Build a call expression node
///
/// `path(arg1, arg2, ...)`
pub fn call_expr(
    id: NodeId,
    path: Path,
    args: Vec<Expr>,
    signature: DigitalSignature,
    code: Option<KernelFnKind>,
) -> Expr {
    Expr {
        id,
        kind: ExprKind::Call(ExprCall {
            path,
            args,
            signature: Some(signature),
            code,
        }),
    }
}

/// Build an array expression node
///
/// `[elem1, elem2, ...]`
pub fn array_expr(id: NodeId, elems: Vec<Expr>) -> Expr {
    Expr {
        id,
        kind: ExprKind::Array(ExprArray { elems }),
    }
}

/// Build an index expression node
///
/// `expr[index]`
pub fn index_expr(id: NodeId, expr: Expr, index: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Index(ExprIndex {
            expr: Box::new(expr),
            index: Box::new(index),
        }),
    }
}

/// Build a method call expression node
///
/// `receiver.method(arg1, arg2, ...)`
pub fn method_expr(
    id: NodeId,
    receiver: Expr,
    args: Vec<Expr>,
    method: &'static str,
    turbo: Option<usize>,
) -> Expr {
    Expr {
        id,
        kind: ExprKind::MethodCall(ExprMethodCall {
            receiver: Box::new(receiver),
            args,
            method,
            turbo,
        }),
    }
}

/// Build a return expression node
///
/// `return expr`
pub fn return_expr(id: NodeId, expr: Option<Expr>) -> Expr {
    Expr {
        id,
        kind: ExprKind::Ret(ExprRet {
            expr: expr.map(Box::new),
        }),
    }
}

/// Build a field pattern node
///
/// `member: pat`
pub fn field_pat(member: Member, pat: Pat) -> FieldPat {
    FieldPat {
        member,
        pat: Box::new(pat),
    }
}

/// Build a wildcard pattern node
///
/// `_`
pub fn wild_pat(id: NodeId) -> Pat {
    Pat {
        id,
        kind: PatKind::Wild,
    }
}

/// Build a wildcard discriminant pattern
pub fn wild_discriminant() -> TypedBits {
    TypedBits::EMPTY
}

/// Build a literal pattern node
///
/// `lit`
pub fn lit_pat(id: NodeId, lit: ExprLit) -> Pat {
    Pat {
        id,
        kind: PatKind::Lit(PatLit { lit: Box::new(lit) }),
    }
}

/// Build a type pattern node
///
/// `pattern: type`
pub fn type_pat(id: NodeId, pat: Pat, kind: Kind) -> Pat {
    Pat {
        id,
        kind: PatKind::Type(PatType {
            pat: Box::new(pat),
            kind,
        }),
    }
}

/// Build a struct pattern node
///
/// `
/// Path {  
///   field1: pat1,
///   field2: pat2,
///   ..rest
/// }
/// `
pub fn struct_pat(id: NodeId, path: Path, fields: Vec<FieldPat>, rest: bool) -> Pat {
    Pat {
        id,
        kind: PatKind::Struct(PatStruct { path, fields, rest }),
    }
}

/// Build a path pattern node
///
/// `Path`
pub fn path_pat(id: NodeId, path: Path) -> Pat {
    Pat {
        id,
        kind: PatKind::Path(PatPath { path }),
    }
}

/// Build a slice pattern node
///
/// `[elem1, elem2, ...]`
pub fn slice_pat(id: NodeId, elems: Vec<Pat>) -> Pat {
    Pat {
        id,
        kind: PatKind::Slice(PatSlice { elems }),
    }
}

/// Build a tuple pattern node
///
/// `(elem1, elem2, ...)`
pub fn tuple_pat(id: NodeId, elems: Vec<Pat>) -> Pat {
    Pat {
        id,
        kind: PatKind::Tuple(PatTuple { elements: elems }),
    }
}

/// Build a tuple struct pattern node
///
/// `Path(elem1, elem2, ...)`
pub fn tuple_struct_pat(id: NodeId, path: Path, elems: Vec<Pat>) -> Pat {
    Pat {
        id,
        kind: PatKind::TupleStruct(PatTupleStruct { path, elems }),
    }
}

/// Build an identifier pattern node
///
/// `name` or `mut name`
pub fn ident_pat(id: NodeId, name: &'static str, mutable: bool) -> Pat {
    Pat {
        id,
        kind: PatKind::Ident(PatIdent { name, mutable }),
    }
}

/// Build a local statement node
///
/// `let pat = init;`
pub fn local_stmt(local_id: NodeId, pat: Pat, init: Option<Expr>) -> Stmt {
    Stmt {
        id: local_id,
        kind: StmtKind::Local(Local {
            id: local_id,
            pat,
            init: init.map(Box::new),
        }),
    }
}

/// Build an item statement node that includes a semicolon
///
/// `item;`
pub fn semi_stmt(id: NodeId, expr: Expr) -> Stmt {
    Stmt {
        id,
        kind: StmtKind::Semi(expr),
    }
}

/// Build an expression statement node
///
/// `expr`
pub fn expr_stmt(id: NodeId, expr: Expr) -> Stmt {
    Stmt {
        id,
        kind: StmtKind::Expr(expr),
    }
}

/// Build a block expression node
///
/// `{ stmts }`
pub fn block_expr(id: NodeId, block: Block) -> Expr {
    Expr {
        id,
        kind: ExprKind::Block(ExprBlock {
            block: Box::new(block),
        }),
    }
}

/// Build a block node
///
/// `{ stmts }`
pub fn block(id: NodeId, stmts: Vec<Stmt>) -> Block {
    Block { id, stmts }
}

/// Build range limits for half-open range `..` or `start..end`
pub fn range_limits_half_open() -> RangeLimits {
    RangeLimits::HalfOpen
}

/// Build range limits for closed range `..=` or `start..=end`
pub fn range_limits_closed() -> RangeLimits {
    RangeLimits::Closed
}

/// Build a named member
///
/// `foo.name`
pub fn member_named(name: &'static str) -> Member {
    Member::Named(name.to_string().into())
}

/// Build an unnamed member
///
/// `foo.index`
pub fn member_unnamed(index: u32) -> Member {
    Member::Unnamed(index)
}

/// Build an integer literal expression
pub fn expr_lit_int(value: &str) -> ExprLit {
    ExprLit::Int(value.to_string())
}

/// Build a boolean literal expression
pub fn expr_lit_bool(value: bool) -> ExprLit {
    ExprLit::Bool(value)
}

/// Build a literal expression representing typed bits
pub fn expr_lit_typed_bits(value: TypedBits, code: &str) -> ExprLit {
    ExprLit::TypedBits(ExprTypedBits {
        path: path(vec![]),
        value,
        code: code.replace(' ', "").to_string(),
    })
}

/// Build a try expression node
///
/// `expr?`
pub fn expr_try(id: NodeId, expr: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Try(ExprTry {
            expr: Box::new(expr),
        }),
    }
}

/// Build a kernel function node
#[allow(clippy::too_many_arguments)]
pub fn kernel_fn(
    id: NodeId,
    name: &'static str,
    inputs: Vec<Pat>,
    ret: Kind,
    body: Block,
    fn_id: std::any::TypeId,
    text: Option<&'static str>,
    meta_db: MetaDB,
    flags: Vec<KernelFlags>,
) -> KernelFnKind {
    // Hash the typeID into a 64 bit unsigned int
    let mut hasher = fnv::FnvHasher::default();
    fn_id.hash(&mut hasher);
    let fn_id = hasher.finish().into();
    KernelFnKind::AstKernel(Box::new(KernelFn {
        id,
        name,
        inputs,
        ret,
        body,
        fn_id,
        text,
        meta_db,
        flags,
    }))
}

/// Build a cast expression node
///
/// `expr as len`
pub fn expr_cast(id: NodeId, expr: Expr, len: usize) -> Expr {
    Expr {
        id,
        kind: ExprKind::Cast(ExprCast {
            expr: Box::new(expr),
            len,
        }),
    }
}

/// Build a typed bits literal expression node
pub fn expr_typed_bits(id: NodeId, path: Path, value: TypedBits, code: &str) -> Expr {
    Expr {
        id,
        kind: ExprKind::Lit(ExprLit::TypedBits(ExprTypedBits {
            path,
            value,
            code: code.to_string(),
        })),
    }
}

/// Build a bits expression node with specified length
///
/// `bits::<len>(arg)`
pub fn expr_bits_with_length(id: NodeId, arg: Expr, len: usize) -> Expr {
    Expr {
        id,
        kind: ExprKind::Bits(ExprBits {
            kind: BitsKind::Unsigned,
            len: Some(len),
            arg: Box::new(arg),
        }),
    }
}

/// Build a bits expression node
///
/// `bits(arg)`
pub fn expr_bits(id: NodeId, arg: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Bits(ExprBits {
            kind: BitsKind::Unsigned,
            len: None,
            arg: Box::new(arg),
        }),
    }
}

/// Build a signed bits expression node with specified length
///
/// `signed::<len>(arg)`
pub fn expr_signed_with_length(id: NodeId, arg: Expr, len: usize) -> Expr {
    Expr {
        id,
        kind: ExprKind::Bits(ExprBits {
            kind: BitsKind::Signed,
            len: Some(len),
            arg: Box::new(arg),
        }),
    }
}

/// Build a signed bits expression node
///
/// `signed(arg)`
pub fn expr_signed(id: NodeId, arg: Expr) -> Expr {
    Expr {
        id,
        kind: ExprKind::Bits(ExprBits {
            kind: BitsKind::Signed,
            len: None,
            arg: Box::new(arg),
        }),
    }
}

/// Build a `None` expression node
///
/// `None`
pub fn expr_none(id: NodeId) -> Expr {
    let path = path(vec![PathSegment {
        ident: "None",
        arguments: vec![],
    }]);
    Expr {
        id,
        kind: ExprKind::Call(ExprCall {
            path,
            args: vec![],
            signature: None,
            code: Some(KernelFnKind::Wrap(WrapOp::None)),
        }),
    }
}

/// Build a `Some(arg)` expression node
///
/// `Some(arg)`
pub fn expr_some(id: NodeId, arg: Expr) -> Expr {
    let path = path(vec![PathSegment {
        ident: "Some",
        arguments: vec![],
    }]);
    Expr {
        id,
        kind: ExprKind::Call(ExprCall {
            path,
            args: vec![arg],
            signature: None,
            code: Some(KernelFnKind::Wrap(WrapOp::Some)),
        }),
    }
}

/// Build an `Ok(arg)` expression node
///
/// `Ok(arg)`
pub fn expr_ok(id: NodeId, arg: Expr) -> Expr {
    let path = path(vec![PathSegment {
        ident: "Ok",
        arguments: vec![],
    }]);
    Expr {
        id,
        kind: ExprKind::Call(ExprCall {
            path,
            args: vec![arg],
            signature: None,
            code: Some(KernelFnKind::Wrap(WrapOp::Ok)),
        }),
    }
}

/// Build a `Err(arg)` expression node
///
/// `Err(arg)`
pub fn expr_err(id: NodeId, arg: Expr) -> Expr {
    let path = path(vec![PathSegment {
        ident: "Err",
        arguments: vec![],
    }]);
    Expr {
        id,
        kind: ExprKind::Call(ExprCall {
            path,
            args: vec![arg],
            signature: None,
            code: Some(KernelFnKind::Wrap(WrapOp::Err)),
        }),
    }
}

/// Build a Signal expression node
///
/// `signal(arg)`
pub fn expr_signal(id: NodeId, arg: Expr, clock: Option<Color>) -> Expr {
    let path = path(vec![PathSegment {
        ident: "signal",
        arguments: vec![],
    }]);
    Expr {
        id,
        kind: ExprKind::Call(ExprCall {
            path,
            args: vec![arg],
            signature: None,
            code: Some(KernelFnKind::SignalConstructor(clock)),
        }),
    }
}
