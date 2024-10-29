use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

use crate::{notable_enum::derive_notable_enum, utils::parse_rhdl_skip_attribute};

pub fn derive_notable(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_notable_struct(decl),
        Data::Enum(_e) => derive_notable_enum(decl),
        _ => Err(syn::Error::new(
            decl.span(),
            "Notable can only be derived for structs or enums",
        )),
    }
}

fn derive_notable_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    match &decl.data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(_) => derive_notable_named_struct(decl),
            syn::Fields::Unnamed(_) => derive_notable_tuple_struct(decl),
            syn::Fields::Unit => Err(syn::Error::new(
                s.fields.span(),
                "Unit structs are not digital",
            )),
        },
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}

fn derive_notable_tuple_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .filter(|f| !parse_rhdl_skip_attribute(&f.attrs))
                .enumerate()
                .map(|(ndx, _)| syn::Index::from(ndx))
                .collect::<Vec<_>>();
            Ok(quote! {
                impl #impl_generics rhdl::core::Notable for #struct_name #ty_generics #where_clause {
                    fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                        #(
                            rhdl::core::Notable::note(&self.#fields, (key, stringify!(#fields)), &mut writer);
                        )*
                    }
                }
            })
        }
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}

fn derive_notable_named_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .filter(|f| !parse_rhdl_skip_attribute(&f.attrs))
                .map(|field| &field.ident)
                .collect::<Vec<_>>();
            Ok(quote! {
                impl #impl_generics rhdl::core::Notable for #struct_name #ty_generics #where_clause {
                    fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                        #(
                            rhdl::core::Notable::note(&self.#fields, (key, stringify!(#fields)), &mut writer);
                        )*
                    }
                }
            })
        }
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}
