use quote::ToTokens;
use quote::quote;
use std::ops::RangeInclusive;

use crate::formatter::Pretty;

#[derive(Clone, Debug)]
pub struct ModuleList(pub Vec<ModuleDef>);

pub fn module_list(modules: Vec<ModuleDef>) -> ModuleList {
    ModuleList(modules)
}

impl Pretty for ModuleList {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        for module in &self.0 {
            module.pretty_print(formatter);
            formatter.newline();
        }
    }
}

impl std::fmt::Display for ModuleList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = crate::formatter::Formatter::new();
        self.pretty_print(&mut fmt);
        write!(f, "{}", fmt.finish())
    }
}

#[derive(Clone, Debug, Copy)]
pub enum HDLKind {
    Wire,
    Reg,
}

impl Pretty for HDLKind {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            HDLKind::Wire => formatter.write("wire"),
            HDLKind::Reg => formatter.write("reg"),
        }
    }
}

impl ToTokens for HDLKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            HDLKind::Wire => quote! {wire},
            HDLKind::Reg => quote! {reg},
        })
    }
}

pub fn wire() -> HDLKind {
    HDLKind::Wire
}

pub fn reg() -> HDLKind {
    HDLKind::Reg
}

#[derive(Clone, Debug, Copy)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

impl Pretty for Direction {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            Direction::Input => formatter.write("input"),
            Direction::Output => formatter.write("output"),
            Direction::Inout => formatter.write("inout"),
        }
    }
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

impl ToTokens for Direction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Direction::Input => quote! {input},
            Direction::Output => quote! {output},
            Direction::Inout => quote! {inout},
        })
    }
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

impl Pretty for SignedWidth {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            SignedWidth::Signed(range) => {
                formatter.write("signed ");
                if range.eq(&(0..=0)) {
                    return;
                }
                formatter.write(&format!("[{}:{}]", range.start(), range.end()));
            }
            SignedWidth::Unsigned(range) => {
                if range.eq(&(0..=0)) {
                    return;
                }
                formatter.write(&format!("[{}:{}]", range.start(), range.end()));
            }
        }
    }
}

impl ToTokens for SignedWidth {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            SignedWidth::Signed(range) => {
                let start = range.start();
                let end = range.end();
                if range.start() == range.end() && range.start() == &0 {
                    quote! {signed}
                } else {
                    quote! {signed [#start:#end]}
                }
            }
            SignedWidth::Unsigned(range) => {
                let start = range.start();
                let end = range.end();
                quote! {[#start:#end]}
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct Declaration {
    pub kind: HDLKind,
    pub signed_width: SignedWidth,
    pub name: String,
}

impl Pretty for Declaration {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.kind.pretty_print(formatter);
        formatter.write(" ");
        self.signed_width.pretty_print(formatter);
        formatter.write(" ");
        formatter.write(&self.name);
    }
}

pub fn declaration(kind: HDLKind, signed_width: SignedWidth, name: &str) -> Declaration {
    Declaration {
        kind,
        signed_width,
        name: name.to_string(),
    }
}

#[derive(Clone, Debug)]
pub struct DeclKind {
    pub name: String,
    pub width: Option<SignedWidth>,
}

impl Pretty for DeclKind {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.name);
        if let Some(width) = &self.width {
            formatter.bracketed(|f| {
                width.pretty_print(f);
            });
        }
    }
}

pub fn decl_kind(name: &str, width: Option<SignedWidth>) -> DeclKind {
    DeclKind {
        name: name.to_string(),
        width,
    }
}

#[derive(Clone, Debug)]
pub struct DeclarationList {
    pub kind: HDLKind,
    pub signed_width: SignedWidth,
    pub items: Vec<DeclKind>,
}

impl Pretty for DeclarationList {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.kind.pretty_print(formatter);
        formatter.write(" ");
        self.signed_width.pretty_print(formatter);
        formatter.write(" ");
        formatter.comma_separated(&self.items);
    }
}

pub fn declaration_list(
    kind: HDLKind,
    signed_width: SignedWidth,
    items: Vec<DeclKind>,
) -> DeclarationList {
    DeclarationList {
        kind,
        signed_width,
        items,
    }
}

#[derive(Clone, Debug)]
pub struct Port {
    pub direction: Direction,
    pub decl: Declaration,
}

impl Pretty for Port {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.direction.pretty_print(formatter);
        formatter.write(" ");
        self.decl.pretty_print(formatter);
    }
}

pub fn port(direction: Direction, decl: Declaration) -> Port {
    Port { direction, decl }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub target: String,
    pub local: Box<Expr>,
}

impl Pretty for Connection {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&format!(".{}", self.target));
        formatter.parenthesized(|f| {
            self.local.pretty_print(f);
        });
    }
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

