use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::{
    Ident, LitInt, LitStr, Result, Token, parenthesized,
    parse::{Parse, ParseStream},
    token,
};

use crate::{
    ParenCommaList, Parenthesized,
    atoms::{LitVerilog, SensitivityList},
    expr::{Expr, ExprConcat, ExprDynIndex, ExprIndex},
    formatter::{Formatter, Pretty},
    kw_ops::{LeftArrow, kw},
};

#[derive(Default, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum StmtKind {
    If(If),
    Always(Always),
    Case(Case),
    LocalParam(LocalParam),
    Block(Block),
    ContinuousAssign(ContinuousAssign),
    FunctionCall(FunctionCall),
    NonblockAssign(NonblockAssign),
    Assign(Assign),
    Instance(Instance),
    DynamicSplice(DynamicSplice),
    Delay(Delay),
    ConcatAssign(ConcatAssign),
    #[default]
    /// Required because the parser for if/else uses it as a placeholder
    Noop,
}

impl Parse for StmtKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::If) {
            input.parse().map(StmtKind::If)
        } else if lookahead.peek(kw::always) {
            input.parse().map(StmtKind::Always)
        } else if lookahead.peek(kw::case) {
            input.parse().map(StmtKind::Case)
        } else if lookahead.peek(kw::localparam) {
            input.parse().map(StmtKind::LocalParam)
        } else if lookahead.peek(kw::begin) {
            input.parse().map(StmtKind::Block)
        } else if lookahead.peek(kw::assign) {
            input.parse().map(StmtKind::ContinuousAssign)
        } else if lookahead.peek(Token![$]) {
            input.parse().map(StmtKind::FunctionCall)
        } else if input.fork().parse::<NonblockAssign>().is_ok() {
            input.parse().map(StmtKind::NonblockAssign)
        } else if input.fork().parse::<Assign>().is_ok() {
            input.parse().map(StmtKind::Assign)
        } else if lookahead.peek(Ident) && input.peek2(Ident) && input.peek3(token::Paren) {
            input.parse().map(StmtKind::Instance)
        } else if lookahead.peek(Ident) && input.peek2(token::Bracket) {
            input.parse().map(StmtKind::DynamicSplice)
        } else if lookahead.peek(token::Brace) {
            input.parse().map(StmtKind::ConcatAssign)
        } else if lookahead.peek(token::Pound) {
            input.parse().map(StmtKind::Delay)
        } else if lookahead.peek(Token![;]) {
            Ok(StmtKind::Noop)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Pretty for StmtKind {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            StmtKind::If(if_stmt) => if_stmt.pretty_print(formatter),
            StmtKind::Always(always_stmt) => always_stmt.pretty_print(formatter),
            StmtKind::Case(case_stmt) => case_stmt.pretty_print(formatter),
            StmtKind::LocalParam(local_param) => local_param.pretty_print(formatter),
            StmtKind::Block(block) => block.pretty_print(formatter),
            StmtKind::ContinuousAssign(continuous_assign) => {
                continuous_assign.pretty_print(formatter)
            }
            StmtKind::FunctionCall(function_call) => function_call.pretty_print(formatter),
            StmtKind::NonblockAssign(nonblock_assign) => nonblock_assign.pretty_print(formatter),
            StmtKind::Assign(assign) => assign.pretty_print(formatter),
            StmtKind::Instance(instance) => instance.pretty_print(formatter),
            StmtKind::DynamicSplice(dynamic_splice) => dynamic_splice.pretty_print(formatter),
            StmtKind::Delay(delay) => delay.pretty_print(formatter),
            StmtKind::ConcatAssign(concat_assign) => concat_assign.pretty_print(formatter),
            StmtKind::Noop => {}
        }
    }
}

