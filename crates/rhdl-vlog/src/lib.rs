#![warn(missing_docs)]
//! A Verilog HDL Abstract Syntax Tree (AST) library for parsing, representing, and generating Verilog code.
//! It provides structures and functions to create, manipulate, and serialize Verilog modules,
//! including support for synthesis attributes and documentation comments.
//!
//! The library is designed to be used in conjunction with the `rhdl` crate to generate
//! Verilog code from Rust code.  It is not intended to be a full Verilog parser or generator.
//!
//! This library works using the `syn` crate for parsing and the `quote` crate for code generation.
//! It also uses `serde` for serialization and deserialization of the AST structures.
//!
//! It can be used with `quote!` macros to generate Verilog code from Rust code.
//!
//! # Example
//!
//! ```rust
//! use rhdl_vlog::*;
//! use quote::quote;
//!
//! let module: ModuleDef = parse_quote! {
//!    module my_module (input wire a, input wire b, output wire c);
//!    endmodule
//! };
//! ```
//!

pub mod atoms;
pub mod builder;
pub mod expr;
pub mod formatter;
pub mod kw_ops;
pub mod stmt;

// Re-export builder functions for convenient access
pub use builder::*;

// Re-export atomic AST types for convenient access
pub use atoms::{
    BitRange, DeclKind, Declaration, DeclarationList, Direction, HDLKind, Port, SignedWidth,
    WidthSpec,
};
use thiserror::Error;

use crate::atoms::ConstExpr;
pub use crate::{
    atoms::LitVerilog,
    expr::{Expr, ExprConcat, ExprDynIndex, ExprIndex},
    formatter::{Formatter, Pretty},
    kw_ops::{BinaryOp, DynOp, UnaryOp},
    stmt::Stmt,
};
use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::{
    Ident, Result, Token, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Paren},
};

use kw_ops::kw;

/// A synthesis attribute in Verilog HDL.

#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SynthesisAttribute {
    /// The name of the synthesis attribute.
    pub name: String,
    /// The value of the synthesis attribute.
    pub value: ConstExpr,
}

impl Parse for SynthesisAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let value: ConstExpr = input.parse()?;
        Ok(SynthesisAttribute {
            name: name.to_string(),
            value,
        })
    }
}

impl ToTokens for SynthesisAttribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.name);
        let value = &self.value;
        tokens.extend(quote! { #name = #value });
    }
}

impl Pretty for SynthesisAttribute {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.write(&format!("{} = ", self.name));
        self.value.pretty_print(formatter);
    }
}

/// A list of synthesis attributes in Verilog HDL.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SynthesisAttributeList {
    /// The list of synthesis attributes.
    pub attributes: Vec<SynthesisAttribute>,
}

impl ToTokens for SynthesisAttributeList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let attrs = &self.attributes;
        tokens.extend(quote! { (* #( #attrs ),* *) });
    }
}

/// A Verilog HDL attribute, either a documentation string or synthesis attributes.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum Attribute {
    /// A documentation string attribute.
    Doc(String),
    /// A synthesis attribute list.
    Synthesis(SynthesisAttributeList),
}

fn parse_doc_string(input: ParseStream) -> Result<String> {
    let _ = input.parse::<Token![#]>()?;
    let content;
    let _paren = bracketed!(content in input);
    let _doc_kw: kw::doc = content.parse()?;
    let _eq: Token![=] = content.parse()?;
    let lit_str: syn::LitStr = content.parse()?;
    Ok(lit_str.value())
}

fn parse_synthesis_attribute(input: ParseStream) -> Result<SynthesisAttributeList> {
    let content;
    let _paren = parenthesized!(content in input);
    let _star = content.parse::<Token![*]>()?;
    let mut attrs = Vec::new();
    while !content.peek(Token![*]) {
        let attr: SynthesisAttribute = content.parse()?;
        if content.peek(Token![,]) {
            let _comma = content.parse::<Token![,]>()?;
        }
        attrs.push(attr);
    }
    let _star = content.parse::<Token![*]>()?;
    Ok(SynthesisAttributeList { attributes: attrs })
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        if parse_synthesis_attribute(&input.fork()).is_ok() {
            let synthesis_attrs = parse_synthesis_attribute(input)?;
            Ok(Attribute::Synthesis(synthesis_attrs))
        } else if input.peek(Token![#]) && input.peek2(token::Bracket) {
            let doc_string = parse_doc_string(input)?;
            Ok(Attribute::Doc(doc_string))
        } else {
            Err(input.error("expected attribute"))
        }
    }
}

impl Pretty for Attribute {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            Attribute::Doc(doc) => {
                formatter.write(&format!("// {doc}"));
                formatter.newline();
            }
            Attribute::Synthesis(synth_attrs) => {
                formatter.write("(* ");
                formatter.comma_separated(&synth_attrs.attributes);
                formatter.write(" *) ");
            }
        }
    }
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Attribute::Doc(doc) => {
                tokens.extend(quote! { #[doc = #doc] });
            }
            Attribute::Synthesis(synth_attrs) => synth_attrs.to_tokens(tokens),
        }
    }
}