impl Pretty for Sensitivity {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            Sensitivity::PosEdge(signal) => formatter.write(&format!("posedge {}", signal)),
            Sensitivity::NegEdge(signal) => formatter.write(&format!("negedge {}", signal)),
            Sensitivity::Signal(signal) => formatter.write(signal),
            Sensitivity::Star => formatter.write("*"),
        }
    }
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

impl Pretty for BinaryOp {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            BinaryOp::Shl => formatter.write("<<"),
            BinaryOp::SignedRightShift => formatter.write(">>>"),
            BinaryOp::Shr => formatter.write(">>"),
            BinaryOp::ShortAnd => formatter.write("&&"),
            BinaryOp::ShortOr => formatter.write("||"),
            BinaryOp::CaseEq => formatter.write("==="),
            BinaryOp::CaseNe => formatter.write("!=="),
            BinaryOp::Ne => formatter.write("!="),
            BinaryOp::Eq => formatter.write("=="),
            BinaryOp::Ge => formatter.write(">="),
            BinaryOp::Le => formatter.write("<="),
            BinaryOp::Gt => formatter.write(">"),
            BinaryOp::Lt => formatter.write("<"),
            BinaryOp::Plus => formatter.write("+"),
            BinaryOp::Minus => formatter.write("-"),
            BinaryOp::And => formatter.write("&"),
            BinaryOp::Or => formatter.write("|"),
            BinaryOp::Xor => formatter.write("^"),
            BinaryOp::Mod => formatter.write("%"),
            BinaryOp::Mul => formatter.write("*"),
        }
    }
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

impl Pretty for UnaryOp {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            UnaryOp::Plus => formatter.write("+"),
            UnaryOp::Minus => formatter.write("-"),
            UnaryOp::Bang => formatter.write("!"),
            UnaryOp::Not => formatter.write("~"),
            UnaryOp::And => formatter.write("&"),
            UnaryOp::Or => formatter.write("|"),
            UnaryOp::Xor => formatter.write("^"),
        }
    }
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

#[derive(Debug, Clone)]
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
    DynamicSplice(DynamicSplice),
    Delay(u32),
    ConcatAssign(ConcatAssign),
}

impl Pretty for Stmt {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            Stmt::If(if_stmt) => if_stmt.pretty_print(formatter),
            Stmt::Always(always_stmt) => always_stmt.pretty_print(formatter),
            Stmt::Case(case_stmt) => case_stmt.pretty_print(formatter),
            Stmt::LocalParam(local_param_stmt) => local_param_stmt.pretty_print(formatter),
            Stmt::Block(block_stmts) => {
                formatter.write("begin\n");
                formatter.scoped(|f| {
                    f.lines(block_stmts);
                });
                formatter.write("end\n");
            }
            Stmt::ContinuousAssign(continuous_assign_stmt) => {
                formatter.write("assign ");
                continuous_assign_stmt.pretty_print(formatter)
            }
            Stmt::FunctionCall(function_call_stmt) => function_call_stmt.pretty_print(formatter),
            Stmt::NonblockAssign(nonblock_assign_stmt) => {
                nonblock_assign_stmt.pretty_print(formatter)
            }
            Stmt::Assign(assign_stmt) => assign_stmt.pretty_print(formatter),
            Stmt::Instance(instance_stmt) => instance_stmt.pretty_print(formatter),
            Stmt::DynamicSplice(dynamic_splice_stmt) => dynamic_splice_stmt.pretty_print(formatter),
            Stmt::Delay(delay) => formatter.write(&format!("#{}", delay)),
            Stmt::ConcatAssign(concat_assign_stmt) => concat_assign_stmt.pretty_print(formatter),
        }
    }
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

