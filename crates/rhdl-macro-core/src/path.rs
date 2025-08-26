/*
    The path! macro takes a path expression and converts it into
    a Path type.  So for example:

    path!(x.axi.val().clock.0)

    Translates into:

    (x.kind,
        Path::default()
            .field("axi")
            .signal_val()
            .field("clock")
            .tuple_index(0)
    )

    Unfortunately, in use, it's kind of clunky.  It is cleaner to
    do something like:

    path!([3].axi.val().clock.0)
*/

use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::bracketed;
use syn::parenthesized;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::token;
use syn::Ident;

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
                quote! {    rhdl::rhdl_core::types::path::Path::default() }.to_tokens(tokens);
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
    elements: Vec<PathSegment>,
}

impl ToTokens for PathExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let elements = &self.elements;
        quote! { #(#elements).* }.to_tokens(tokens);
    }
}

// Change to accept
//  input.foo[3].val()...
//  output.baz.val()...

impl Parse for PathExpression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let elements = parse_path_sequence(input)?;
        Ok(Self { elements })
    }
}

fn parse_path_sequence(input: ParseStream) -> syn::Result<Vec<PathSegment>> {
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
    Ok(ret)
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
        let tokens = quote!([3]);
        let paths1: PathExpression = syn::parse2(tokens)?;
        let tokens = quote!([3].foo);
        let paths2: PathExpression = syn::parse2(tokens)?;
        let tokens = quote!([3].foo.val().4.val);
        let paths3: PathExpression = syn::parse2(tokens)?;
        let expect = expect_file!["expect/custom_path_parser.expect"];
        expect.assert_debug_eq(&(paths1, paths2, paths3));
        Ok(())
    }

    #[test]
    fn test_path_macro() -> syn::Result<()> {
        let tokens = quote!([3].foo.val().4.val);
        let macro_out = path_macro(tokens)?.to_string();
        let expect = expect_file!["expect/path_macro.expect"];
        expect.assert_eq(&macro_out);
        Ok(())
    }
}