impl ToTokens for StmtKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            StmtKind::If(if_stmt) => if_stmt.to_tokens(tokens),
            StmtKind::Always(always_stmt) => always_stmt.to_tokens(tokens),
            StmtKind::Case(case_stmt) => case_stmt.to_tokens(tokens),
            StmtKind::LocalParam(local_param) => local_param.to_tokens(tokens),
            StmtKind::Block(block) => block.to_tokens(tokens),
            StmtKind::ContinuousAssign(continuous_assign) => continuous_assign.to_tokens(tokens),
            StmtKind::FunctionCall(function_call) => function_call.to_tokens(tokens),
            StmtKind::NonblockAssign(nonblock_assign) => nonblock_assign.to_tokens(tokens),
            StmtKind::Assign(assign) => assign.to_tokens(tokens),
            StmtKind::Instance(instance) => instance.to_tokens(tokens),
            StmtKind::DynamicSplice(dynamic_splice) => dynamic_splice.to_tokens(tokens),
            StmtKind::Delay(delay) => delay.to_tokens(tokens),
            StmtKind::ConcatAssign(concat_assign) => concat_assign.to_tokens(tokens),
            StmtKind::Noop => {}
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize, Default)]
pub struct Stmt {
    pub kind: StmtKind,
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse()?;
        let _ = input.parse::<Option<Token![;]>>()?;
        Ok(Stmt { kind })
    }
}

impl Pretty for Stmt {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.kind.pretty_print(formatter);
    }
}

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.kind.to_tokens(tokens);
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Delay {
    pub length: u64,
}

impl Parse for Delay {
    fn parse(input: ParseStream) -> Result<Self> {
        let _hash = input.parse::<Token![#]>()?;
        let length: LitInt = input.parse()?;
        let length = length.base10_parse::<u64>()?;
        Ok(Delay { length })
    }
}

impl Pretty for Delay {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("#");
        formatter.write(&self.length.to_string());
    }
}

impl ToTokens for Delay {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let length = syn::Index::from(self.length as usize);
        let hash = token::Pound::default();
        tokens.extend(quote! { #hash #length });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expr>,
}

impl Parse for FunctionCall {
    fn parse(input: ParseStream) -> Result<Self> {
        let _dollar = input.parse::<Token![$]>()?;
        let name = input.parse::<Ident>()?;
        let args = if input.peek(token::Paren) {
            Some(input.parse::<ParenCommaList<Expr>>()?)
        } else {
            None
        };
        Ok(FunctionCall {
            name: name.to_string(),
            args: args.into_iter().flat_map(|x| x.inner.into_iter()).collect(),
        })
    }
}

impl Pretty for FunctionCall {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("$");
        formatter.write(&self.name);
        if !self.args.is_empty() {
            formatter.parenthesized(|f| {
                f.comma_separated(self.args.iter());
            });
        }
    }
}

impl ToTokens for FunctionCall {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.name);
        let args = &self.args;
        if args.is_empty() {
            tokens.extend(quote! { $ #name ; });
        } else {
            tokens.extend(quote! { $ #name ( #( #args ),* ) ; });
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum CaseItem {
    Ident(String),
    Literal(LitVerilog),
    Wild,
}

impl Parse for CaseItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::default) {
            input.parse::<kw::default>().map(|_| CaseItem::Wild)
        } else if input.peek(Ident) {
            input
                .parse::<Ident>()
                .map(|ident| CaseItem::Ident(ident.to_string()))
        } else {
            input.parse().map(CaseItem::Literal)
        }
    }
}

impl Pretty for CaseItem {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            CaseItem::Ident(ident) => formatter.write(ident),
            CaseItem::Literal(literal) => literal.pretty_print(formatter),
            CaseItem::Wild => formatter.write("default"),
        }
    }
}

impl ToTokens for CaseItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            CaseItem::Ident(ident) => {
                let ident = format_ident!("{}", ident);
                tokens.extend(quote! { #ident });
            }
            CaseItem::Literal(literal) => literal.to_tokens(tokens),
            CaseItem::Wild => tokens.extend(quote! { default }),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct CaseLine {
    pub item: CaseItem,
    pub stmt: Box<Stmt>,
}

impl Parse for CaseLine {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse()?;
        let _colon = input.parse::<Token![:]>()?;
        let stmt = input.parse()?;
        Ok(CaseLine {
            item,
            stmt: Box::new(stmt),
        })
    }
}

impl Pretty for CaseLine {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.item.pretty_print(formatter);
        formatter.write(" : ");
        self.stmt.pretty_print(formatter);
    }
}

impl ToTokens for CaseLine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item;
        let stmt = &self.stmt;
        tokens.extend(quote! { #item : #stmt });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Case {
    pub discriminant: Box<Expr>,
    pub lines: Vec<CaseLine>,
}

impl Parse for Case {
    fn parse(input: ParseStream) -> Result<Self> {
        let _case = input.parse::<kw::case>()?;
        let discriminant;
        let _parens = parenthesized!(discriminant in input);
        let discriminant = discriminant.parse::<Box<Expr>>()?;
        let mut lines = Vec::<CaseLine>::new();
        loop {
            if input.peek(kw::endcase) {
                break;
            }
            lines.push(input.parse()?);
        }
        let _endcase = input.parse::<kw::endcase>()?;
        Ok(Self {
            discriminant,
            lines,
        })
    }
}

impl Pretty for Case {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("case ");
        formatter.parenthesized(|f| self.discriminant.pretty_print(f));
        formatter.newline();
        formatter.scoped(|f| {
            f.lines(&self.lines);
        });
        formatter.write("endcase");
    }
}

impl ToTokens for Case {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let discriminant = &self.discriminant;
        let lines = &self.lines;
        tokens.extend(quote! {
            case (#discriminant)
                #( #lines )*
            endcase
        });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub target: String,
    pub local: Box<Expr>,
}

impl Parse for Connection {
    fn parse(input: ParseStream) -> Result<Self> {
        let _dot = input.parse::<Token![.]>()?;
        let target = input.parse::<Ident>()?;
        let content;
        let _paren = parenthesized!(content in input);
        let local = content.parse::<Box<Expr>>()?;
        Ok(Self {
            target: target.to_string(),
            local,
        })
    }
}

impl Pretty for Connection {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&format!(".{}", self.target));
        formatter.parenthesized(|f| {
            self.local.pretty_print(f);
        });
    }
}