pub fn local_param_stmt(target: &str, rhs: ConstExpr) -> Stmt {
    Stmt::LocalParam(LocalParam {
        target: target.to_string(),
        rhs,
    })
}

pub fn block_stmt(stmts: Vec<Stmt>) -> Stmt {
    Stmt::Block(stmts)
}

pub fn continuous_assign_stmt(target: AssignTarget, rhs: Expr) -> Stmt {
    Stmt::ContinuousAssign(Assign {
        target,
        rhs: Box::new(rhs),
    })
}

pub fn function_call_stmt(name: &str, args: Vec<Expr>) -> Stmt {
    Stmt::FunctionCall(FunctionCall {
        name: name.to_string(),
        args,
    })
}

pub fn nonblock_assign_stmt(target: AssignTarget, rhs: Expr) -> Stmt {
    Stmt::NonblockAssign(Assign {
        target,
        rhs: Box::new(rhs),
    })
}

pub fn assign_stmt(target: AssignTarget, rhs: Expr) -> Stmt {
    Stmt::Assign(Assign {
        target,
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

pub fn dynamic_splice_stmt(target: &str, base: Expr, op: DynOp, width: Expr, rhs: Expr) -> Stmt {
    Stmt::DynamicSplice(DynamicSplice {
        target: target.to_string(),
        base: Box::new(base),
        op,
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

impl Pretty for ConcatAssign {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.braced(|formatter| {
            formatter.comma_separated(&self.target);
        });
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
}

#[derive(Debug, Clone)]
pub enum DynOp {
    PlusColon,
    MinusColon,
}

impl Pretty for DynOp {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            DynOp::PlusColon => formatter.write("+:"),
            DynOp::MinusColon => formatter.write("-:"),
        }
    }
}

pub fn dyn_plus_colon() -> DynOp {
    DynOp::PlusColon
}

pub fn dyn_minus_colon() -> DynOp {
    DynOp::MinusColon
}

#[derive(Debug, Clone)]
pub struct DynamicSplice {
    pub target: String,
    pub base: Box<Expr>,
    pub op: DynOp,
    pub width: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Pretty for DynamicSplice {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.target);
        formatter.bracketed(|f| {
            self.base.pretty_print(f);
            self.op.pretty_print(f);
            self.width.pretty_print(f);
        });
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
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

impl Pretty for Instance {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&format!("{} {}", self.module, self.instance));
        formatter.parenthesized(|f| {
            f.comma_separated(&self.connections);
        });
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expr>,
}

impl Pretty for FunctionCall {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.name);
        formatter.parenthesized(|f| {
            f.comma_separated(&self.args);
        });
    }
}

#[derive(Debug, Clone)]
pub enum AssignTarget {
    Ident(String),
    Index(Expr),
}

impl Pretty for AssignTarget {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            AssignTarget::Ident(ident) => formatter.write(ident),
            AssignTarget::Index(target) => target.pretty_print(formatter),
        }
    }
}

pub fn assign_target_ident(ident: &str) -> AssignTarget {
    AssignTarget::Ident(ident.to_string())
}

pub fn assign_target_index(target: Expr) -> AssignTarget {
    AssignTarget::Index(target)
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub target: AssignTarget,
    pub rhs: Box<Expr>,
}

impl Pretty for Assign {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.target.pretty_print(formatter);
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
}

#[derive(Debug, Clone)]
pub struct LocalParam {
    pub target: String,
    pub rhs: ConstExpr,
}

