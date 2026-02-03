/*
    The path! macro takes a full path expression and converts it into
    a Path type.  So for example:

    path!(input.axi.val().clock.0)

    Translates into:

    {
        _ = input.axi.val().clock.0; // Ensures the expression is valid
        Path::default()
            .field("axi")
            .signal_value()
            .field("clock")
            .tuple_index(0)
    }

    The root identifier (e.g., `input`) is stripped and the expression
    is validated at compile time, while the path segments are used to
    build the Path.
*/

use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::bracketed;
use syn::parenthesized;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::token;

#[derive(Debug)]
enum PathSegment {
    Default,
    Field(FieldName),
    TupleIndex(Literal),
    SignalVal,
    ArrayIndex(Literal),
}

impl ToTokens for PathSegment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PathSegment::Default => {
                quote! {    rhdl::core::types::path::Path::default() }.to_tokens(tokens);
            }
            PathSegment::Field(fieldname) => {
                quote! {field(#fieldname)}.to_tokens(tokens);
            }
            PathSegment::TupleIndex(ndx) => {
                quote! {tuple_index(#ndx as usize)}.to_tokens(tokens);
            }
            PathSegment::SignalVal => {
                quote! {signal_value()}.to_tokens(tokens);
            }
            PathSegment::ArrayIndex(ndx) => {
                quote! {index(#ndx as usize)}.to_tokens(tokens);
            }
        }
    }
}

#[derive(Debug)]
enum FieldName {
    Ident(Ident),
    ValKeyWord,
}

impl ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldName::Ident(ident) => {
                quote! {stringify!(#ident)}.to_tokens(tokens);
            }
            FieldName::ValKeyWord => {
                let ident = format_ident!("val");
                quote! {stringify!(#ident)}.to_tokens(tokens);
            }
        }
    }
}

mod kw {
    syn::custom_keyword!(val);
}

#[derive(Debug)]
struct PathExpression {
    full_expr: TokenStream,
    elements: Vec<PathSegment>,
}

impl ToTokens for PathExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let elements = &self.elements;
        let full_expr = &self.full_expr;
        quote! {
            {
                _ = #full_expr;
                #(#elements).*
            }
        }
        .to_tokens(tokens);
    }
}

impl Parse for PathExpression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Store the full input for validation
        let full_expr = input.parse::<proc_macro2::TokenStream>()?;

        // Re-parse to extract the path segments
        let mut iter = full_expr.clone().into_iter().peekable();

        // Skip the root identifier (e.g., "input" or "output")
        if let Some(proc_macro2::TokenTree::Ident(_)) = iter.peek() {
            iter.next();
        }

        // Parse remaining path segments
        let remaining: TokenStream = iter.collect();
        let elements = syn::parse2::<PathSequence>(remaining)?.0;

        Ok(Self {
            full_expr,
            elements,
        })
    }
}

struct PathSequence(Vec<PathSegment>);

impl Parse for PathSequence {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ret = vec![PathSegment::Default];
        loop {
            if input.is_empty() {
                break;
            }
            let lookahead = input.lookahead1();
            if lookahead.peek(token::Bracket) {
                let content;
                let _bracket = bracketed!(content in input);
                ret.push(PathSegment::ArrayIndex(content.parse()?));
            } else if lookahead.peek(token::Dot) {
                let _dot: token::Dot = input.parse()?;
                if input.peek(kw::val) {
                    let _val: kw::val = input.parse()?;
                    if input.peek(token::Paren) {
                        let _content;
                        let _ = parenthesized!(_content in input);
                        ret.push(PathSegment::SignalVal);
                    } else {
                        ret.push(PathSegment::Field(FieldName::ValKeyWord));
                    }
                } else if input.peek(Ident) {
                    ret.push(PathSegment::Field(FieldName::Ident(input.parse()?)));
                } else {
                    ret.push(PathSegment::TupleIndex(input.parse()?));
                }
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(PathSequence(ret))
    }
}

pub fn path_macro(input: TokenStream) -> syn::Result<TokenStream> {
    let fields: PathExpression = syn::parse2(input)?;
    Ok(quote! { #fields})
}

#[cfg(test)]
mod test {
    use super::*;
    use expect_test::expect_file;

    #[test]
    fn test_custom_parser() -> syn::Result<()> {
        let tokens = quote!(input[3]);
        let paths1: PathExpression = syn::parse2(tokens)?;
        let tokens = quote!(input[3].foo);
        let paths2: PathExpression = syn::parse2(tokens)?;
        let tokens = quote!(input[3].foo.val().4.val);
        let paths3: PathExpression = syn::parse2(tokens)?;
        let expect = expect_file!["expect/custom_path_parser.expect"];
        expect.assert_debug_eq(&(paths1, paths2, paths3));
        Ok(())
    }

    #[test]
    fn test_path_macro() -> syn::Result<()> {
        let tokens = quote!(input[3].foo.val().4.val);
        let macro_out = path_macro(tokens)?.to_string();
        let expect = expect_file!["expect/path_macro.expect"];
        expect.assert_eq(&macro_out);
        Ok(())
    }
}
