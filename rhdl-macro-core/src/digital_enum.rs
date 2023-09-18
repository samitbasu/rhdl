use anyhow::anyhow;
use anyhow::bail;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Variant;
use syn::{Data, DeriveInput};

// By convention, fields of a tuple are named _0, _1, ...
fn variant_payload_record(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("_{}", i));
            quote! {
                #(
                    #field_names.record(tag, &mut logger);
                )*
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                #(
                    #field_names.record(tag, &mut logger);
                )*
            }
        }
    }
}

fn variant_payload_skip(variant: &Variant) -> TokenStream {
    let field_types = variant.fields.iter().map(|f| &f.ty);
    quote! (
        #(
            <#field_types as rhdl_core::Digital>::skip(tag, &mut logger);
        )*
    )
}

// Generate the body of a record function for a specific variant.
// We write the variant name, and then for the matching payload,
// we write the actual data.  For all other payloads, we call the
// skip function.
fn variant_payload_case<'a>(
    variant: &Variant,
    all_variants: impl Iterator<Item = &'a Variant>,
) -> TokenStream {
    // The skips have to be called in order
    let record_or_skip = all_variants.map(|x| {
        if x.ident == variant.ident {
            variant_payload_record(x)
        } else {
            variant_payload_skip(x)
        }
    });
    let variant_name = &variant.ident;
    quote!(
        logger.write_string(tag, stringify!(#variant_name));
        #(
            #record_or_skip
        )*
    )
}

// Generate the payload destructure arguments used in the 
// match
fn variant_destructure_args(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("_{}", i));
            quote! {
                (#(
                    #field_names
                ),*)
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                {
                    #(
                        #field_names
                    ),*
                }
            }
        }
    }
}


fn variant_allocate(variant: &Variant) -> TokenStream {
    let variant_name = &variant.ident;
    match &variant.fields {
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| syn::Index::from(i));
            let field_types = fields.unnamed.iter().map(|f| &f.ty);
            quote! (
                {
                    let mut builder = builder.namespace(stringify!(#variant_name));
                    #(
                        <#field_types as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(#field_names)));
                    )*
                }
            )
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            let field_types = fields.named.iter().map(|f| &f.ty);
            quote!(
            {
                let mut builder = builder.namespace(stringify!(#variant_name));
                #(
                    <#field_types as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(#field_names)));
                )*
            })
        }
    }
}

pub fn derive_digital_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|x| &x.ident);
            // For each variant, we need to create the allocate and record functions if the variant has fields
            let allocate_fns = e.variants.iter().map(variant_allocate);
            Ok(quote! {
                impl #impl_generics rhdl_core::Digital for #enum_name #ty_generics #where_clause {
                    fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                        builder.allocate(tag, 0);
                        #(
                            #allocate_fns
                        )*
                    }
                    fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                        match self {
                            #(
                                Self::#variants => logger.write_string(tag, stringify!(#variants)),
                            )*
                        }
                    }
                }
            })
        }
        _ => Err(anyhow!("Only enums can be digital")),
    }
}

#[cfg(test)]
mod test {
    use crate::utils::{assert_frag_eq, assert_tokens_eq};

    use super::*;