impl Pretty for LocalParam {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write("localparam ");
        formatter.write(&self.target);
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
}

#[derive(Debug, Clone)]
pub enum ConstExpr {
    LitVerilog(LitVerilog),
    LitInt(i32),
    LitStr(String),
}

impl Pretty for ConstExpr {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            ConstExpr::LitVerilog(lit) => lit.pretty_print(formatter),
            ConstExpr::LitInt(int) => formatter.write(&int.to_string()),
            ConstExpr::LitStr(string) => formatter.write(&format!("\"{}\"", string)),
        }
    }
}

pub fn const_verilog(val: LitVerilog) -> ConstExpr {
    ConstExpr::LitVerilog(val)
}

pub fn const_int(val: i32) -> ConstExpr {
    ConstExpr::LitInt(val)
}

pub fn const_str(val: &str) -> ConstExpr {
    ConstExpr::LitStr(val.to_string())
}

#[derive(Debug, Clone)]
pub struct LitVerilog {
    pub width: u32,
    pub value: String,
}

impl Pretty for LitVerilog {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&format!("{}'", self.width));
        formatter.write(&self.value);
    }
}
pub fn lit_verilog(width: u32, value: &str) -> LitVerilog {
    LitVerilog {
        width,
        value: value.to_string(),
    }
}

#[derive(Debug, Clone)]
pub enum CaseItem {
    Ident(String),
    Literal(LitVerilog),
    Wild,
}

impl Pretty for CaseItem {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            CaseItem::Ident(ident) => formatter.write(ident),
            CaseItem::Literal(lit) => lit.pretty_print(formatter),
            CaseItem::Wild => formatter.write("default"),
        }
        formatter.write(": ");
    }
}

pub fn case_item_ident(ident: &str) -> CaseItem {
    CaseItem::Ident(ident.to_string())
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

impl Pretty for CaseLine {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.item.pretty_print(formatter);
        self.stmt.pretty_print(formatter);
    }
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

impl Pretty for Case {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write("case ");
        formatter.parenthesized(|f| self.discriminant.pretty_print(f));
        formatter.newline();
        formatter.scoped(|f| {
            f.lines(&self.lines);
        });
        formatter.write("endcase");
    }
}

#[derive(Debug, Clone)]
pub struct Always {
    pub sensitivity: Vec<Sensitivity>,
    pub body: Box<Stmt>,
}

impl Pretty for Always {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write("always @");
        formatter.parenthesized(|f| {
            f.comma_separated(&self.sensitivity);
        });
        formatter.write(" ");
        self.body.pretty_print(formatter);
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Expr>,
    pub true_stmt: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

impl Pretty for If {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write("if ");
        formatter.parenthesized(|f| self.condition.pretty_print(f));
        formatter.write(" ");
        self.true_stmt.pretty_print(formatter);
        if let Some(else_branch) = &self.else_branch {
            formatter.write("else ");
            else_branch.pretty_print(formatter);
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub width: SignedWidth,
    pub name: String,
    pub args: Vec<Port>,
    pub items: Vec<Item>,
}

impl Pretty for FunctionDef {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&format!("function "));
        formatter.scoped(|formatter| {
            self.width.pretty_print(formatter);
            formatter.write(&format!(" {}", self.name));
            formatter.parenthesized(|f| f.comma_separated(&self.args));
            formatter.newline();
            formatter.scoped(|f| {
                f.lines(&self.items);
            });
        });
        formatter.write("endfunction");
    }
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
    Declaration(DeclarationList),
    FunctionDef(FunctionDef),
    Initial(Stmt),
}

impl Pretty for Item {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            Item::Statement(stmt) => stmt.pretty_print(formatter),
            Item::Declaration(decl) => decl.pretty_print(formatter),
            Item::FunctionDef(func) => func.pretty_print(formatter),
            Item::Initial(initial) => {
                formatter.write("initial ");
                initial.pretty_print(formatter);
            }
        }
    }
}

