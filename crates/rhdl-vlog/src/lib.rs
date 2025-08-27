pub mod builder;
pub mod formatter;

// Re-export builder functions for convenient access
pub use builder::*;

use crate::formatter::{Formatter, Pretty};
use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::{
    Ident, Lifetime, LitInt, LitStr, Result, Token, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Bracket, Paren},
};

syn::custom_punctuation!(PlusColon, +:);

syn::custom_punctuation!(MinusColon, -:);
syn::custom_punctuation!(LeftArrow, <=);

syn::custom_punctuation!(CaseUnequal, !==);

syn::custom_punctuation!(CaseEqual, ===);

syn::custom_punctuation!(SignedRightShift, >>>);

mod kw {
    syn::custom_keyword!(input);
    syn::custom_keyword!(output);
    syn::custom_keyword!(inout);
    syn::custom_keyword!(reg);
    syn::custom_keyword!(wire);
    syn::custom_keyword!(signed);
    syn::custom_keyword!(assign);
    syn::custom_keyword!(always);
    syn::custom_keyword!(negedge);
    syn::custom_keyword!(posedge);
    syn::custom_keyword!(localparam);
    syn::custom_keyword!(begin);
    syn::custom_keyword!(end);
    syn::custom_keyword!(function);
    syn::custom_keyword!(endfunction);
    syn::custom_keyword!(case);
    syn::custom_keyword!(endcase);
    syn::custom_keyword!(default);
    syn::custom_keyword!(module);
    syn::custom_keyword!(endmodule);
    syn::custom_keyword!(initial);
}

#[derive(Debug, Clone, Hash, Copy, PartialEq, Serialize, Deserialize)]
pub enum HDLKind {
    Wire,
    Reg,
}

impl Parse for HDLKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::wire) {
            let _ = input.parse::<kw::wire>()?;
            Ok(HDLKind::Wire)
        } else if lookahead.peek(kw::reg) {
            let _ = input.parse::<kw::reg>()?;
            Ok(HDLKind::Reg)
        } else {
            Err(lookahead.error())
        }
    }
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
            HDLKind::Wire => quote! { wire },
            HDLKind::Reg => quote! { reg },
        });
    }
}

#[derive(Debug, Clone, Hash, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

impl Parse for Direction {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::input) {
            let _ = input.parse::<kw::input>()?;
            Ok(Direction::Input)
        } else if lookahead.peek(kw::output) {
            let _ = input.parse::<kw::output>()?;
            Ok(Direction::Output)
        } else if lookahead.peek(kw::inout) {
            let _ = input.parse::<kw::inout>()?;
            Ok(Direction::Inout)
        } else {
            Err(lookahead.error())
        }
    }
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

impl ToTokens for Direction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Direction::Input => quote! { input },
            Direction::Output => quote! { output },
            Direction::Inout => quote! { inout },
        });
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct BitRange {
    pub start: u32,
    pub end: u32,
}

pub fn bit_range(start: u32, end: u32) -> BitRange {
    BitRange { start, end }
}

impl Parse for BitRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = input.parse::<LitInt>()?;
        let _ = input.parse::<Token![:]>()?;
        let end = input.parse::<LitInt>()?;
        let start = start.base10_parse::<u32>()?;
        let end = end.base10_parse::<u32>()?;
        Ok(BitRange { start, end })
    }
}

impl Pretty for BitRange {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.start.to_string());
        formatter.write(":");
        formatter.write(&self.end.to_string());
    }
}

impl ToTokens for BitRange {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let start = syn::Index::from(self.start as usize);
        let end = syn::Index::from(self.end as usize);
        tokens.extend(quote! { #start : #end });
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct WidthSpec {
    pub bit_range: BitRange,
}

pub fn width_spec(bit_range: BitRange) -> WidthSpec {
    WidthSpec { bit_range }
}

impl From<BitRange> for WidthSpec {
    fn from(bit_range: BitRange) -> Self {
        WidthSpec { bit_range }
    }
}

impl Parse for WidthSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _bracket = bracketed!(content in input);
        let bit_range = content.parse()?;
        Ok(WidthSpec { bit_range })
    }
}

impl Pretty for WidthSpec {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.bracketed(|f| {
            self.bit_range.pretty_print(f);
        });
    }
}

impl ToTokens for WidthSpec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let bit_range = &self.bit_range;
        tokens.extend(quote! { [ #bit_range ] });
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum SignedWidth {
    Signed(WidthSpec),
    Unsigned(WidthSpec),
}

impl From<WidthSpec> for SignedWidth {
    fn from(width_spec: WidthSpec) -> Self {
        SignedWidth::Unsigned(width_spec)
    }
}

impl Parse for SignedWidth {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::signed) {
            let _ = input.parse::<kw::signed>()?;
            let bit_range = input.parse()?;
            Ok(SignedWidth::Signed(bit_range))
        } else {
            let bit_range = input.parse()?;
            Ok(SignedWidth::Unsigned(bit_range))
        }
    }
}

impl Pretty for SignedWidth {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        match self {
            SignedWidth::Signed(width) => {
                formatter.write("signed ");
                width.pretty_print(formatter);
            }
            SignedWidth::Unsigned(width) => {
                width.pretty_print(formatter);
            }
        }
    }
}