/// An item in Verilog HDL, such as a statement, declaration, or function definition.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// The list of attributes associated with the item.
    pub attributes: Vec<Attribute>,
    /// The kind of item.
    pub kind: ItemKind,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attributes = Vec::new();
        while input.fork().parse::<Attribute>().is_ok() {
            attributes.push(input.parse()?);
        }
        let kind = input.parse()?;
        Ok(Item { attributes, kind })
    }
}

impl Pretty for Item {
    fn pretty_print(&self, formatter: &mut Formatter) {
        for attr in &self.attributes {
            attr.pretty_print(formatter);
        }
        self.kind.pretty_print(formatter);
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in &self.attributes {
            attr.to_tokens(tokens);
        }
        self.kind.to_tokens(tokens);
    }
}

/// The kind of item in Verilog HDL.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    /// A statement item.
    Statement(Stmt),
    /// A declaration item.
    Declaration(DeclarationList),
    /// A function definition item.
    FunctionDef(FunctionDef),
    /// An initial block item.
    Initial(Initial),
}

impl Parse for ItemKind {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::function) {
            input.parse().map(ItemKind::FunctionDef)
        } else if input.peek(kw::reg) || input.peek(kw::wire) {
            input.parse().map(ItemKind::Declaration)
        } else if input.peek(kw::initial) {
            input.parse().map(ItemKind::Initial)
        } else {
            input.parse().map(ItemKind::Statement)
        }
    }
}

impl Pretty for ItemKind {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            ItemKind::Statement(stmt) => stmt.pretty_print(formatter),
            ItemKind::Declaration(decl) => decl.pretty_print(formatter),
            ItemKind::FunctionDef(func) => func.pretty_print(formatter),
            ItemKind::Initial(initial) => initial.pretty_print(formatter),
        }
    }
}

impl ToTokens for ItemKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ItemKind::Statement(stmt) => stmt.to_tokens(tokens),
            ItemKind::Declaration(decl) => decl.to_tokens(tokens),
            ItemKind::FunctionDef(func) => func.to_tokens(tokens),
            ItemKind::Initial(initial) => initial.to_tokens(tokens),
        }
    }
}

/// A list of items in Verilog HDL.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize, Default)]
pub struct ItemList {
    /// The list of items.
    pub items: Vec<Item>,
}

impl Parse for ItemList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}

impl ToTokens for ItemList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let items = &self.items;
        tokens.extend(quote! { #( #items )* });
    }
}

impl Pretty for ItemList {
    fn pretty_print(&self, formatter: &mut Formatter) {
        formatter.lines(&self.items);
    }
}

/// An initial block in Verilog HDL.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Initial {
    /// The statement inside the initial block.
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

/// A list of Verilog HDL modules.
#[derive(Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModuleList {
    /// The list of modules.
    pub modules: Vec<ModuleDef>,
}

