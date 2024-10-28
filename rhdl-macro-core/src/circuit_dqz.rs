use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

use crate::utils::FieldSet;

pub fn derive_circuit_dqz(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_circuit_dqz_struct(decl)
}

fn derive_circuit_dqz_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;

    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let Data::Struct(s) = &decl.data else {
        return Err(syn::Error::new(
            decl.span(),
            "CircuitDQ can only be derived for structs with named fields",
        ));
    };
    let field_set = FieldSet::try_from(&s.fields)?;
    let component_ty = &field_set.component_ty;
    let component_name = &field_set.component_name;
    let generics = &decl.generics;
    let new_struct_q = quote! {
        #[derive(Debug, Clone, PartialEq, Digital, Timed, Copy)]
        pub struct Q #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::CircuitIO>::O),*
        }
    };
    let new_struct_d = quote! {
        #[derive(Debug, Clone, PartialEq, Digital, Timed, Copy)]
        pub struct D #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::CircuitIO>::I),*
        }
    };
    let ty_z = quote! {
        (#(<#component_ty as rhdl::core::CircuitDQZ>::Z,)*)
    };
    Ok(quote! {
        #new_struct_q
        #new_struct_d

        impl #impl_generics rhdl::core::CircuitDQZ for #struct_name #ty_generics #where_clause {
            type Q = Q #ty_generics;
            type D = D #ty_generics;
            type Z = #ty_z;
        }
    })
}
