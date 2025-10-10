use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

use crate::utils::FieldSet;

pub fn derive_circuit_dq(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_circuit_dq_struct(decl)
}

fn derive_circuit_dq_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
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
        #[derive(PartialEq, Digital, Clone, Timed)]
        #[doc(hidden)]
        pub struct Q #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::CircuitIO>::O),*
        }
    };
    let new_struct_d = quote! {
        #[derive(PartialEq, Digital, Clone, Timed)]
        #[doc(hidden)]
        pub struct D #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::CircuitIO>::I),*
        }
    };
    Ok(quote! {
        #new_struct_q
        #new_struct_d

        impl #impl_generics rhdl::core::CircuitDQ for #struct_name #ty_generics #where_clause {
            type Q = Q #ty_generics;
            type D = D #ty_generics;
        }
    })
}