impl ToTokens for Connection {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = format_ident!("{}", self.target);
        let local = &self.local;
        tokens.extend(quote! { .#target ( #local ) });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    pub module: String,
    pub instance: String,
    pub connections: Vec<Connection>,
}

impl Parse for Instance {
    fn parse(input: ParseStream) -> Result<Self> {
        let module = input.parse::<Ident>()?;
        let instance = input.parse::<Ident>()?;
        let connections = input.parse::<ParenCommaList<Connection>>()?;
        Ok(Self {
            module: module.to_string(),
            instance: instance.to_string(),
            connections: connections.inner.into_iter().collect(),
        })
    }
}

impl Pretty for Instance {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&format!("{} {}", self.module, self.instance));
        formatter.parenthesized(|f| {
            f.comma_separated(&self.connections);
        });
    }
}

impl ToTokens for Instance {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module = format_ident!("{}", self.module);
        let instance = format_ident!("{}", self.instance);
        let connections = &self.connections;
        tokens.extend(quote! { #module #instance ( #( #connections ),* ) ; });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub body: Vec<Stmt>,
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let _begin = input.parse::<kw::begin>()?;
        let mut body = Vec::new();
        loop {
            if input.peek(kw::end) {
                break;
            }
            body.push(input.parse()?);
        }
        let _end = input.parse::<kw::end>()?;
        Ok(Self { body })
    }
}

impl Pretty for Block {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write("begin");
        formatter.newline();
        formatter.scoped(|f| {
            f.lines(&self.body);
        });
        formatter.write("end");
    }
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let body = &self.body;
        tokens.extend(quote! {
            begin
                #( #body )*
            end
        });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct DynamicSplice {
    pub lhs: Box<ExprDynIndex>,
    pub rhs: Box<Expr>,
}

impl Parse for DynamicSplice {
    fn parse(input: ParseStream) -> Result<Self> {
        let lhs = input.parse::<Box<ExprDynIndex>>()?;
        let _eq = input.parse::<Token![=]>()?;
        let rhs = input.parse::<Box<Expr>>()?;
        Ok(Self { lhs, rhs })
    }
}

impl Pretty for DynamicSplice {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.lhs.pretty_print(formatter);
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
}

impl ToTokens for DynamicSplice {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let lhs = &self.lhs;
        let rhs = &self.rhs;
        tokens.extend(quote! { #lhs = #rhs ; });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Always {
    pub sensitivity: SensitivityList,
    pub body: Box<Stmt>,
}

impl Parse for Always {
    fn parse(input: ParseStream) -> Result<Self> {
        let _always = input.parse::<kw::always>()?;
        let sensitivity = input.parse()?;
        let body = input.parse()?;
        Ok(Always {
            sensitivity,
            body: Box::new(body),
        })
    }
}

impl Pretty for Always {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("always ");
        self.sensitivity.pretty_print(formatter);
        formatter.write(" ");
        self.body.pretty_print(formatter);
    }
}

impl ToTokens for Always {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sensitivity = &self.sensitivity;
        let body = &self.body;
        tokens.extend(quote! { always #sensitivity #body });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct LocalParam {
    pub target: String,
    pub rhs: ConstExpr,
}

impl Parse for LocalParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let _localparam = input.parse::<kw::localparam>()?;
        let target = input.parse::<Ident>()?;
        let _eq = input.parse::<Token![=]>()?;
        let rhs = input.parse()?;
        let _term = input.parse::<Token![;]>()?;
        Ok(LocalParam {
            target: target.to_string(),
            rhs,
        })
    }
}

impl Pretty for LocalParam {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("localparam ");
        formatter.write(&self.target);
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
}

impl ToTokens for LocalParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = format_ident!("{}", self.target);
        let rhs = &self.rhs;
        tokens.extend(quote! { localparam #target = #rhs ; });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum ConstExpr {
    LitVerilog(LitVerilog),
    LitInt(i32),
    LitStr(String),
}

impl Parse for ConstExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.fork().parse::<LitVerilog>().is_ok() {
            Ok(ConstExpr::LitVerilog(input.parse()?))
        } else if input.fork().parse::<LitInt>().is_ok() {
            Ok(ConstExpr::LitInt(input.parse::<LitInt>()?.base10_parse()?))
        } else if input.fork().parse::<LitStr>().is_ok() {
            Ok(ConstExpr::LitStr(input.parse::<LitStr>()?.value()))
        } else {
            Err(input.error("expected constant expression"))
        }
    }
}

impl Pretty for ConstExpr {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            ConstExpr::LitVerilog(lit) => lit.pretty_print(formatter),
            ConstExpr::LitInt(i) => formatter.write(&i.to_string()),
            ConstExpr::LitStr(s) => formatter.write(&format!("\"{}\"", s)),
        }
    }
}

