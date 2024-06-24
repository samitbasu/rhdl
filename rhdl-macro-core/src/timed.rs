use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

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
    let fqdn = crate::utils::get_fqdn(&decl);
    match decl.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .map(|field| &field.ident)
                .collect::<Vec<_>>();
            let field_types = s.fields.iter().map(|x| &x.ty).collect::<Vec<_>>();
            Ok(quote! {
                impl #impl_generics rhdl_core::Timed for #struct_name #ty_generics #where_clause {
                    fn static_kind() -> rhdl_core::Kind {
                        rhdl_core::Kind::make_struct(
                            #fqdn,
                            vec![
                            #(
                                rhdl_core::Kind::make_field(stringify!(#fields), <#field_types as rhdl_core::Timed>::static_kind()),
                            )*
                        ],
                    )
                    }
                }
            })
        }
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}
