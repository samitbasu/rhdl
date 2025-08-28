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

use crate::{
    atoms::LitVerilog,
    expr::{Expr, ExprConcat, ExprDynIndex, ExprIndex},
    formatter::{Formatter, Pretty},
    kw_ops::{BinaryOp, DynOp, UnaryOp},
    stmt::Stmt,
};
use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::{
    Ident, Result, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Paren},
};

use kw_ops::kw;

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
