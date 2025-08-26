use std::cell::Cell;
use std::hash::{Hash, Hasher};

pub use crate::rhdl_core::ast::ast_impl::BinOp;
pub use crate::rhdl_core::ast::ast_impl::UnOp;
use crate::rhdl_core::kernel::KernelFnKind;
use crate::rhdl_core::rhif::spec::Member;
use crate::rhdl_core::types::typed_bits::TypedBits;
use crate::rhdl_core::{Color, Digital, DigitalSignature};
use crate::rhdl_core::{Kind, ast::ast_impl::*};

#[derive(Default)]
pub struct ASTBuilder {
    node_id: Cell<u32>,
}

impl ASTBuilder {
    pub fn id(&self) -> NodeId {
        let id = self.node_id.take();
        self.node_id.set(id + 1);
        NodeId::new(id)
    }

    pub fn binary_expr(&self, op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Binary(ExprBinary { op, lhs, rhs }),
        })
    }

    pub fn unary_expr(&self, op: UnOp, expr: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Unary(ExprUnary { op, expr }),
        })
    }

    pub fn assign_expr(&self, lhs: Box<Expr>, rhs: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Assign(ExprAssign { lhs, rhs }),
        })
    }

    pub fn lit_expr(&self, lit: ExprLit) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Lit(lit),
        })
    }

    pub fn struct_expr(
        &self,
        path: Box<Path>,
        fields: Vec<Box<FieldValue>>,
        rest: Option<Box<Expr>>,
        template: TypedBits,
    ) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Struct(ExprStruct {
                path,
                fields,
                rest,
                template,
            }),
        })
    }

    pub fn if_expr(
        &self,
        cond: Box<Expr>,
        then_branch: Box<Block>,
        else_branch: Option<Box<Expr>>,
    ) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::If(ExprIf {
                cond,
                then_branch,
                else_branch,
            }),
        })
    }

    pub fn if_let_expr(
        &self,
        test: Box<Expr>,
        kind: ArmKind,
        then_block: Box<Block>,
        else_branch: Option<Box<Expr>>,
    ) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::IfLet(ExprIfLet {
                test,
                kind,
                then_block,
                else_branch,
            }),
        })
    }

    pub fn let_expr(&self, pattern: Box<Pat>, value: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Let(ExprLet { pattern, value }),
        })
    }

    pub fn path_arguments_none(&self) -> Vec<&'static str> {
        vec![]
    }

    pub fn path_segment(&self, ident: &'static str, arguments: Vec<&'static str>) -> PathSegment {
        PathSegment { ident, arguments }
    }

    pub fn path(&self, segments: Vec<PathSegment>) -> Box<Path> {
        Box::new(Path { segments })
    }

    pub fn path_expr(&self, path: Box<Path>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Path(ExprPath { path }),
        })
    }

    pub fn arm_kind_wild(&self) -> ArmKind {
        ArmKind::Wild
    }

    pub fn arm_kind_constant(&self, value: ExprLit) -> ArmKind {
        ArmKind::Constant(ArmConstant { value })
    }

    pub fn arm_kind_enum(&self, pat: Box<Pat>, discriminant: TypedBits) -> ArmKind {
        ArmKind::Enum(ArmEnum { pat, discriminant })
    }

    pub fn arm_kind_none(&self) -> ArmKind {
        ArmKind::Enum(ArmEnum {
            pat: self.wild_pat(),
            discriminant: false.typed_bits(),
        })
    }

    pub fn arm(&self, kind: ArmKind, body: Box<Expr>) -> Box<Arm> {
        let id = self.id();
        Box::new(Arm { id, kind, body })
    }

    pub fn field_expr(&self, expr: Box<Expr>, member: Member) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Field(ExprField { expr, member }),
        })
    }

    pub fn field_value(&self, member: Member, value: Box<Expr>) -> Box<FieldValue> {
        Box::new(FieldValue { member, value })
    }

    pub fn match_expr(&self, expr: Box<Expr>, arms: Vec<Box<Arm>>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Match(ExprMatch { expr, arms }),
        })
    }

    pub fn range_expr(
        &self,
        start: Option<Box<Expr>>,
        limits: RangeLimits,
        end: Option<Box<Expr>>,
    ) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Range(ExprRange { start, limits, end }),
        })
    }

    pub fn paren_expr(&self, expr: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Paren(ExprParen { expr }),
        })
    }

    pub fn group_expr(&self, expr: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Group(ExprGroup { expr }),
        })
    }

    pub fn tuple_expr(&self, elements: Vec<Box<Expr>>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Tuple(ExprTuple { elements }),
        })
    }

    pub fn repeat_expr(&self, value: Box<Expr>, len: i64) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Repeat(ExprRepeat { value, len }),
        })
    }

    pub fn for_expr(&self, pat: Box<Pat>, expr: Box<Expr>, body: Box<Block>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::ForLoop(ExprForLoop { pat, expr, body }),
        })
    }

    pub fn call_expr(
        &self,
        path: Box<Path>,
        args: Vec<Box<Expr>>,
        signature: DigitalSignature,
        code: Option<KernelFnKind>,
    ) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Call(ExprCall {
                path,
                args,
                signature: Some(signature),
                code,
            }),
        })
    }

    pub fn array_expr(&self, elems: Vec<Box<Expr>>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Array(ExprArray { elems }),
        })
    }

    pub fn index_expr(&self, expr: Box<Expr>, index: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Index(ExprIndex { expr, index }),
        })
    }

    pub fn method_expr(
        &self,
        receiver: Box<Expr>,
        args: Vec<Box<Expr>>,
        method: &'static str,
        turbo: Option<usize>,
    ) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::MethodCall(ExprMethodCall {
                receiver,
                args,
                method,
                turbo,
            }),
        })
    }
    pub fn return_expr(&self, expr: Option<Box<Expr>>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Ret(ExprRet { expr }),
        })
    }

    pub fn field_pat(&self, member: Member, pat: Box<Pat>) -> Box<FieldPat> {
        Box::new(FieldPat { member, pat })
    }

    pub fn wild_pat(&self) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Wild,
        })
    }

    pub fn wild_discriminant(&self) -> TypedBits {
        TypedBits {
            bits: vec![],
            kind: Kind::Empty,
        }
    }

    pub fn lit_pat(&self, lit: ExprLit) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Lit(PatLit { lit: Box::new(lit) }),
        })
    }

    pub fn type_pat(&self, pat: Box<Pat>, kind: Kind) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Type(PatType { pat, kind }),
        })
    }

    pub fn struct_pat(&self, path: Box<Path>, fields: Vec<Box<FieldPat>>, rest: bool) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Struct(PatStruct { path, fields, rest }),
        })
    }

    pub fn path_pat(&self, path: Box<Path>) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Path(PatPath { path }),
        })
    }

    pub fn slice_pat(&self, elems: Vec<Box<Pat>>) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Slice(PatSlice { elems }),
        })
    }

    pub fn tuple_pat(&self, elems: Vec<Box<Pat>>) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Tuple(PatTuple { elements: elems }),
        })
    }

    pub fn tuple_struct_pat(&self, path: Box<Path>, elems: Vec<Box<Pat>>) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::TupleStruct(PatTupleStruct { path, elems }),
        })
    }

    pub fn ident_pat(&self, name: &'static str, mutable: bool) -> Box<Pat> {
        let id = self.id();
        Box::new(Pat {
            id,
            kind: PatKind::Ident(PatIdent { name, mutable }),
        })
    }

    pub fn local_stmt(&self, pat: Box<Pat>, init: Option<Box<Expr>>) -> Box<Stmt> {
        let local_id = self.id();
        let stmt_id = self.id();
        Box::new(Stmt {
            id: stmt_id,
            kind: StmtKind::Local(Box::new(Local {
                id: local_id,
                pat,
                init,
            })),
        })
    }

    pub fn semi_stmt(&self, expr: Box<Expr>) -> Box<Stmt> {
        let id = self.id();
        Box::new(Stmt {
            id,
            kind: StmtKind::Semi(expr),
        })
    }

    pub fn expr_stmt(&self, expr: Box<Expr>) -> Box<Stmt> {
        let id = self.id();
        Box::new(Stmt {
            id,
            kind: StmtKind::Expr(expr),
        })
    }

    pub fn block_expr(&self, block: Box<Block>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Block(ExprBlock { block }),
        })
    }

    pub fn block(&self, stmts: Vec<Box<Stmt>>) -> Box<Block> {
        let id = self.id();
        Box::new(Block { id, stmts })
    }

    pub fn range_limits_half_open(&self) -> RangeLimits {
        RangeLimits::HalfOpen
    }

    pub fn range_limits_closed(&self) -> RangeLimits {
        RangeLimits::Closed
    }

    pub fn member_named(&self, name: &'static str) -> Member {
        Member::Named(name.to_string().into())
    }

    pub fn member_unnamed(&self, index: u32) -> Member {
        Member::Unnamed(index)
    }

    pub fn expr_lit_int(&self, value: &str) -> ExprLit {
        ExprLit::Int(value.to_string())
    }

    pub fn expr_lit_bool(&self, value: bool) -> ExprLit {
        ExprLit::Bool(value)
    }

    pub fn expr_lit_typed_bits(&self, value: TypedBits, code: &str) -> ExprLit {
        ExprLit::TypedBits(ExprTypedBits {
            path: self.path(vec![]),
            value,
            code: code.replace(' ', "").to_string(),
        })
    }

    pub fn expr_try(&self, expr: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Try(ExprTry { expr }),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn kernel_fn(
        &self,
        name: &'static str,
        inputs: Vec<Box<Pat>>,
        ret: Kind,
        body: Box<Block>,
        fn_id: std::any::TypeId,
        text: &'static str,
        file: &'static str,
        flags: Vec<KernelFlags>,
    ) -> KernelFnKind {
        let id = self.id();
        // Hash the typeID into a 64 bit unsigned int
        let mut hasher = fnv::FnvHasher::default();
        fn_id.hash(&mut hasher);
        let fn_id = hasher.finish().into();
        KernelFnKind::Kernel(
            Box::new(KernelFn {
                id,
                name,
                inputs,
                ret,
                body,
                fn_id,
                text,
                file,
                flags,
            })
            .into(),
        )
    }

    pub fn expr_cast(&self, expr: Box<Expr>, len: usize) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Cast(ExprCast { expr, len }),
        })
    }

    pub fn expr_typed_bits(&self, path: Box<Path>, value: TypedBits, code: &str) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Lit(ExprLit::TypedBits(ExprTypedBits {
                path,
                value,
                code: code.to_string(),
            })),
        })
    }

    pub fn expr_bits_with_length(&self, arg: Box<Expr>, len: usize) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Bits(ExprBits {
                kind: BitsKind::Unsigned,
                len: Some(len),
                arg,
            }),
        })
    }

    pub fn expr_bits(&self, arg: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Bits(ExprBits {
                kind: BitsKind::Unsigned,
                len: None,
                arg,
            }),
        })
    }

    pub fn expr_signed_with_length(&self, arg: Box<Expr>, len: usize) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Bits(ExprBits {
                kind: BitsKind::Signed,
                len: Some(len),
                arg,
            }),
        })
    }

    pub fn expr_signed(&self, arg: Box<Expr>) -> Box<Expr> {
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Bits(ExprBits {
                kind: BitsKind::Signed,
                len: None,
                arg,
            }),
        })
    }

    pub fn expr_none(&self) -> Box<Expr> {
        let path = self.path(vec![PathSegment {
            ident: "None",
            arguments: vec![],
        }]);
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Call(ExprCall {
                path,
                args: vec![],
                signature: None,
                code: Some(KernelFnKind::Wrap(WrapOp::None)),
            }),
        })
    }

    pub fn expr_some(&self, arg: Box<Expr>) -> Box<Expr> {
        let path = self.path(vec![PathSegment {
            ident: "Some",
            arguments: vec![],
        }]);
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Call(ExprCall {
                path,
                args: vec![arg],
                signature: None,
                code: Some(KernelFnKind::Wrap(WrapOp::Some)),
            }),
        })
    }

    pub fn expr_ok(&self, arg: Box<Expr>) -> Box<Expr> {
        let path = self.path(vec![PathSegment {
            ident: "Ok",
            arguments: vec![],
        }]);
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Call(ExprCall {
                path,
                args: vec![arg],
                signature: None,
                code: Some(KernelFnKind::Wrap(WrapOp::Ok)),
            }),
        })
    }

    pub fn expr_err(&self, arg: Box<Expr>) -> Box<Expr> {
        let path = self.path(vec![PathSegment {
            ident: "Err",
            arguments: vec![],
        }]);
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Call(ExprCall {
                path,
                args: vec![arg],
                signature: None,
                code: Some(KernelFnKind::Wrap(WrapOp::Err)),
            }),
        })
    }

    pub fn expr_signal(&self, arg: Box<Expr>, clock: Option<Color>) -> Box<Expr> {
        let path = self.path(vec![PathSegment {
            ident: "signal",
            arguments: vec![],
        }]);
        let id = self.id();
        Box::new(Expr {
            id,
            kind: ExprKind::Call(ExprCall {
                path,
                args: vec![arg],
                signature: None,
                code: Some(KernelFnKind::SignalConstructor(clock)),
            }),
        })
    }
}
