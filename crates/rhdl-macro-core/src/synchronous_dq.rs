use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, spanned::Spanned};

use crate::utils::{FieldSet, parse_dq_no_prefix_attribute};

pub fn derive_synchronous_dq(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_synchronous_dq_struct(decl)
}

fn derive_synchronous_dq_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let no_prefix = parse_dq_no_prefix_attribute(&decl.attrs);

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
    let q_fields = if !component_name.is_empty() {
        quote! { { #(#component_name: <#component_ty as rhdl::core::SynchronousIO>::O),*  }}
    } else {
        quote! {{} }
    };
    let d_fields = if !component_name.is_empty() {
        quote! { { #(#component_name: <#component_ty as rhdl::core::SynchronousIO>::I),*  }}
    } else {
        quote! {{}}
    };
    let q_name = if no_prefix {
        format_ident!("Q")
    } else {
        format_ident!("{}Q", struct_name)
    };
    let d_name = if no_prefix {
        format_ident!("D")
    } else {
        format_ident!("{}D", struct_name)
    };
    // Create a new struct by appending a Q to the name of the struct, and for each field, map
    // the type to <ty as rhdl::core::Synchronous>::O,
    let new_struct_q = quote! {
        #[derive(Digital, Clone, Copy, PartialEq)]
        #[doc(hidden)]
        pub struct #q_name #generics #where_clause #q_fields
    };
    let new_struct_d = quote! {
        #[derive(Digital, Clone, Copy, PartialEq)]
        #[doc(hidden)]
        pub struct #d_name #generics #where_clause #d_fields
    };

    Ok(quote! {
        #new_struct_q
        #new_struct_d

        impl #impl_generics rhdl::core::SynchronousDQ for #struct_name #ty_generics #where_clause {
            type Q = #q_name #ty_generics;
            type D = #d_name #ty_generics;
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use expect_test::expect_file;
    use quote::quote;

    #[test]
    fn test_synchronous_dq_derive_with_prefix() {
        // Check that SynchronousDQ derive generates prefixed struct names
        let decl = quote!(
            pub struct MySync {
                field_a: u8,
                field_b: u16,
            }
        );
        let output = derive_synchronous_dq(decl).unwrap().to_string();
        let expected = expect_file!["expect/template_synchronous_dq_derive_with_prefix.expect"];
        expected.assert_eq(&output);
    }

    #[test]
    fn test_synchronous_dq_derive_no_prefix() {
        // Check that #[rhdl(dq_no_prefix)] generates unprefixed struct names
        let decl = quote!(
            #[rhdl(dq_no_prefix)]
            pub struct MySync {
                field_a: u8,
                field_b: u16,
            }
        );
        let output = derive_synchronous_dq(decl).unwrap().to_string();
        let expected = expect_file!["expect/template_synchronous_dq_derive_no_prefix.expect"];
        expected.assert_eq(&output);
    }
}