    #[test]
    fn test_allocate_function() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Test {
                A,
                B(u32),
                C { a: u32, b: u32 },
            }
        };
        let e = match input.data {
            Data::Enum(e) => e,
            _ => panic!("Not an enum"),
        };
        let a = &e.variants[0];
        let b = &e.variants[1];
        let c = &e.variants[2];
        let a_fn = variant_allocate(a);
        let b_fn = variant_allocate(b);
        let c_fn = variant_allocate(c);
        assert_frag_eq(&quote! {}, &a_fn);
        assert_frag_eq(
            &quote! {
                {
                    let mut builder = builder.namespace(stringify!(B));
                    <u32 as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(0)));
                }
            },
            &b_fn,
        );
        assert_frag_eq(
            &quote! {
                {
                    let mut builder = builder.namespace(stringify!(C));
                    <u32 as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(a)));
                    <u32 as rhdl_core::Digital>::allocate(tag, builder.namespace(stringify!(b)));
                }
            },
            &c_fn,
        );
    }

    #[test]
    fn test_variant_payload_record_function() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Test {
                A,
                B(u32),
                C { a: u32, b: u32 },
            }
        };
        let e = match input.data {
            Data::Enum(e) => e,
            _ => panic!("Not an enum"),
        };
        let a = &e.variants[0];
        let b = &e.variants[1];
        let c = &e.variants[2];
        let a_fn = variant_payload_record(a);
        let b_fn = variant_payload_record(b);
        let c_fn = variant_payload_record(c);
        assert_frag_eq(&quote! {}, &a_fn);
        assert_frag_eq(
            &quote! {
                _0.record(tag, &mut logger);
            },
            &b_fn,
        );
        assert_frag_eq(
            &quote! {
                a.record(tag, &mut logger);
                b.record(tag, &mut logger);
            },
            &c_fn,
        );
    }

    #[test]
    fn test_variant_payload_skip_function() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Test {
                A,
                B(u32),
                C { a: u32, b: u32 },
            }
        };
        let e = match input.data {
            Data::Enum(e) => e,
            _ => panic!("Not an enum"),
        };
        let a = &e.variants[0];
        let b = &e.variants[1];
        let c = &e.variants[2];
        let a_fn = variant_payload_skip(a);
        let b_fn = variant_payload_skip(b);
        let c_fn = variant_payload_skip(c);
        assert_frag_eq(&quote! {}, &a_fn);
        assert_frag_eq(
            &quote! {
                <u32 as rhdl_core::Digital>::skip(tag, &mut logger);
            },
            &b_fn,
        );
        assert_frag_eq(
            &quote! {
                <u32 as rhdl_core::Digital>::skip(tag, &mut logger);
                <u32 as rhdl_core::Digital>::skip(tag, &mut logger);
            },
            &c_fn,
        );
    }

    #[test]
    fn test_variant_payload_case_function() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Test {
                A,
                B(u16),
                C { a: u32, b: u8 },
            }
        };
        let e = match input.data {
            Data::Enum(e) => e,
            _ => panic!("Not an enum"),
        };
        let a = &e.variants[0];
        let b = &e.variants[1];
        let c = &e.variants[2];
        let a_fn = variant_payload_case(a, e.variants.iter());
        let b_fn = variant_payload_case(b, e.variants.iter());
        let c_fn = variant_payload_case(c, e.variants.iter());
        assert_frag_eq(
            &quote! {
                logger.write_string(tag, stringify!(A));
                <u16 as rhdl_core::Digital>::skip(tag, &mut logger);
                <u32 as rhdl_core::Digital>::skip(tag, &mut logger);
                <u8 as rhdl_core::Digital>::skip(tag, &mut logger);
            },
            &a_fn,
        );
        assert_frag_eq(
            &quote! {
                logger.write_string(tag, stringify!(B));
                _0.record(tag, &mut logger);
                <u32 as rhdl_core::Digital>::skip(tag, &mut logger);
                <u8 as rhdl_core::Digital>::skip(tag, &mut logger);
            },
            &b_fn,
        );
        assert_frag_eq(
            &quote! {
                logger.write_string(tag, stringify!(C));
                <u16 as rhdl_core::Digital>::skip(tag, &mut logger);
                a.record(tag, &mut logger);
                b.record(tag, &mut logger);
            },
            &c_fn,
        );
    }

    #[test]
    fn test_variant_destructure_args_function() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Test {
                A,
                B(u16),
                C { a: u32, b: u8 },
            }
        };
        let e = match input.data {
            Data::Enum(e) => e,
            _ => panic!("Not an enum"),
        };
        let a = &e.variants[0];
        let b = &e.variants[1];
        let c = &e.variants[2];
        let a_fn = variant_destructure_args(a);
        let b_fn = variant_destructure_args(b);
        let c_fn = variant_destructure_args(c);
        assert_frag_eq(&quote! {}, &a_fn);
        assert_frag_eq(&quote! { (_0, ) }, &b_fn);
        assert_frag_eq(&quote! { { a, b } }, &c_fn);
    }
}
