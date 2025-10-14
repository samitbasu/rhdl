use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

pub fn derive_timed(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_) => derive_timed_struct(decl),
        _ => Err(syn::Error::new_spanned(
            decl,
            "can only derive `Timed` for structs",
        )),
    }
}

fn derive_timed_struct(decl: syn::DeriveInput) -> syn::Result<TokenStream> {
    match &decl.data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(_) => derive_timed_named_struct(decl),
            _ => Err(syn::Error::new_spanned(
                decl,
                "can only derive `Timed` for structs with named fields",
            )),
        },
        _ => unreachable!(),
    }
}

fn derive_timed_named_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let field_where_clauses: Vec<syn::WherePredicate> = s
                .fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    syn::parse_quote! {
                        #ty: rhdl::core::Timed
                    }
                })
                .collect();
            let where_clause = if let Some(mut wc) = where_clause.cloned() {
                wc.predicates.extend(field_where_clauses);
                wc
            } else {
                syn::WhereClause {
                    where_token: Default::default(),
                    predicates: field_where_clauses.into_iter().collect(),
                }
            };
            Ok(quote! {
                impl #impl_generics rhdl::core::Timed for #struct_name #ty_generics #where_clause {}
            })
        }
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timed_with_generics() {
        let t = quote! {
            pub struct In<W: Domain, R: Domain> {
                /// The data signal (comes from the input clock domain)
                pub data: Signal<bool, W>,
                /// The clock and reset signal from the output clock domain
                pub cr: Signal<ClockReset, R>,
            }
        };
        let derived = super::derive_timed(t).unwrap();
        let expected = expect_test::expect![
            "impl < W : Domain , R : Domain > rhdl :: core :: Timed for In < W , R > where Signal < bool , W > : rhdl :: core :: Timed , Signal < ClockReset , R > : rhdl :: core :: Timed { }"
        ];
        expected.assert_eq(&derived.to_string());
    }
}
