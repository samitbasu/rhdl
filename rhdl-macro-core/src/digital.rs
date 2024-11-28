use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

use crate::{digital_enum::derive_digital_enum, utils::parse_rhdl_skip_attribute};

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
                    fn bin(self) -> Vec<bool> {
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
                    fn maybe_init() -> Self {
                        Self(
                            #(
                                <#field_types as rhdl::core::Digital>::maybe_init(),
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
                    fn bin(self) -> Vec<bool> {
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
                    fn maybe_init() -> Self {
                        Self {
                            #(
                                #fields: <#field_types as rhdl::core::Digital>::maybe_init(),
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
    use crate::utils::assert_tokens_eq;

    #[test]
    fn test_digital_proc_macro() {
        let decl = quote!(
            pub struct NestedBits {
                nest_1: bool,
                nest_2: u8,
                nest_3: TwoBits,
            }
        );
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl::core::Digital for NestedBits {
                fn static_kind() -> rhdl::core::Kind {
                    rhdl::core::Kind::make_struct(
                        concat!(module_path!(), "::", stringify!(NestedBits)),
                        vec![
                        rhdl::core::Kind::make_field(stringify!(nest_1), <bool as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(nest_2), <u8 as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(nest_3), <TwoBits as rhdl::core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.nest_1.bin());
                    result.extend(self.nest_2.bin());
                    result.extend(self.nest_3.bin());
                    result
                }
            }
        };
        assert_tokens_eq(&expected, &output);
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
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl::core::Digital for Inputs {
                fn static_kind() -> rhdl::core::Kind {
                    rhdl::core::Kind::make_struct(
                        concat!(module_path!(), "::", stringify!(Inputs)),
                        vec![
                        rhdl::core::Kind::make_field(stringify!(input), <u32 as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(write), <bool as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(read), <bool as rhdl::core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.input.bin());
                    result.extend(self.write.bin());
                    result.extend(self.read.bin());
                    result
                }
            }
        };
        assert_tokens_eq(&expected, &output);
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
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl<T: Digital> rhdl::core::Digital for Inputs<T> {
                fn static_kind() -> rhdl::core::Kind {
                    rhdl::core::Kind::make_struct(
                        &vec![
                            module_path!().to_string(),"::".to_string(),
                            stringify!(Inputs).to_string(), "<".to_string(), std::any::type_name::<T>().to_string(), ">".to_string()
                        ].join(""),
                        vec![
                        rhdl::core::Kind::make_field(stringify!(input), <T as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(write), <bool as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(read), <bool as rhdl::core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.input.bin());
                    result.extend(self.write.bin());
                    result.extend(self.read.bin());
                    result
                }
            }
        };
        assert_tokens_eq(&expected, &output);
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
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl::core::Digital for Inputs {
                fn static_kind() -> rhdl::core::Kind {
                    rhdl::core::Kind::make_struct(
                        concat!(module_path!(), "::", stringify!(Inputs)),
                        vec![
                        rhdl::core::Kind::make_field(stringify!(input), <u32 as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(write), <bool as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(read), <(bool, bool) as rhdl::core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.input.bin());
                    result.extend(self.write.bin());
                    result.extend(self.read.bin());
                    result
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_digital_with_tuple_struct() {
        let decl = quote!(
            pub struct Inputs(pub u32, pub bool, pub bool);
        );
        let output = derive_digital(decl).unwrap();
        let expected = quote! {
            impl rhdl::core::Digital for Inputs {
                fn static_kind() -> rhdl::core::Kind {
                    rhdl::core::Kind::make_struct(
                        concat!(module_path!(), "::", stringify!(Inputs)),
                        vec![
                        rhdl::core::Kind::make_field(stringify!(0), <u32 as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(1), <bool as rhdl::core::Digital>::static_kind()),
                        rhdl::core::Kind::make_field(stringify!(2), <bool as rhdl::core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.0.bin());
                    result.extend(self.1.bin());
                    result.extend(self.2.bin());
                    result
                }
            }
            impl rhdl::core::DigitalFn for Inputs {
                fn kernel_fn() -> Option<rhdl::core::KernelFnKind> {
                    Some(rhdl::core::KernelFnKind::TupleStructConstructor(<Self as rhdl::core::Digital>::static_kind().place_holder()))
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }
}
