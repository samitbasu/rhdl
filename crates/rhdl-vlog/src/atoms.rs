//! Support for atoms - basic building blocks of Verilog HDL.
//!
//! Note that none of these terms are spec compliant Verilog terminology; they are
//! simply convenient names for concepts that appear in Verilog HDL.

use crate::{
    ParenCommaList,
    formatter::{Formatter, Pretty},
};
use proc_macro2::TokenTree;
use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::{
    Ident, Lifetime, LitFloat, LitInt, LitStr, Result, Token, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self},
};

// Import the custom keywords from the parent module
use crate::kw_ops::kw;

/// HDL declaration kinds: wire, reg, etc.
#[derive(Debug, Clone, Hash, Copy, PartialEq, Serialize, Deserialize)]
pub enum HDLKind {
    /// Wire declaration
    Wire,
    /// Register declaration
    Reg,
}

impl HDLKind {
    /// Returns true if the kind is `Reg`.
    pub fn is_reg(&self) -> bool {
        matches!(self, HDLKind::Reg)
    }
    /// Returns true if the kind is `Wire`.
    pub fn is_wire(&self) -> bool {
        matches!(self, HDLKind::Wire)
    }
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

/// Direction of a port: input, output, or inout.
#[derive(Debug, Clone, Hash, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    /// Input port
    Input,
    /// Output port
    Output,
    /// Inout port
    Inout,
}

impl Direction {
    /// Returns true if the direction is `Input`.
    pub fn is_input(&self) -> bool {
        matches!(self, Direction::Input)
    }
    /// Returns true if the direction is `Output`.
    pub fn is_output(&self) -> bool {
        matches!(self, Direction::Output)
    }
    /// Returns true if the direction is `Inout`.
    pub fn is_inout(&self) -> bool {
        matches!(self, Direction::Inout)
    }
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
            let next: TokenTree = input.parse()?;
            log::error!(
                "Expected input, output, or inout, but got token: {:?}.  Remainer of input is {}",
                next,
                input
            );
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

/// A range of bits, specified by start and end indices.  Note that
/// start is the lower index and end is the higher index.
/// Note also that end is inclusive.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct BitRange {
    /// Start index of the bit range.
    pub start: u32,
    /// End index of the bit range.
    pub end: u32,
}

impl From<&std::ops::Range<usize>> for BitRange {
    fn from(r: &std::ops::Range<usize>) -> Self {
        BitRange {
            start: r.start as u32,
            end: (r.end as u32).saturating_sub(1),
        }
    }
}

impl Parse for BitRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let end = input.parse::<LitInt>()?;
        let _ = input.parse::<Token![:]>()?;
        let start = input.parse::<LitInt>()?;
        let start = start.base10_parse::<u32>()?;
        let end = end.base10_parse::<u32>()?;
        Ok(BitRange { start, end })
    }
}

impl Pretty for BitRange {
    fn pretty_print(&self, formatter: &mut crate::formatter::Formatter) {
        formatter.write(&self.end.to_string());
        formatter.write(":");
        formatter.write(&self.start.to_string());
    }
}

impl ToTokens for BitRange {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let start = syn::Index::from(self.start as usize);
        let end = syn::Index::from(self.end as usize);
        tokens.extend(quote! { #end : #start });
    }
}

/// A width specification for a signal, e.g., [7:0].
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct WidthSpec {
    /// The bit range specifying the width.
    pub bit_range: BitRange,
}

impl WidthSpec {
    /// Returns the length of the width specification.
    pub fn len(&self) -> usize {
        (self.bit_range.end - self.bit_range.start + 1) as usize
    }
    /// Returns true if the width specification is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

/// A signed or unsigned width specification.
///
/// In Verilog, this would be represented as either "signed [7:0]" or simply "[7:0]".
#[derive(Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum SignedWidth {
    /// Signed width specification.
    Signed(WidthSpec),
    /// Unsigned width specification.
    Unsigned(WidthSpec),
}

impl SignedWidth {
    /// Returns the length of the width specification.
    pub fn len(&self) -> usize {
        match self {
            SignedWidth::Signed(width) | SignedWidth::Unsigned(width) => width.len(),
        }
    }
    /// Returns true if the width specification is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

/// A declaration kind, consisting of a name and an optional width.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct DeclKind {
    /// The name of the declaration kind.
    pub name: String,
    /// The optional width specification.
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

/// A list of declarations of a given kind (wire, reg, etc.).
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct DeclarationList {
    /// The kind of declaration (e.g., wire, reg).
    pub kind: HDLKind,
    /// The optional signed width specification.
    pub signed_width: Option<SignedWidth>,
    /// The list of declaration items.
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

/// A single declaration, consisting of a kind, optional width, and name.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    /// The kind of declaration (e.g., wire, reg).
    pub kind: HDLKind,
    /// The optional signed width specification.
    pub signed_width: Option<SignedWidth>,
    /// The name of the declaration.
    pub name: String,
}

impl Declaration {
    /// Returns the width of the declaration.
    pub fn width(&self) -> usize {
        self.signed_width.as_ref().map(|w| w.len()).unwrap_or(1)
    }
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

/// A port declaration, consisting of a direction and a declaration.
/// In Verilog, this would be represented as e.g., `input wire [7:0] data_in`
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Port {
    /// The direction of the port (input, output, inout).
    pub direction: Direction,
    /// The declaration associated with the port.
    pub decl: Declaration,
}

impl Port {
    /// Returns the width of the port.
    pub fn width(&self) -> usize {
        self.decl.width()
    }
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

/// A Verilog literal, consisting of a width and a value.
/// In Verilog, this would be represented as e.g., `8'b10101010`.
/// The value is represented as a string to accommodate various
/// formats (binary, hexadecimal, decimal, etc.).
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub struct LitVerilog {
    /// The width of the literal.
    pub width: u32,
    /// The value of the literal as a string.
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

/// A sensitivity specification for always blocks.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum Sensitivity {
    /// A positive edge sensitivity (posedge).
    PosEdge(PosEdgeSensitivity),
    /// A negative edge sensitivity (negedge).
    NegEdge(NegEdgeSensitivity),
    /// A signal sensitivity.
    Signal(String),
    /// A star (*) sensitivity.
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

/// A positive edge sensitivity (posedge).
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct PosEdgeSensitivity {
    /// The identifier associated with the positive edge sensitivity.
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

/// A negative edge sensitivity (negedge).
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct NegEdgeSensitivity {
    /// The identifier associated with the negative edge sensitivity.
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

/// A sensitivity list for always blocks.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SensitivityList {
    /// The list of sensitivity elements.
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

/// A constant expression.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum ConstExpr {
    /// A Verilog literal.
    LitVerilog(LitVerilog),
    /// An integer literal.
    LitInt(i32),
    /// A string literal.
    LitStr(String),
    /// A real number literal.
    LitReal(String),
}

impl Parse for ConstExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.fork().parse::<LitVerilog>().is_ok() {
            Ok(ConstExpr::LitVerilog(input.parse()?))
        } else if input.fork().parse::<LitInt>().is_ok() {
            Ok(ConstExpr::LitInt(input.parse::<LitInt>()?.base10_parse()?))
        } else if input.fork().parse::<LitFloat>().is_ok() {
            Ok(ConstExpr::LitReal(input.parse::<LitFloat>()?.to_string()))
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
            ConstExpr::LitReal(r) => formatter.write(r),
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
            ConstExpr::LitReal(r) => {
                let r = LitFloat::new(r, proc_macro2::Span::call_site());
                tokens.extend(quote! { #r });
            }
        }
    }
}
