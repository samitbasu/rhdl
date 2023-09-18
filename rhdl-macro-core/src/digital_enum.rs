use anyhow::anyhow;
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

fn variant_kind_mapping(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {rhdl_core::Kind::Empty},
        syn::Fields::Unnamed(fields) => {
            let field_types = fields.unnamed.iter().map(|f| &f.ty);
            quote! {
                rhdl_core::Kind::make_tuple(vec![#(
                    <#field_types as rhdl_core::Digital>::static_kind()
                ),*])
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            let field_types = fields.named.iter().map(|f| &f.ty);
            quote! {
                rhdl_core::Kind::make_struct(vec![#(
                    rhdl_core::Kind::make_field(stringify!(#field_names), <#field_types as rhdl_core::Digital>::static_kind())
                ),*])
            }
        }
    }
}

fn variant_payload_bin(variant: &Variant, width: usize, discriminant: usize) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {
            rhdl_bits::bits::<#width>(#discriminant as u128).to_bools()
        },
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("_{}", i));
            quote! {
                let mut v = rhdl_bits::bits::<#width>(#discriminant as u128).to_bools();
                #(
                    v.extend(#field_names.bin());
                )*
                v
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                let mut v = rhdl_bits::bits::<#width>(#discriminant as u128).to_bools();
                #(
                    v.extend(#field_names.bin());
                )*
                v
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

pub const fn clog2(t: usize) -> usize {
    let mut p = 0;
    let mut b = 1;
    while b < t {
        p += 1;
        b *= 2;
    }
    p
}

pub fn derive_digital_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|x| &x.ident);
            let variant_destructure_args = e.variants.iter().map(variant_destructure_args);
            let variant_destructure_args_for_bin = variant_destructure_args.clone();
            // For each variant, we need to create the allocate and record functions if the variant has fields
            let allocate_fns = e.variants.iter().map(variant_allocate);
            let record_fns = e
                .variants
                .iter()
                .map(|variant| variant_payload_case(variant, e.variants.iter()));
            let skip_fns = e.variants.iter().map(variant_payload_skip);
            let kind_mapping = e.variants.iter().map(variant_kind_mapping);
            let variant_names_for_kind = variants.clone();
            let variant_names_for_bin = variants.clone();
            let width = clog2(e.variants.len());
            let bin_fns = e
                .variants
                .iter()
                .enumerate()
                .map(|(ndx, variant)| variant_payload_bin(variant, width, ndx));
            let discriminants = e.variants.iter().map(|x| {
                x.discriminant
                    .as_ref()
                    .map(|x| &x.1)
                    .map(|x| quote! {Some(#x)})
                    .unwrap_or(quote! {None})
            });
            Ok(quote! {
                impl #impl_generics rhdl_core::Digital for #enum_name #ty_generics #where_clause {
                    fn static_kind() -> rhdl_core::Kind {
                        Kind::make_enum(
                            vec![
                                #(
                                    Kind::make_variant(stringify!(#variant_names_for_kind), #kind_mapping).with_discriminant(#discriminants)
                                ),*
                            ],
                            None,
                            DiscriminantAlignment::Msb
                        )
                    }
                    fn bin(self) -> Vec<bool> {
                        self.kind().pad(match self {
                            #(
                                Self::#variant_names_for_bin #variant_destructure_args_for_bin => {#bin_fns}
                            )*
                        })

                    }
                    fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                        builder.allocate(tag, 0);
                        #(
                            #allocate_fns
                        )*
                    }
                    fn record<L: rhdl_core::Digital>(&self, tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                        match self {
                            #(
                                Self::#variants #variant_destructure_args => {#record_fns},
                            )*
                        }
                    }
                    fn skip<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, mut logger: impl rhdl_core::LoggerImpl) {
                        logger.skip(tag);
                        #(
                            #skip_fns;
                        )*
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
        assert_eq!(quote! {}.to_string(), a_fn.to_string());
        assert_eq!(quote! { (_0) }.to_string(), b_fn.to_string());
        assert_eq!(quote! { { a, b } }.to_string(), c_fn.to_string());
    }
    #[test]
    fn test_enum_derive() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Test {
                A = 1,
                B(Bits::<16>),
                C {a: Bits::<32>, b: Bits::<8>},
            }
        };
        let output = derive_digital_enum(input).unwrap();
        assert_frag_eq(
            &output,
            &quote! {
                impl rhdl_core::Digital for Test {
                    fn static_kind() -> rhdl_core::Kind {
                        Kind::make_enum(
                            vec![
                                Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty)
                                .with_discriminant(Some(1)), Kind::make_variant(stringify!(B),
                                rhdl_core::Kind::make_tuple(vec![< Bits:: < 16 > as
                                rhdl_core::Digital > ::static_kind()])).with_discriminant(None),
                                Kind::make_variant(stringify!(C),
                                rhdl_core::Kind::make_struct(vec![rhdl_core::Kind::make_field(stringify!(a),
                                < Bits:: < 32 > as rhdl_core::Digital > ::static_kind()),
                                rhdl_core::Kind::make_field(stringify!(b), < Bits:: < 8 > as
                                rhdl_core::Digital > ::static_kind())])).with_discriminant(None)
                            ],
                            None,
                            DiscriminantAlignment::Msb,
                        )
                    }
                    fn bin(self) -> Vec<bool> {
                        self.kind()
                            .pad(
                                match self {
                                    Self::A => rhdl_bits::bits::<2usize>(0usize as u128).to_bools(),
                                    Self::B(_0) => {
                                        let mut v = rhdl_bits::bits::<2usize>(1usize as u128)
                                            .to_bools();
                                        v.extend(_0.bin());
                                        v
                                    }
                                    Self::C { a, b } => {
                                        let mut v = rhdl_bits::bits::<2usize>(2usize as u128)
                                            .to_bools();
                                        v.extend(a.bin());
                                        v.extend(b.bin());
                                        v
                                    }
                                },
                            )
                    }
                    fn allocate<L: rhdl_core::Digital>(
                        tag: rhdl_core::TagID<L>,
                        builder: impl rhdl_core::LogBuilder,
                    ) {
                        builder.allocate(tag, 0);
                        {
                            let mut builder = builder.namespace(stringify!(B));
                            <Bits<
                                16,
                            > as rhdl_core::Digital>::allocate(
                                tag,
                                builder.namespace(stringify!(0)),
                            );
                        }
                        {
                            let mut builder = builder.namespace(stringify!(C));
                            <Bits<
                                32,
                            > as rhdl_core::Digital>::allocate(
                                tag,
                                builder.namespace(stringify!(a)),
                            );
                            <Bits<
                                8,
                            > as rhdl_core::Digital>::allocate(
                                tag,
                                builder.namespace(stringify!(b)),
                            );
                        }
                    }
                    fn record<L: rhdl_core::Digital>(
                        &self,
                        tag: rhdl_core::TagID<L>,
                        mut logger: impl rhdl_core::LoggerImpl,
                    ) {
                        match self {
                            Self::A => {
                                logger.write_string(tag, stringify!(A));
                                <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                                <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                                <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
                            }
                            Self::B(_0) => {
                                logger.write_string(tag, stringify!(B));
                                _0.record(tag, &mut logger);
                                <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                                <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
                            }
                            Self::C { a, b } => {
                                logger.write_string(tag, stringify!(C));
                                <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                                a.record(tag, &mut logger);
                                b.record(tag, &mut logger);
                            }
                        }
                    }
                    fn skip<L: rhdl_core::Digital>(
                        tag: rhdl_core::TagID<L>,
                        mut logger: impl rhdl_core::LoggerImpl,
                    ) {
                        logger.skip(tag);
                        <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
                    }
                }
            },
        );
    }

    #[test]
    fn test_enum_no_payloads() {
        let decl = quote!(
            pub enum State {
                Init,
                Boot,
                Running,
                Stop,
                Boom,
            }
        );
        let output = derive_digital_enum(syn::parse2(decl).unwrap()).unwrap();
        let expected = quote! {
            impl rhdl_core::Digital for State {
                fn static_kind() -> rhdl_core::Kind {
                    Kind::make_enum(
                        vec![
                            Kind::make_variant(stringify!(Init), rhdl_core::Kind::Empty)
                            .with_discriminant(None), Kind::make_variant(stringify!(Boot),
                            rhdl_core::Kind::Empty).with_discriminant(None),
                            Kind::make_variant(stringify!(Running), rhdl_core::Kind::Empty)
                            .with_discriminant(None), Kind::make_variant(stringify!(Stop),
                            rhdl_core::Kind::Empty).with_discriminant(None),
                            Kind::make_variant(stringify!(Boom), rhdl_core::Kind::Empty)
                            .with_discriminant(None)
                        ],
                        None,
                        DiscriminantAlignment::Msb,
                    )
                }
                fn bin(self) -> Vec<bool> {
                    self.kind()
                        .pad(
                            match self {
                                Self::Init => {
                                    rhdl_bits::bits::<3usize>(0usize as u128).to_bools()
                                }
                                Self::Boot => {
                                    rhdl_bits::bits::<3usize>(1usize as u128).to_bools()
                                }
                                Self::Running => {
                                    rhdl_bits::bits::<3usize>(2usize as u128).to_bools()
                                }
                                Self::Stop => {
                                    rhdl_bits::bits::<3usize>(3usize as u128).to_bools()
                                }
                                Self::Boom => {
                                    rhdl_bits::bits::<3usize>(4usize as u128).to_bools()
                                }
                            },
                        )
                }
                fn allocate<L: rhdl_core::Digital>(
                    tag: rhdl_core::TagID<L>,
                    builder: impl rhdl_core::LogBuilder
                ) {
                    builder.allocate(tag, 0);
                }
                fn record<L: rhdl_core::Digital>(
                    &self,
                    tag: rhdl_core::TagID<L>,
                    mut logger: impl rhdl_core::LoggerImpl
                ) {
                    match self {
                        Self::Init => {
                            logger.write_string(tag, stringify!(Init));
                        }
                        Self::Boot => {
                            logger.write_string(tag, stringify!(Boot));
                        }
                        Self::Running => {
                            logger.write_string(tag, stringify!(Running));
                        }
                        Self::Stop => {
                            logger.write_string(tag, stringify!(Stop));
                        }
                        Self::Boom => {
                            logger.write_string(tag, stringify!(Boom));
                        }
                    }
                }
                fn skip<L: rhdl_core::Digital>(
                    tag: rhdl_core::TagID<L>,
                    mut logger: impl rhdl_core::LoggerImpl
                ) {
                    logger.skip(tag);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }
}