impl ToTokens for SignedWidth {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            SignedWidth::Signed(width) => {
                tokens.extend(quote! { signed });
                width.to_tokens(tokens);
            }
            SignedWidth::Unsigned(width) => {
                width.to_tokens(tokens);
            }
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct DeclKind {
    pub name: String,
    pub width: Option<SignedWidth>,
}

impl Parse for DeclKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let width = if input.peek(token::Bracket) || input.peek(kw::signed) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(DeclKind {
            name: name.to_string(),
            width,
        })
    }
}

impl Pretty for DeclKind {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.name);
        self.width.pretty_print(formatter);
    }
}

impl ToTokens for DeclKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.name);
        let width = &self.width;
        tokens.extend(quote! { #name #width });
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct DeclarationList {
    pub kind: HDLKind,
    pub signed_width: Option<SignedWidth>,
    pub items: Vec<DeclKind>,
}

impl Parse for DeclarationList {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse()?;
        let signed_width = if input.peek(token::Bracket) || input.peek(kw::signed) {
            Some(input.parse()?)
        } else {
            None
        };
        let items = Punctuated::<DeclKind, Token![,]>::parse_separated_nonempty(input)?;
        let _term = input.parse::<Token![;]>()?;
        Ok(DeclarationList {
            kind,
            signed_width,
            items: items.into_iter().collect(),
        })
    }
}

impl Pretty for DeclarationList {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.kind.pretty_print(formatter);
        formatter.write(" ");
        self.signed_width.pretty_print(formatter);
        formatter.write(" ");
        formatter.comma_separated(self.items.iter());
    }
}

impl ToTokens for DeclarationList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let kind = &self.kind;
        let signed_width = &self.signed_width;
        let items = &self.items;
        tokens.extend(quote! { #kind #signed_width #(#items),* ; });
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    pub kind: HDLKind,
    pub signed_width: Option<SignedWidth>,
    pub name: String,
}

impl Parse for Declaration {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse()?;
        let signed_width = if input.peek(token::Bracket) || input.peek(kw::signed) {
            Some(input.parse()?)
        } else {
            None
        };
        let name: Ident = input.parse()?;
        Ok(Declaration {
            kind,
            signed_width,
            name: name.to_string(),
        })
    }
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

impl ToTokens for Declaration {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.kind.to_tokens(tokens);
        self.signed_width.to_tokens(tokens);
        let name = format_ident!("{}", self.name);
        tokens.extend(quote! { #name });
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub direction: Direction,
    pub decl: Declaration,
}

impl Parse for Port {
    fn parse(input: ParseStream) -> Result<Self> {
        let direction = input.parse()?;
        let decl = input.parse()?;
        Ok(Port { direction, decl })
    }
}

impl Pretty for Port {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        self.direction.pretty_print(formatter);
        formatter.write(" ");
        self.decl.pretty_print(formatter);
    }
}

impl ToTokens for Port {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let direction = &self.direction;
        let decl = &self.decl;
        direction.to_tokens(tokens);
        decl.to_tokens(tokens);
    }
}

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
        } else if lookahead.peek(Ident) && input.peek2(Bracket) {
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
pub enum Item {
    Statement(Stmt),
    Declaration(DeclarationList),
    FunctionDef(FunctionDef),
    Initial(Initial),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::function) {
            input.parse().map(Item::FunctionDef)
        } else if input.peek(kw::reg) || input.peek(kw::wire) {
            input.parse().map(Item::Declaration)
        } else if input.peek(kw::initial) {
            input.parse().map(Item::Initial)
        } else {
            input.parse().map(Item::Statement)
        }
    }
}

impl Pretty for Item {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            Item::Statement(stmt) => stmt.pretty_print(formatter),
            Item::Declaration(decl) => decl.pretty_print(formatter),
            Item::FunctionDef(func) => func.pretty_print(formatter),
            Item::Initial(initial) => initial.pretty_print(formatter),
        }
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Item::Statement(stmt) => stmt.to_tokens(tokens),
            Item::Declaration(decl) => decl.to_tokens(tokens),
            Item::FunctionDef(func) => func.to_tokens(tokens),
            Item::Initial(initial) => initial.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Initial {
    pub statement: Stmt,
}

impl Parse for Initial {
    fn parse(input: ParseStream) -> Result<Self> {
        let _initial_kw: kw::initial = input.parse()?;
        let statement = input.parse()?;
        Ok(Initial { statement })
    }
}

impl Pretty for Initial {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("initial ");
        self.statement.pretty_print(formatter);
    }
}

