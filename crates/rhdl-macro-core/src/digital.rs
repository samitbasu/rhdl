use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

use crate::{
    clone::derive_clone_from_inner, digital_enum::derive_digital_enum,
    utils::parse_rhdl_skip_attribute,
};

pub fn derive_digital(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_digital_struct(decl),
        Data::Enum(_e) => derive_digital_enum(decl),
        _ => Err(syn::Error::new(
            decl.span(),
            "Only structs and enums can be digital",
        )),
    }
}

fn derive_digital_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    match &decl.data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(_) => derive_digital_named_struct(decl),
            syn::Fields::Unnamed(_) => derive_digital_tuple_struct(decl),
            syn::Fields::Unit => Err(syn::Error::new(
                s.fields.span(),
                "Unit structs are not digital",
            )),
        },
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}

//  Add the module path to the name

fn derive_digital_tuple_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let fqdn = crate::utils::get_fqdn(&decl);
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let clone = derive_clone_from_inner(decl.clone())?;
    //let partial_eq = derive_partial_eq_from_inner(decl.clone())?;
    match decl.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .filter(|f| !parse_rhdl_skip_attribute(&f.attrs))
                .enumerate()
                .map(|(ndx, _)| syn::Index::from(ndx))
                .collect::<Vec<_>>();
            let field_types = s
                .fields
                .iter()
                .filter(|f| !parse_rhdl_skip_attribute(&f.attrs))
                .map(|x| &x.ty)
                .collect::<Vec<_>>();
            Ok(quote! {
                impl #impl_generics core::marker::Copy for #struct_name #ty_generics #where_clause {}

                #clone

                impl #impl_generics rhdl::core::Digital for #struct_name #ty_generics #where_clause {
                    const BITS: usize = #(
                        <#field_types as rhdl::core::Digital>::BITS
                    )+*;
                    const TRACE_BITS: usize = #(
                        <#field_types as rhdl::core::Digital>::TRACE_BITS
                    )+*;
                    fn static_kind() -> rhdl::core::Kind {
                        rhdl::core::Kind::make_struct(
                            #fqdn,
                            vec![
                            #(
                                rhdl::core::Kind::make_field(stringify!(#fields), <#field_types as rhdl::core::Digital>::static_kind()),
                            )*
                            ]
                        )
                    }
                    fn static_trace_type() -> rhdl::rtt::TraceType {
                        rhdl::rtt::make_struct(
                            #fqdn,
                            vec![
                            #(
                                rhdl::rtt::make_field(stringify!(#fields), <#field_types as rhdl::core::Digital>::static_trace_type()),
                            )*
                        ]
                    )
                    }
                    fn bin(self) -> Vec<rhdl::core::BitX> {
                        [
                        #(
                            self.#fields.bin().as_slice(),
                        )*
                        ].concat()
                    }
                    fn trace(self) -> Vec<rhdl::core::TraceBit> {
                        [
                        #(
                            self.#fields.trace().as_slice(),
                        )*
                        ].concat()
                    }
                    fn dont_care() -> Self {
                        Self(
                            #(
                                <#field_types as rhdl::core::Digital>::dont_care(),
                            )*
                        )
                    }
                }
                impl #impl_generics rhdl::core::DigitalFn for #struct_name #ty_generics #where_clause {
                    fn kernel_fn() -> Option<rhdl::core::KernelFnKind> {
                        Some(rhdl::core::KernelFnKind::TupleStructConstructor(<Self as rhdl::core::Digital>::static_kind().place_holder()))
                    }
                }
            })
        }
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}

fn derive_digital_named_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let clone = derive_clone_from_inner(decl.clone())?;
    let fqdn = crate::utils::get_fqdn(&decl);
    match decl.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .filter(|f| !parse_rhdl_skip_attribute(&f.attrs))
                .map(|field| &field.ident)
                .collect::<Vec<_>>();
            let field_types = s
                .fields
                .iter()
                .filter(|f| !parse_rhdl_skip_attribute(&f.attrs))
                .map(|x| &x.ty)
                .collect::<Vec<_>>();
            Ok(quote! {
                impl #impl_generics core::marker::Copy for #struct_name #ty_generics #where_clause {}

                #clone

                impl #impl_generics rhdl::core::Digital for #struct_name #ty_generics #where_clause {
                    const BITS: usize = #(
                        <#field_types as rhdl::core::Digital>::BITS
                    )+*;
                    const TRACE_BITS: usize = #(
                        <#field_types as rhdl::core::Digital>::TRACE_BITS
                    )+*;
                    fn static_kind() -> rhdl::core::Kind {
                        rhdl::core::Kind::make_struct(
                            #fqdn,
                            vec![
                            #(
                                rhdl::core::Kind::make_field(stringify!(#fields), <#field_types as rhdl::core::Digital>::static_kind()),
                            )*
                        ],
                        )
                    }
                    fn static_trace_type() -> rhdl::core::TraceType {
                        rhdl::rtt::make_struct(
                            #fqdn,
                            vec![
                            #(
                                rhdl::rtt::make_field(stringify!(#fields), <#field_types as rhdl::core::Digital>::static_trace_type()),
                            )*
                        ],
                        )
                    }
                    fn bin(self) -> Vec<rhdl::core::BitX> {
                        [
                        #(
                            self.#fields.bin().as_slice(),
                        )*
                        ].concat()
                    }
                    fn trace(self) -> Vec<rhdl::core::TraceBit> {
                        [
                        #(
                            self.#fields.trace().as_slice(),
                        )*
                        ].concat()
                    }
                    fn dont_care() -> Self {
                        Self {
                            #(
                                #fields: <#field_types as rhdl::core::Digital>::dont_care(),
                            )*
                        }
                    }
                }
            })
        }
        _ => Err(syn::Error::new(decl.span(), "Only structs can be digital")),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use expect_test::expect_file;

    #[test]
    fn test_digital_proc_macro() {
        let decl = quote!(
            pub struct NestedBits {
                nest_1: bool,
                nest_2: u8,
                nest_3: TwoBits,
            }
        );
        let output = derive_digital(decl).unwrap().to_string();
        let expected = expect_file!["expect/digital_proc_macro.expect"];
        expected.assert_debug_eq(&output);
    }

    #[test]
    fn test_digital_with_struct() {
        let decl = quote!(
            pub struct Inputs {
                pub input: u32,
                pub write: bool,
                pub read: bool,
            }
        );
        let output = derive_digital(decl).unwrap().to_string();
        let expected = expect_file!["expect/digital_with_struct.expect"];
        expected.assert_eq(&output);
    }

    #[test]
    fn test_struct_with_generics() {
        let decl = quote!(
            pub struct Inputs<T: Digital> {
                pub input: T,
                pub write: bool,
                pub read: bool,
            }
        );
        let output = derive_digital(decl).unwrap().to_string();
        let expected = expect_file!["expect/struct_with_generics.expect"];
        expected.assert_eq(&output);
    }

    #[test]
    fn test_struct_with_tuple_field() {
        let decl = quote!(
            pub struct Inputs {
                pub input: u32,
                pub write: bool,
                pub read: (bool, bool),
            }
        );
        let output = derive_digital(decl).unwrap().to_string();
        let expected = expect_file!["expect/struct_with_tuple_field.expect"];
        expected.assert_eq(&output);
    }

    #[test]
    fn test_digital_with_tuple_struct() {
        let decl = quote!(
            pub struct Inputs(pub u32, pub bool, pub bool);
        );
        let output = derive_digital(decl).unwrap().to_string();
        let expected = expect_file!["expect/digital_with_tuple_struct.expect"];
        expected.assert_eq(&output);
    }
}