impl ModuleList {
    /// Check the module list for syntactic correctness using Icarus Verilog.
    pub fn checked(&self) -> anyhow::Result<()> {
        let d = tempfile::tempdir()?;
        // Write the test bench to a file
        let d_path = d.path();
        std::fs::write(d_path.join("top.v"), self.to_string())?;
        // Compile the test bench
        let mut cmd = std::process::Command::new("iverilog");
        cmd.arg("-t").arg("null").arg(d_path.join("top.v"));
        let status = cmd
            .status()
            .expect("Icarus Verilog should be installed and in your PATH.");
        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to compile testbench with {}",
                status
            ));
        }
        Ok(())
    }
}

impl From<ModuleDef> for ModuleList {
    fn from(module: ModuleDef) -> Self {
        Self {
            modules: vec![module],
        }
    }
}

impl IntoIterator for ModuleList {
    type Item = ModuleDef;
    type IntoIter = std::vec::IntoIter<ModuleDef>;
    fn into_iter(self) -> Self::IntoIter {
        self.modules.into_iter()
    }
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

impl std::fmt::Debug for ModuleList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = crate::formatter::Formatter::new();
        self.pretty_print(&mut fmt);
        write!(f, "{}", fmt.finish())
    }
}

/// A Verilog HDL module definition.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct ModuleDef {
    /// The name of the module.    
    pub name: String,
    /// The ports of the module.
    pub args: Vec<Port>,
    /// The items inside the module.
    pub items: Vec<Item>,
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

/// A Verilog HDL function definition.
#[derive(Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    /// The signed width of the function's return value.
    pub signed_width: SignedWidth,
    /// The name of the function.
    pub name: String,
    /// The ports (arguments) of the function.
    pub args: Vec<Port>,
    /// The items inside the function.
    pub items: Vec<Item>,
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
        formatter.write("function ");
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

#[derive(Debug, Error)]
#[error("Parse Error in Verilog code")]
/// A parse error that includes source code context for better error reporting.
pub struct ParseError {
    source_code: std::sync::Arc<String>,
    syn_error: syn::Error,
}

impl ParseError {
    /// Create a new ParseError with the given syn::Error and source code.
    pub fn new(syn_err: syn::Error, source_code: &str) -> Self {
        Self {
            source_code: std::sync::Arc::new(source_code.into()),
            syn_error: syn_err,
        }
    }
}

/// Format Verilog code for better error display by adding line breaks
/// without requiring the code to be syntactically valid.
pub fn format_verilog_for_error_display(code: &str) -> String {
    // First, add structural breaks at common Verilog delimiters
    code.replace(" ; ", " ;\n")
        .replace(" { ", " {\n")
        .replace(" } ", "\n}\n")
        .replace(") ;", ") ;\n")
        .replace(" wire ", "\n    wire ")
        .replace(" reg ", "\n    reg ")
        .replace(" assign ", "\n    assign ")
        .replace(" function ", "\nfunction ")
        .replace(" endfunction", "\nendfunction")
        .replace(" endmodule", "\nendmodule")
        .replace(" begin", "\n        begin")
        .replace(" end ", "\n        end\n")
        .replace(" localparam ", "\n        localparam ")
}

impl miette::Diagnostic for ParseError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source_code)
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new((&self.syn_error).into_iter().map(
            move |syn_error| {
                let span = syn_error.span();
                let span_start = span.start();
                let span_end = span.end();
                let start_offset = miette::SourceOffset::from_location(
                    self.source_code.as_str(),
                    span_start.line,
                    span_start.column + 1,
                );
                let end_offset = miette::SourceOffset::from_location(
                    self.source_code.as_str(),
                    span_end.line,
                    span_end.column + 1,
                );
                let length = end_offset.offset() - start_offset.offset();
                miette::LabeledSpan::new_with_span(
                    Some(syn_error.to_string()),
                    miette::SourceSpan::new(start_offset, length),
                )
            },
        )))
    }
}

#[macro_export]
/// A macro to parse Verilog code with enhanced error reporting.
macro_rules! parse_quote_miette {
    ($($tt:tt)*) => {{
        let tokens = ::quote::quote! { $($tt)* };
        let text = $crate::format_verilog_for_error_display(&tokens.to_string());
        ::syn::parse_str(&text).map_err(|err| $crate::ParseError::new(err, text.as_str()))
    }};
}

#[cfg(test)]
mod tests;