impl ToTokens for Initial {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let statement = &self.statement;
        tokens.extend(quote! { initial #statement });
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

struct ParenCommaList<T: Parse> {
    _paren: Paren,
    inner: Punctuated<T, Token![,]>,
}

impl<T: Parse> Parse for ParenCommaList<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _paren = parenthesized!(content in input);
        let inner = Punctuated::<T, Token![,]>::parse_terminated(&content)?;
        Ok(Self { _paren, inner })
    }
}

struct Parenthesized<T: Parse> {
    _paren: Paren,
    inner: T,
}

impl<T: Parse> Parse for Parenthesized<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _paren = parenthesized!(content in input);
        let inner = content.parse()?;
        Ok(Self { _paren, inner })
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

pub fn dynamic_splice(lhs: ExprDynIndex, rhs: Expr) -> DynamicSplice {
    DynamicSplice {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
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
pub enum Sensitivity {
    PosEdge(PosEdgeSensitivity),
    NegEdge(NegEdgeSensitivity),
    Signal(String),
    Star,
}

impl Parse for Sensitivity {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::posedge) {
            Ok(Sensitivity::PosEdge(input.parse()?))
        } else if lookahead.peek(kw::negedge) {
            Ok(Sensitivity::NegEdge(input.parse()?))
        } else if lookahead.peek(Ident) {
            Ok(Sensitivity::Signal(input.parse::<Ident>()?.to_string()))
        } else if lookahead.peek(Token![*]) {
            let _ = input.parse::<Token![*]>()?;
            Ok(Sensitivity::Star)
        } else {
            Err(input.error("expected sensitivity"))
        }
    }
}

impl Pretty for Sensitivity {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            Sensitivity::PosEdge(pos) => {
                pos.pretty_print(formatter);
            }
            Sensitivity::NegEdge(neg) => {
                neg.pretty_print(formatter);
            }
            Sensitivity::Signal(sig) => {
                formatter.write(sig);
            }
            Sensitivity::Star => {
                formatter.write("*");
            }
        }
    }
}