pub fn stmt_item(stmt: Stmt) -> Item {
    Item::Statement(stmt)
}

pub fn declaration_item(decl: DeclarationList) -> Item {
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

impl Pretty for ModuleDef {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&format!("module {}", self.name));
        formatter.parenthesized(|formatter| {
            formatter.comma_separated(&self.args);
        });
        formatter.write(";");
        formatter.newline();
        formatter.scoped(|formatter| {
            formatter.lines(&self.items);
        });
        formatter.write("endmodule");
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

impl Pretty for Expr {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            Expr::Binary(expr) => expr.pretty_print(formatter),
            Expr::Unary(expr) => expr.pretty_print(formatter),
            Expr::Constant(lit) => lit.pretty_print(formatter),
            Expr::Literal(value) => formatter.write(&value.to_string()),
            Expr::String(value) => formatter.write(&format!("\"{}\"", value)),
            Expr::Ident(name) => formatter.write(name),
            Expr::Paren(expr) => {
                formatter.parenthesized(|f| expr.pretty_print(f));
            }
            Expr::Ternary(expr) => expr.pretty_print(formatter),
            Expr::Concat(exprs) => {
                formatter.braced(|f| f.comma_separated(exprs));
            }
            Expr::Replica(expr) => expr.pretty_print(formatter),
            Expr::Index(expr) => expr.pretty_print(formatter),
            Expr::DynIndex(expr) => expr.pretty_print(formatter),
            Expr::Function(expr) => expr.pretty_print(formatter),
        }
    }
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

pub fn dyn_index_expr(target: &str, base: Expr, op: DynOp, width: Expr) -> Expr {
    Expr::DynIndex(ExprDynIndex {
        target: target.to_string(),
        base: Box::new(base),
        op,
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
    pub op: DynOp,
    pub width: Box<Expr>,
}

impl Pretty for ExprDynIndex {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.target);
        formatter.bracketed(|f| {
            self.base.pretty_print(f);
            self.op.pretty_print(f);
            self.width.pretty_print(f);
        });
    }
}

#[derive(Debug, Clone)]
pub struct ExprIndex {
    pub target: String,
    pub msb: Box<Expr>,
    pub lsb: Option<Box<Expr>>,
}

impl Pretty for ExprIndex {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.target);
        formatter.bracketed(|f| {
            self.msb.pretty_print(f);
            if let Some(lsb) = &self.lsb {
                f.write(":");
                lsb.pretty_print(f);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct ExprTernary {
    pub lhs: Box<Expr>,
    pub mhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Pretty for ExprTernary {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.lhs.pretty_print(formatter);
        formatter.write(" ? ");
        self.mhs.pretty_print(formatter);
        formatter.write(" : ");
        self.rhs.pretty_print(formatter);
    }
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
    pub lhs: Box<Expr>,
    pub op: BinaryOp,
    pub rhs: Box<Expr>,
}

impl Pretty for ExprBinary {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.lhs.pretty_print(formatter);
        self.op.pretty_print(formatter);
        self.rhs.pretty_print(formatter);
    }
}

#[derive(Debug, Clone)]
pub struct ExprUnary {
    pub op: UnaryOp,
    pub arg: Box<Expr>,
}

impl Pretty for ExprUnary {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.op.pretty_print(formatter);
        self.arg.pretty_print(formatter);
    }
}

#[derive(Debug, Clone)]
pub struct ExprReplica {
    pub count: usize,
    pub concatenation: Vec<Expr>,
}

impl Pretty for ExprReplica {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.braced(|f| {
            f.write(&format!("{}", self.count));
            f.braced(|f| {
                f.comma_separated(&self.concatenation);
            });
        })
    }
}

#[derive(Debug, Clone)]
pub struct ExprFunction {
    pub name: String,
    pub args: Vec<Expr>,
}

impl Pretty for ExprFunction {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.name);
        formatter.parenthesized(|f| f.comma_separated(&self.args));
    }
}