impl ToTokens for ConstExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ConstExpr::LitVerilog(lit) => lit.to_tokens(tokens),
            ConstExpr::LitInt(i) => {
                let i = LitInt::new(&i.to_string(), proc_macro2::Span::call_site());
                tokens.extend(quote! { #i });
            }
            ConstExpr::LitStr(s) => {
                let s = LitStr::new(s, proc_macro2::Span::call_site());
                tokens.extend(quote! { #s });
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ElseBranch {
    pub stmt: Box<Stmt>,
}

impl Pretty for ElseBranch {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("else ");
        self.stmt.pretty_print(formatter);
    }
}

impl ToTokens for ElseBranch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let stmt = &self.stmt;
        tokens.extend(quote! { else #stmt });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct If {
    pub condition: Box<Expr>,
    pub true_stmt: Box<Stmt>,
    pub else_branch: Option<ElseBranch>,
}

impl Parse for If {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut clauses = Vec::new();
        let mut stmt;
        loop {
            let _if_token: Token![if] = input.parse()?;
            let condition = input.parse::<Parenthesized<Box<Expr>>>()?;
            let true_stmt = input.parse()?;

            stmt = If {
                condition: condition.inner,
                true_stmt,
                else_branch: None,
            };

            if !input.peek(Token![else]) {
                break;
            }

            let _else_token: Token![else] = input.parse()?;
            if input.peek(Token![if]) {
                stmt.else_branch = Some(ElseBranch {
                    stmt: Box::new(Stmt::default()),
                });
                clauses.push(stmt);
            } else {
                stmt.else_branch = Some(ElseBranch {
                    stmt: input.parse()?,
                });
                break;
            }
        }

        while let Some(mut prev) = clauses.pop() {
            *prev.else_branch.as_mut().unwrap().stmt = Stmt {
                kind: StmtKind::If(stmt),
            };
            stmt = prev;
        }
        Ok(stmt)
    }
}

impl Pretty for If {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("if ");
        formatter.parenthesized(|f| {
            self.condition.pretty_print(f);
        });
        formatter.write(" ");
        self.true_stmt.pretty_print(formatter);
        if let Some(else_branch) = &self.else_branch {
            formatter.write(" ");
            else_branch.pretty_print(formatter);
        }
    }
}

impl ToTokens for If {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let condition = &self.condition;
        let true_stmt = &self.true_stmt;
        let else_branch = &self.else_branch;
        tokens.extend(quote! { if ( #condition ) #true_stmt #else_branch });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum AssignTarget {
    Ident(String),
    Index(ExprIndex),
}

impl Parse for AssignTarget {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident) && input.peek2(token::Bracket) {
            Ok(AssignTarget::Index(input.parse()?))
        } else if input.peek(Ident) {
            Ok(AssignTarget::Ident(input.parse::<Ident>()?.to_string()))
        } else {
            Err(input.error("expected assignment target"))
        }
    }
}

impl Pretty for AssignTarget {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            AssignTarget::Ident(ident) => {
                formatter.write(ident);
            }
            AssignTarget::Index(index) => {
                index.pretty_print(formatter);
            }
        }
    }
}

impl ToTokens for AssignTarget {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            AssignTarget::Ident(ident) => {
                let ident = format_ident!("{}", ident);
                tokens.extend(quote! { #ident });
            }
            AssignTarget::Index(index) => {
                index.to_tokens(tokens);
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct NonblockAssign {
    pub target: AssignTarget,
    pub rhs: Box<Expr>,
}

impl Parse for NonblockAssign {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse()?;
        let _left_arrow = input.parse::<LeftArrow>()?;
        let rhs = input.parse()?;
        Ok(NonblockAssign { target, rhs })
    }
}

impl Pretty for NonblockAssign {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.target.pretty_print(formatter);
        formatter.write(" <= ");
        self.rhs.pretty_print(formatter);
    }
}

impl ToTokens for NonblockAssign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let rhs = &self.rhs;
        tokens.extend(quote! { #target <= #rhs ; });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ContinuousAssign {
    pub assign: Assign,
}

impl Parse for ContinuousAssign {
    fn parse(input: ParseStream) -> Result<Self> {
        let _kw_assign = input.parse::<kw::assign>()?;
        let assign = input.parse()?;
        Ok(ContinuousAssign { assign })
    }
}

impl Pretty for ContinuousAssign {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("assign ");
        self.assign.pretty_print(formatter);
    }
}

impl ToTokens for ContinuousAssign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let assign = &self.assign;
        tokens.extend(quote! { assign #assign });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Assign {
    pub target: AssignTarget,
    pub rhs: Box<Expr>,
}

impl Parse for Assign {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse()?;
        let _eq = input.parse::<Token![=]>()?;
        let rhs = input.parse()?;
        Ok(Assign { target, rhs })
    }
}

impl Pretty for Assign {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.target.pretty_print(formatter);
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
}

impl ToTokens for Assign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let rhs = &self.rhs;
        tokens.extend(quote! { #target = #rhs ; });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ConcatAssign {
    pub target: ExprConcat,
    pub rhs: Box<Expr>,
}

impl Parse for ConcatAssign {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse()?;
        let _eq = input.parse::<Token![=]>()?;
        let rhs = input.parse()?;
        Ok(ConcatAssign { target, rhs })
    }
}

impl Pretty for ConcatAssign {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.target.pretty_print(formatter);
        formatter.write(" = ");
        self.rhs.pretty_print(formatter);
    }
}

impl ToTokens for ConcatAssign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let rhs = &self.rhs;
        tokens.extend(quote! { #target = #rhs ; });
    }
}