impl ToTokens for Sensitivity {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Sensitivity::PosEdge(pos) => pos.to_tokens(tokens),
            Sensitivity::NegEdge(neg) => neg.to_tokens(tokens),
            Sensitivity::Signal(sig) => {
                let sig = format_ident!("{}", sig);
                tokens.extend(quote! { #sig });
            }
            Sensitivity::Star => {
                tokens.extend(quote! { * });
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct PosEdgeSensitivity {
    pub ident: String,
}

impl Parse for PosEdgeSensitivity {
    fn parse(input: ParseStream) -> Result<Self> {
        let _posedge = input.parse::<kw::posedge>()?;
        let ident = input.parse::<Ident>()?;
        Ok(PosEdgeSensitivity {
            ident: ident.to_string(),
        })
    }
}

impl Pretty for PosEdgeSensitivity {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("posedge ");
        formatter.write(&self.ident);
    }
}

impl ToTokens for PosEdgeSensitivity {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = format_ident!("{}", self.ident);
        tokens.extend(quote! { posedge #ident });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct NegEdgeSensitivity {
    pub ident: String,
}

impl Parse for NegEdgeSensitivity {
    fn parse(input: ParseStream) -> Result<Self> {
        let _negedge = input.parse::<kw::negedge>()?;
        let ident = input.parse::<Ident>()?;
        Ok(NegEdgeSensitivity {
            ident: ident.to_string(),
        })
    }
}

impl Pretty for NegEdgeSensitivity {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("negedge ");
        formatter.write(&self.ident);
    }
}

impl ToTokens for NegEdgeSensitivity {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = format_ident!("{}", self.ident);
        tokens.extend(quote! { negedge #ident });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SensitivityList {
    pub elements: Vec<Sensitivity>,
}

impl Parse for SensitivityList {
    fn parse(input: ParseStream) -> Result<Self> {
        let _at = input.parse::<Token![@]>()?;
        let elements = input.parse::<ParenCommaList<Sensitivity>>()?;
        Ok(SensitivityList {
            elements: elements.inner.into_iter().collect(),
        })
    }
}

impl Pretty for SensitivityList {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write("@");
        formatter.parenthesized(|f| {
            f.comma_separated(self.elements.iter());
        });
    }
}

impl ToTokens for SensitivityList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let elements = &self.elements;
        tokens.extend(quote! { @ ( #( #elements ),* ) });
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

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Constant(LitVerilog),
    Literal(i32),
    String(String),
    Ident(String),
    Paren(Box<Expr>),
    Ternary(ExprTernary),
    Concat(ExprConcat),
    Replica(ExprReplica),
    Index(ExprIndex),
    DynIndex(ExprDynIndex),
    Function(ExprFunction),
}
impl Parse for Expr {
    fn parse(mut input: ParseStream) -> Result<Self> {
        expr_bp(&mut input, 0)
    }
}

impl Pretty for Expr {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            Expr::Binary(expr) => expr.pretty_print(formatter),
            Expr::Unary(expr) => expr.pretty_print(formatter),
            Expr::Constant(lit) => lit.pretty_print(formatter),
            Expr::Literal(i) => formatter.write(&i.to_string()),
            Expr::String(s) => formatter.write(&format!("\"{}\"", s)),
            Expr::Ident(ident) => formatter.write(ident),
            Expr::Paren(expr) => {
                formatter.parenthesized(|f| expr.pretty_print(f));
            }
            Expr::Ternary(expr) => expr.pretty_print(formatter),
            Expr::Concat(expr) => expr.pretty_print(formatter),
            Expr::Replica(expr) => expr.pretty_print(formatter),
            Expr::Index(expr) => expr.pretty_print(formatter),
            Expr::DynIndex(expr) => expr.pretty_print(formatter),
            Expr::Function(expr) => expr.pretty_print(formatter),
        }
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Expr::Binary(expr) => expr.to_tokens(tokens),
            Expr::Unary(expr) => expr.to_tokens(tokens),
            Expr::Constant(lit) => lit.to_tokens(tokens),
            Expr::Literal(i) => {
                let i = LitInt::new(&i.to_string(), proc_macro2::Span::call_site());
                tokens.extend(quote! { #i });
            }
            Expr::String(s) => {
                let s = LitStr::new(s, proc_macro2::Span::call_site());
                tokens.extend(quote! { #s });
            }
            Expr::Ident(ident) => {
                let ident = format_ident!("{}", ident);
                tokens.extend(quote! { #ident });
            }
            Expr::Paren(expr) => {
                let expr = &**expr;
                tokens.extend(quote! { ( #expr ) });
            }
            Expr::Ternary(expr) => expr.to_tokens(tokens),
            Expr::Concat(expr) => expr.to_tokens(tokens),
            Expr::Replica(expr) => expr.to_tokens(tokens),
            Expr::Index(expr) => expr.to_tokens(tokens),
            Expr::DynIndex(expr) => expr.to_tokens(tokens),
            Expr::Function(expr) => expr.to_tokens(tokens),
        }
    }
}

// matklad's Pratt parser.  From
// https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn expr_bp(input: &mut ParseStream, min_bp: u8) -> Result<Expr> {
    let lookahead = input.lookahead1();
    let mut lhs = if lookahead.peek(LitInt) && input.peek2(Lifetime) {
        input.parse().map(Expr::Constant)?
    } else if lookahead.peek(LitInt) {
        input
            .parse::<LitInt>()?
            .base10_parse::<i32>()
            .map(Expr::Literal)?
    } else if lookahead.peek(LitStr) {
        input
            .parse::<LitStr>()
            .map(|lit| Expr::String(lit.value()))?
    } else if input.fork().parse::<ExprFunction>().is_ok() {
        input.parse().map(Expr::Function)?
    } else if input.fork().parse::<ExprIndex>().is_ok() {
        input.parse().map(Expr::Index)?
    } else if input.fork().parse::<ExprDynIndex>().is_ok() {
        input.parse().map(Expr::DynIndex)?
    } else if lookahead.peek(Ident) {
        input.parse::<Ident>().map(|x| Expr::Ident(x.to_string()))?
    } else if lookahead.peek(token::Paren) {
        let content;
        parenthesized!(content in input);
        let expr = content.parse::<Expr>()?;
        Expr::Paren(Box::new(expr))
    } else if lookahead.peek(token::Brace) {
        // Try to parse as a replica
        if input.fork().parse::<ExprReplica>().is_ok() {
            input.parse().map(Expr::Replica)?
        } else {
            input.parse().map(Expr::Concat)?
        }
    } else {
        let op = input.parse::<UnaryOp>()?;
        let r_bp = op.binding_power();
        let arg = Box::new(expr_bp(input, r_bp)?);
        Expr::Unary(ExprUnary { op, arg })
    };
    loop {
        if input.is_empty() {
            break;
        }
        // Check for a trinary operator
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![:])
            || lookahead.peek(Token![,])
            || lookahead.peek(PlusColon)
            || lookahead.peek(MinusColon)
            || lookahead.peek(Token![;])
            || lookahead.peek(kw::begin)
        {
            break;
        } else if lookahead.peek(Token![?]) {
            let (l_bp, r_bp) = TERNARY_BINDING;
            if l_bp < min_bp {
                break;
            }
            let _op = input.parse::<Token![?]>()?;
            let mhs = Box::new(expr_bp(input, 0)?);
            let _colon = input.parse::<Token![:]>()?;
            let rhs = Box::new(expr_bp(input, r_bp)?);
            lhs = Expr::Ternary(ExprTernary {
                lhs: Box::new(lhs),
                mhs,
                rhs,
            });
        } else {
            let op = input.fork().parse::<BinaryOp>()?;
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp {
                break;
            }
            let _ = input.parse::<BinaryOp>()?;
            let rhs = Box::new(expr_bp(input, r_bp)?);
            lhs = Expr::Binary(ExprBinary {
                op,
                lhs: Box::new(lhs),
                rhs,
            });
        }
    }
    Ok(lhs)
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprBinary {
    pub lhs: Box<Expr>,
    pub op: BinaryOp,
    pub rhs: Box<Expr>,
}

impl Pretty for ExprBinary {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.lhs.pretty_print(formatter);
        formatter.write(" ");
        self.op.pretty_print(formatter);
        formatter.write(" ");
        self.rhs.pretty_print(formatter);
    }
}

impl ToTokens for ExprBinary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let lhs = &self.lhs;
        let op = &self.op;
        let rhs = &self.rhs;
        tokens.extend(quote! { #lhs #op #rhs });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprUnary {
    pub op: UnaryOp,
    pub arg: Box<Expr>,
}

impl Pretty for ExprUnary {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.op.pretty_print(formatter);
        self.arg.pretty_print(formatter);
    }
}

impl ToTokens for ExprUnary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let op = &self.op;
        let arg = &self.arg;
        tokens.extend(quote! { #op #arg });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprConcat {
    pub elements: Vec<Expr>,
}

impl Parse for ExprConcat {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _brace = braced!(content in input);
        let elements = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;
        Ok(ExprConcat {
            elements: elements.into_iter().collect(),
        })
    }
}

impl Pretty for ExprConcat {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.braced(|f| {
            f.comma_separated(self.elements.iter());
        });
    }
}

impl ToTokens for ExprConcat {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let elements = &self.elements;
        tokens.extend(quote! { { #( #elements ),* } });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprReplicaInner {
    pub count: u32,
    pub concatenation: ExprConcat,
}

impl Parse for ExprReplicaInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let count: LitInt = input.parse()?;
        let count = count.base10_parse::<u32>()?;
        let concatenation: ExprConcat = input.parse()?;
        Ok(ExprReplicaInner {
            count,
            concatenation,
        })
    }
}

impl Pretty for ExprReplicaInner {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.braced(|f| {
            f.write(&self.count.to_string());
            self.concatenation.pretty_print(f);
        });
    }
}

impl ToTokens for ExprReplicaInner {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let count = self.count;
        let concatenation = &self.concatenation;
        tokens.extend(quote! { { #count #concatenation } });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprReplica {
    pub inner: ExprReplicaInner,
}

impl Parse for ExprReplica {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _brace = braced!(content in input);
        let inner = content.parse()?;
        Ok(ExprReplica { inner })
    }
}

impl Pretty for ExprReplica {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.inner.pretty_print(formatter);
    }
}

impl ToTokens for ExprReplica {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.inner.to_tokens(tokens);
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprFunction {
    pub name: String,
    pub args: Vec<Expr>,
}

impl Parse for ExprFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let dollar = if input.peek(Token![$]) {
            Some(input.parse::<Token![$]>()?)
        } else {
            None
        };
        let name = input.parse::<Ident>()?;
        let args = input.parse::<ParenCommaList<Expr>>()?;
        let name = if dollar.is_some() {
            format!("${}", name)
        } else {
            name.to_string()
        };
        Ok(ExprFunction {
            name: name.to_string(),
            args: args.inner.into_iter().collect(),
        })
    }
}

impl Pretty for ExprFunction {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write(&self.name);
        formatter.parenthesized(|f| {
            f.comma_separated(self.args.iter());
        });
    }
}

impl ToTokens for ExprFunction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.name.starts_with('$') {
            let ident = format_ident!("{}", &self.name[1..]);
            tokens.extend(quote! { $ #ident });
        } else {
            let ident = format_ident!("{}", &self.name);
            tokens.extend(quote! { #ident });
        }
        let args = &self.args;
        tokens.extend(quote! { ( #( #args ),* ) });
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub enum DynOp {
    PlusColon,
    MinusColon,
}

impl Parse for DynOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(PlusColon) {
            let _: PlusColon = input.parse()?;
            Ok(DynOp::PlusColon)
        } else if input.peek(MinusColon) {
            let _: MinusColon = input.parse()?;
            Ok(DynOp::MinusColon)
        } else {
            Err(input.error("expected dynamic operator"))
        }
    }
}

impl Pretty for DynOp {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            DynOp::PlusColon => formatter.write("+:"),
            DynOp::MinusColon => formatter.write("-:"),
        }
    }
}

impl ToTokens for DynOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DynOp::PlusColon => {
                let op = PlusColon::default();
                tokens.extend(quote! { #op });
            }
            DynOp::MinusColon => {
                let op = MinusColon::default();
                tokens.extend(quote! { #op });
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprDynIndexInner {
    pub base: Box<Expr>,
    pub op: DynOp,
    pub width: Box<Expr>,
}

impl Parse for ExprDynIndexInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let base = input.parse()?;
        let op = input.parse()?;
        let width = input.parse()?;
        Ok(ExprDynIndexInner { base, op, width })
    }
}

impl Pretty for ExprDynIndexInner {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.base.pretty_print(formatter);
        self.op.pretty_print(formatter);
        self.width.pretty_print(formatter);
    }
}

impl ToTokens for ExprDynIndexInner {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let base = &self.base;
        let op = &self.op;
        let width = &self.width;
        tokens.extend(quote! { #base #op #width });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprDynIndex {
    pub target: String,
    pub address: ExprDynIndexInner,
}

impl Parse for ExprDynIndex {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse::<Ident>()?;
        let content;
        let _bracket = bracketed!(content in input);
        let address = content.parse()?;
        Ok(ExprDynIndex {
            target: target.to_string(),
            address,
        })
    }
}

impl Pretty for ExprDynIndex {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write(&self.target);
        formatter.bracketed(|f| {
            self.address.pretty_print(f);
        });
    }
}

impl ToTokens for ExprDynIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = format_ident!("{}", self.target);
        let address = &self.address;
        tokens.extend(quote! { #target [ #address ] });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprIndexAddress {
    pub msb: Box<Expr>,
    pub lsb: Option<Box<Expr>>,
}

impl Parse for ExprIndexAddress {
    fn parse(input: ParseStream) -> Result<Self> {
        let msb = input.parse()?;
        if !input.is_empty() {
            let _colon = input.parse::<Token![:]>()?;
            let lsb = input.parse()?;
            Ok(ExprIndexAddress {
                msb,
                lsb: Some(lsb),
            })
        } else {
            Ok(ExprIndexAddress { msb, lsb: None })
        }
    }
}

impl Pretty for ExprIndexAddress {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.msb.pretty_print(formatter);
        if let Some(lsb) = &self.lsb {
            formatter.write(":");
            lsb.pretty_print(formatter);
        }
    }
}

impl ToTokens for ExprIndexAddress {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let msb = &self.msb;
        if let Some(lsb) = &self.lsb {
            tokens.extend(quote! { #msb : #lsb });
        } else {
            tokens.extend(quote! { #msb });
        }
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprIndex {
    pub target: String,
    pub address: ExprIndexAddress,
}

impl Parse for ExprIndex {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse::<Ident>()?;
        let content;
        let _bracket = bracketed!(content in input);
        let address = content.parse()?;
        Ok(ExprIndex {
            target: target.to_string(),
            address,
        })
    }
}

impl Pretty for ExprIndex {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write(&self.target);
        formatter.bracketed(|f| {
            self.address.pretty_print(f);
        });
    }
}

impl ToTokens for ExprIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = format_ident!("{}", self.target);
        let address = &self.address;
        tokens.extend(quote! { #target [ #address ] });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ExprTernary {
    pub lhs: Box<Expr>,
    pub mhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Pretty for ExprTernary {
    fn pretty_print(&self, formatter: &mut Formatter) {
        self.lhs.pretty_print(formatter);
        formatter.write(" ? ");
        self.mhs.pretty_print(formatter);
        formatter.write(" : ");
        self.rhs.pretty_print(formatter);
    }
}

impl ToTokens for ExprTernary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let lhs = &self.lhs;
        let mhs = &self.mhs;
        let rhs = &self.rhs;
        tokens.extend(quote! { #lhs ? #mhs : #rhs });
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Plus,
    Minus,
    Bang,
    Not,
    And,
    Or,
    Xor,
}

impl Parse for UnaryOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![+]) {
            let _: Token![+] = input.parse()?;
            Ok(UnaryOp::Plus)
        } else if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(UnaryOp::Minus)
        } else if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            Ok(UnaryOp::Bang)
        } else if input.peek(Token![~]) {
            let _: Token![~] = input.parse()?;
            Ok(UnaryOp::Not)
        } else if input.peek(Token![&]) {
            let _: Token![&] = input.parse()?;
            Ok(UnaryOp::And)
        } else if input.peek(Token![|]) {
            let _: Token![|] = input.parse()?;
            Ok(UnaryOp::Or)
        } else if input.peek(Token![^]) {
            let _: Token![^] = input.parse()?;
            Ok(UnaryOp::Xor)
        } else {
            Err(input.error("expected unary operator"))
        }
    }
}

impl Pretty for UnaryOp {
    fn pretty_print(&self, formatter: &mut Formatter) {
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

impl ToTokens for UnaryOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            UnaryOp::Plus => {
                tokens.extend(quote! { + });
            }
            UnaryOp::Minus => {
                tokens.extend(quote! { - });
            }
            UnaryOp::Bang => {
                tokens.extend(quote! { ! });
            }
            UnaryOp::Not => {
                tokens.extend(quote! { ~ });
            }
            UnaryOp::And => {
                tokens.extend(quote! { & });
            }
            UnaryOp::Or => {
                tokens.extend(quote! { | });
            }
            UnaryOp::Xor => {
                tokens.extend(quote! { ^ });
            }
        }
    }
}

impl UnaryOp {
    fn binding_power(&self) -> u8 {
        50
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Serialize, Deserialize)]
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

impl Parse for BinaryOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<<]) {
            let _: Token![<<] = input.parse()?;
            Ok(BinaryOp::Shl)
        } else if lookahead.peek(SignedRightShift) {
            let _: SignedRightShift = input.parse()?;
            Ok(BinaryOp::SignedRightShift)
        } else if lookahead.peek(Token![>>]) {
            let _: Token![>>] = input.parse()?;
            Ok(BinaryOp::Shr)
        } else if lookahead.peek(Token![&&]) {
            let _: Token![&&] = input.parse()?;
            Ok(BinaryOp::ShortAnd)
        } else if lookahead.peek(Token![||]) {
            let _: Token![||] = input.parse()?;
            Ok(BinaryOp::ShortOr)
        } else if lookahead.peek(CaseEqual) {
            let _: CaseEqual = input.parse()?;
            Ok(BinaryOp::CaseEq)
        } else if lookahead.peek(CaseUnequal) {
            let _: CaseUnequal = input.parse()?;
            Ok(BinaryOp::CaseNe)
        } else if lookahead.peek(Token![!=]) {
            let _: Token![!=] = input.parse()?;
            Ok(BinaryOp::Ne)
        } else if lookahead.peek(Token![==]) {
            let _: Token![==] = input.parse()?;
            Ok(BinaryOp::Eq)
        } else if lookahead.peek(Token![>=]) {
            let _: Token![>=] = input.parse()?;
            Ok(BinaryOp::Ge)
        } else if lookahead.peek(Token![<=]) {
            let _: Token![<=] = input.parse()?;
            Ok(BinaryOp::Le)
        } else if lookahead.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            Ok(BinaryOp::Gt)
        } else if lookahead.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            Ok(BinaryOp::Lt)
        } else if lookahead.peek(Token![+]) {
            let _: Token![+] = input.parse()?;
            Ok(BinaryOp::Plus)
        } else if lookahead.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(BinaryOp::Minus)
        } else if lookahead.peek(Token![&]) {
            let _: Token![&] = input.parse()?;
            Ok(BinaryOp::And)
        } else if lookahead.peek(Token![|]) {
            let _: Token![|] = input.parse()?;
            Ok(BinaryOp::Or)
        } else if lookahead.peek(Token![^]) {
            let _: Token![^] = input.parse()?;
            Ok(BinaryOp::Xor)
        } else if lookahead.peek(Token![%]) {
            let _: Token![%] = input.parse()?;
            Ok(BinaryOp::Mod)
        } else if lookahead.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            Ok(BinaryOp::Mul)
        } else {
            Err(input.error("expected binary operator"))
        }
    }
}

impl Pretty for BinaryOp {
    fn pretty_print(&self, formatter: &mut Formatter) {
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

impl ToTokens for BinaryOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            BinaryOp::Shl => {
                tokens.extend(quote! { << });
            }
            BinaryOp::SignedRightShift => {
                let op = SignedRightShift::default();
                tokens.extend(quote! { #op });
            }
            BinaryOp::Shr => {
                tokens.extend(quote! { >> });
            }
            BinaryOp::ShortAnd => {
                tokens.extend(quote! { && });
            }
            BinaryOp::ShortOr => {
                tokens.extend(quote! { || });
            }
            BinaryOp::CaseEq => {
                let op = CaseEqual::default();
                tokens.extend(quote! { #op });
            }
            BinaryOp::CaseNe => {
                let op = CaseUnequal::default();
                tokens.extend(quote! { #op });
            }
            BinaryOp::Ne => {
                tokens.extend(quote! { != });
            }
            BinaryOp::Eq => {
                tokens.extend(quote! { == });
            }
            BinaryOp::Ge => {
                tokens.extend(quote! { >= });
            }
            BinaryOp::Le => {
                tokens.extend(quote! { <= });
            }
            BinaryOp::Gt => {
                tokens.extend(quote! { > });
            }
            BinaryOp::Lt => {
                tokens.extend(quote! { < });
            }
            BinaryOp::Plus => {
                tokens.extend(quote! { + });
            }
            BinaryOp::Minus => {
                tokens.extend(quote! { - });
            }
            BinaryOp::And => {
                tokens.extend(quote! { & });
            }
            BinaryOp::Or => {
                tokens.extend(quote! { | });
            }
            BinaryOp::Xor => {
                tokens.extend(quote! { ^ });
            }
            BinaryOp::Mod => {
                tokens.extend(quote! { % });
            }
            BinaryOp::Mul => {
                tokens.extend(quote! { * });
            }
        }
    }
}

impl BinaryOp {
    fn binding_power(&self) -> (u8, u8) {
        match self {
            BinaryOp::Mod | BinaryOp::Mul => (20, 21),
            BinaryOp::Plus | BinaryOp::Minus => (18, 19),
            BinaryOp::Shl | BinaryOp::Shr | BinaryOp::SignedRightShift => (16, 17),
            BinaryOp::Ge | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Lt => (14, 15),
            BinaryOp::Ne | BinaryOp::Eq | BinaryOp::CaseNe | BinaryOp::CaseEq => (12, 13),
            BinaryOp::And => (10, 11),
            BinaryOp::Xor => (9, 10),
            BinaryOp::Or => (7, 8),
            BinaryOp::ShortAnd => (5, 6),
            BinaryOp::ShortOr => (3, 4),
        }
    }
}
const TERNARY_BINDING: (u8, u8) = (2, 1);

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct LitVerilog {
    pub width: u32,
    pub value: String,
}

impl Parse for LitVerilog {
    fn parse(input: ParseStream) -> Result<Self> {
        let width: LitInt = input.parse()?;
        let lifetime: Lifetime = input.parse()?;
        let width = width.base10_parse::<u32>()?;
        let value = lifetime.ident.to_string();
        Ok(LitVerilog { width, value })
    }
}

impl Pretty for LitVerilog {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write(&self.width.to_string());
        formatter.write("'");
        formatter.write(&self.value);
    }
}

impl ToTokens for LitVerilog {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let width = syn::Index::from(self.width as usize);
        let lifetime =
            syn::Lifetime::new(&format!("'{}", self.value), proc_macro2::Span::call_site());
        tokens.extend(quote! { #width #lifetime });
    }
}

#[derive(Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModuleList {
    pub modules: Vec<ModuleDef>,
}

impl Parse for ModuleList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut modules = Vec::new();
        loop {
            if input.is_empty() {
                break;
            }
            modules.push(input.parse()?);
        }
        Ok(Self { modules })
    }
}

impl Pretty for ModuleList {
    fn pretty_print(&self, formatter: &mut Formatter) {
        for module in &self.modules {
            module.pretty_print(formatter);
            formatter.newline();
        }
    }
}

impl ToTokens for ModuleList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let modules = &self.modules;
        tokens.extend(quote! { #( #modules )* });
    }
}

impl std::fmt::Display for ModuleList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = crate::formatter::Formatter::new();
        self.pretty_print(&mut fmt);
        write!(f, "{}", fmt.finish())
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ModuleDef {
    pub name: String,
    pub args: Vec<Port>,
    pub items: Vec<Item>,
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

impl Parse for ModuleDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let _module = input.parse::<kw::module>()?;
        let name = input.parse::<Ident>()?;
        let args = if input.peek(token::Paren) {
            Some(input.parse::<ParenCommaList<Port>>()?)
        } else {
            None
        };
        let _semi = input.parse::<Token![;]>()?;
        let mut items = Vec::new();
        while !input.peek(kw::endmodule) {
            items.push(input.parse()?);
        }
        let _end_module = input.parse::<kw::endmodule>()?;
        Ok(Self {
            name: name.to_string(),
            args: args.into_iter().flat_map(|x| x.inner.into_iter()).collect(),
            items,
        })
    }
}

impl Pretty for ModuleDef {
    fn pretty_print(&self, formatter: &mut Formatter) {
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

impl ToTokens for ModuleDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.name);
        let args = &self.args;
        let items = &self.items;
        tokens.extend(quote! {
            module #name ( #( #args ),* );
            #( #items )*
            endmodule
        });
    }
}

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub signed_width: SignedWidth,
    pub name: String,
    pub args: Vec<Port>,
    pub items: Vec<Item>,
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

impl Parse for FunctionDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let _function: kw::function = input.parse()?;
        let signed_width = input.parse()?;
        let name: Ident = input.parse()?;
        let args: ParenCommaList<Port> = input.parse()?;
        let _semi: Token![;] = input.parse()?;
        let mut items: Vec<Item> = Vec::new();
        while !input.peek(kw::endfunction) {
            items.push(input.parse()?);
        }
        let _end_function = input.parse::<kw::endfunction>()?;
        Ok(FunctionDef {
            signed_width,
            name: name.to_string(),
            args: args.inner.into_iter().collect(),
            items,
        })
    }
}

impl Pretty for FunctionDef {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write(&format!("function "));
        formatter.scoped(|formatter| {
            self.signed_width.pretty_print(formatter);
            formatter.write(&format!(" {}", self.name));
            formatter.parenthesized(|f| f.comma_separated(&self.args));
            formatter.write(";");
            formatter.newline();
            formatter.scoped(|f| {
                f.lines(&self.items);
            });
        });
        formatter.write("endfunction");
    }
}

impl ToTokens for FunctionDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let signed_width = &self.signed_width;
        let name = format_ident!("{}", self.name);
        let args = &self.args;
        let items = &self.items;
        tokens.extend(quote! {
            function #signed_width #name ( #( #args ),* );
            #( #items )*
            endfunction
        });
    }
}

#[cfg(test)]
mod tests;
