/*
    We want:

    export!(
        input aclk => i.clock,
        input aresetn => i.reset_n
        input data => i.axi.val().data
        output foo => o.foo
    )

    to translate into

    [
        Map::Input("aclk", i.kind(), Path::default().field("clock")),
        Map::Input("aresetn", i.kind(), Path::default().field("reset_n")),
        Map::Input("data", i.kind(), Path::default().field("axi").signal_val().field("data")),
        Map::Output("foo", o.kind(), Path::default().field("foo"))
    ]

*/

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Member, Result, Token,
};

enum MapDirection {
    Input,
    Output,
}

struct MapLine {
    direction: MapDirection,
    name: Ident,
    var: Expr,
}

struct Export {
    lines: Punctuated<MapLine, Token![,]>,
}

impl Parse for MapDirection {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "input" {
            Ok(MapDirection::Input)
        } else if ident == "output" {
            Ok(MapDirection::Output)
        } else {
            Err(syn::Error::new(
                ident.span(),
                format!("Expected input or output instead of {ident:?}"),
            ))
        }
    }
}

impl Parse for MapLine {
    fn parse(input: ParseStream) -> Result<Self> {
        let direction: MapDirection = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<syn::Token![=>]>()?;
        let var: Expr = input.parse()?;
        Ok(MapLine {
            direction,
            name,
            var,
        })
    }
}

impl Parse for Export {
    fn parse(input: ParseStream) -> Result<Self> {
        let lines = Punctuated::parse_terminated(input)?;
        Ok(Export { lines })
    }
}

#[derive(Debug, Clone)]
enum PathSegment {
    Field(Ident),
    TupleIndex(u32),
    SignalVal,
}

impl ToTokens for PathSegment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PathSegment::Field(ident) => {
                quote![field(stringify!(#ident))].to_tokens(tokens);
            }
            PathSegment::TupleIndex(index) => {
                quote![tuple_index(#index as usize)].to_tokens(tokens);
            }
            PathSegment::SignalVal => {
                quote![signal_value()].to_tokens(tokens);
            }
        }
    }
}

fn convert_expr_to_path(expr: &Expr) -> Vec<PathSegment> {
    let mut result = vec![];
    match expr {
        Expr::Path(path) => {
            for segment in &path.path.segments {
                result.push(PathSegment::Field(segment.ident.clone()));
            }
        }
        Expr::Field(field) => {
            result = convert_expr_to_path(&field.base);
            match &field.member {
                Member::Named(ident) => {
                    result.push(PathSegment::Field(ident.clone()));
                }
                Member::Unnamed(index) => {
                    result.push(PathSegment::TupleIndex(index.index));
                }
            }
        }
        Expr::MethodCall(method) => {
            result = convert_expr_to_path(&method.receiver);
            if method.method == "val" {
                result.push(PathSegment::SignalVal);
            } else {
                panic!("Unexpected method call {method:?}");
            }
        }
        _ => panic!("Unexpected expression {expr:?}"),
    }
    result
}

pub fn export_macro(input: TokenStream) -> syn::Result<TokenStream> {
    let Export { lines } = syn::parse2(input)?;
    let mut result = Vec::new();
    for line in lines {
        let direction = match line.direction {
            MapDirection::Input => format_ident!("Input"),
            MapDirection::Output => format_ident!("Output"),
        };
        let name = line.name;
        let var = line.var;
        let path = convert_expr_to_path(&var);
        let (first, rest) = path.split_first().unwrap();
        let PathSegment::Field(base) = first else {
            panic!("Expected a variable as the first segment of the path");
        };
        let fields: Punctuated<PathSegment, Token![.]> = rest.iter().cloned().collect();
        result.push(quote::quote! {
            (rhdl::prelude::Direction::#direction, stringify!(#name), #base.kind(), Path::default().#fields)
        });
    }
    Ok(quote::quote! {
        [
            #(#result),*
        ]
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use expect_test::expect_file;
    use quote::quote;

    #[test]
    fn test_export_macro() {
        let decls = quote!(
            input aclk => i.clock,
            input aresetn => i.reset_n,
            input data => i.axi.val().data.1,
            output foo => o.foo
        );
        let result = export_macro(decls).unwrap();
        let expected = expect_file!["expect/export_macro.expect"];
        expected.assert_eq(&result.to_string());
    }
}
