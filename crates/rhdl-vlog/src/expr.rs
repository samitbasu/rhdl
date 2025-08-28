use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Ident, Lifetime, LitInt, LitStr, Result, Token, braced, bracketed, parenthesized, token,
};

use crate::ParenCommaList;
use crate::atoms::LitVerilog;
use crate::formatter::{Formatter, Pretty};
use crate::kw_ops::{BinaryOp, DynOp, MinusColon, PlusColon, UnaryOp};

const TERNARY_BINDING: (u8, u8) = (2, 1);

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
            || lookahead.peek(crate::kw_ops::kw::begin)
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
