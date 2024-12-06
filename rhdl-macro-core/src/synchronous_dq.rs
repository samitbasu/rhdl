use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

use crate::utils::FieldSet;

pub fn derive_synchronous_dq(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_synchronous_dq_struct(decl)
}

fn derive_synchronous_dq_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;

    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let Data::Struct(s) = &decl.data else {
        return Err(syn::Error::new(
            decl.span(),
            "Synchronous can only be derived for structs with named fields",
        ));
    };
    let field_set = FieldSet::try_from(&s.fields)?;
    let component_ty = &field_set.component_ty;
    let component_name = &field_set.component_name;
    let generics = &decl.generics;
    // Create a new struct by appending a Q to the name of the struct, and for each field, map
    // the type to <ty as rhdl::core::Synchronous>::O,
    let new_struct_q = quote! {
        #[derive(Digital)]
        pub struct Q #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::SynchronousIO>::O),*
        }
    };
    let new_struct_d = quote! {
        #[derive(Digital)]
        pub struct D #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::SynchronousIO>::I),*
        }
    };

    Ok(quote! {
        #new_struct_q
        #new_struct_d

        impl #impl_generics rhdl::core::SynchronousDQ for #struct_name #ty_generics #where_clause {
            type Q = Q #ty_generics;
            type D = D #ty_generics;
        }
    })
}
