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
                            Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1i64),
                            Kind::make_variant(stringify!(B),
                            rhdl_core::Kind::make_tuple(vec![< Bits:: < 16 > as
                            rhdl_core::Digital > ::static_kind()]), 2i64),
                            Kind::make_variant(stringify!(C),
                            rhdl_core::Kind::make_struct(vec![rhdl_core::Kind::make_field(stringify!(a),
                            < Bits:: < 32 > as rhdl_core::Digital > ::static_kind()),
                            rhdl_core::Kind::make_field(stringify!(b), < Bits:: < 8 > as
                            rhdl_core::Digital > ::static_kind())]), 3i64)
                        ],
                        2usize,
                        DiscriminantAlignment::Msb,
                    )
                }
                fn bin(self) -> Vec<bool> {
                    self.kind()
                        .pad(
                            match self {
                                Self::A => rhdl_bits::bits::<2usize>(1i64 as u128).to_bools(),
                                Self::B(_0) => {
                                    let mut v = rhdl_bits::bits::<2usize>(2i64 as u128)
                                        .to_bools();
                                    v.extend(_0.bin());
                                    v
                                }
                                Self::C { a, b } => {
                                    let mut v = rhdl_bits::bits::<2usize>(3i64 as u128)
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
                    builder.namespace("$disc").allocate(tag, 0);
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
                        Kind::make_variant(stringify!(Init), rhdl_core::Kind::Empty, 0i64),
                        Kind::make_variant(stringify!(Boot), rhdl_core::Kind::Empty, 1i64),
                        Kind::make_variant(stringify!(Running), rhdl_core::Kind::Empty, 2i64),
                        Kind::make_variant(stringify!(Stop), rhdl_core::Kind::Empty, 3i64),
                        Kind::make_variant(stringify!(Boom), rhdl_core::Kind::Empty, 4i64)
                    ],
                    3usize,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind()
                    .pad(
                        match self {
                            Self::Init => {
                                rhdl_bits::bits::<3usize>(0i64 as u128).to_bools()
                            }
                            Self::Boot => {
                                rhdl_bits::bits::<3usize>(1i64 as u128).to_bools()
                            }
                            Self::Running => {
                                rhdl_bits::bits::<3usize>(2i64 as u128).to_bools()
                            }
                            Self::Stop => {
                                rhdl_bits::bits::<3usize>(3i64 as u128).to_bools()
                            }
                            Self::Boom => {
                                rhdl_bits::bits::<3usize>(4i64 as u128).to_bools()
                            }
                        },
                    )
            }
            fn allocate<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                builder: impl rhdl_core::LogBuilder
            ) {
                builder.namespace("$disc").allocate(tag, 0);
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

#[test]
fn test_enum_with_signed_discriminants() {
    let decl = quote! {
        enum Test {
            A = 1,
            B = 3 + 6,
            C = -8,
        }
    };
    let output = derive_digital_enum(syn::parse2(decl).unwrap()).unwrap();
    let expected = quote! {
        impl rhdl_core::Digital for Test {
            fn static_kind() -> rhdl_core::Kind {
                Kind::make_enum(
                    vec![
                        Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1i64),
                        Kind::make_variant(stringify!(B), rhdl_core::Kind::Empty, 9i64),
                        Kind::make_variant(stringify!(C), rhdl_core::Kind::Empty, -8i64)
                    ],
                    5usize,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind()
                    .pad(
                        match self {
                            Self::A => {
                                rhdl_bits::signed_bits::<5usize>(1i64 as i128).to_bools()
                            }
                            Self::B => {
                                rhdl_bits::signed_bits::<5usize>(9i64 as i128).to_bools()
                            }
                            Self::C => {
                                rhdl_bits::signed_bits::<5usize>(-8i64 as i128).to_bools()
                            }
                        },
                    )
            }
            fn allocate<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                builder: impl rhdl_core::LogBuilder
            ) {
                builder.namespace("$disc").allocate(tag, 0);
            }
            fn record<L: rhdl_core::Digital>(
                &self,
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl
            ) {
                match self {
                    Self::A => {
                        logger.write_string(tag, stringify!(A));
                    }
                    Self::B => {
                        logger.write_string(tag, stringify!(B));
                    }
                    Self::C => {
                        logger.write_string(tag, stringify!(C));
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

#[test]
fn test_enum_with_discriminants() {
    let decl = quote! {
        enum Test {
            A = 1,
            B = 7-1,
            C = 8,
        }
    };
    let output = derive_digital_enum(syn::parse2(decl).unwrap()).unwrap();
    let expected = quote! {
        impl rhdl_core::Digital for Test {
            fn static_kind() -> rhdl_core::Kind {
                Kind::make_enum(vec![Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1i64),
                Kind::make_variant(stringify!(B), rhdl_core::Kind::Empty, 6i64),
                Kind::make_variant(stringify!(C), rhdl_core::Kind::Empty, 8i64)], 4usize, DiscriminantAlignment::Msb)
            }
            fn bin(self) -> Vec<bool> {
                self.kind()
                    .pad(
                        match self {
                            Self::A => {
                                rhdl_bits::bits::<4usize>(1i64 as u128).to_bools()
                            }
                            Self::B => {
                                rhdl_bits::bits::<4usize>(6i64 as u128).to_bools()
                            }
                            Self::C => {
                                rhdl_bits::bits::<4usize>(8i64 as u128).to_bools()
                            }
                        },
                    )
            }
            fn allocate<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                builder: impl rhdl_core::LogBuilder
            ) {
                builder.namespace("$disc").allocate(tag, 0);
            }
            fn record<L: rhdl_core::Digital>(
                &self,
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl
            ) {
                match self {
                    Self::A => {
                        logger.write_string(tag, stringify!(A));
                    }
                    Self::B => {
                        logger.write_string(tag, stringify!(B));
                    }
                    Self::C => {
                        logger.write_string(tag, stringify!(C));
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

#[test]
fn test_allocate_discriminants() {
    let x = vec![Some(5), None, Some(7), None, Some(9)];
    assert_eq!(allocate_discriminants(&x), vec![5, 6, 7, 8, 9]);
    let x = vec![None, None, Some(-1), Some(-3), None];
    assert_eq!(allocate_discriminants(&x), vec![0, 1, -1, -3, -2]);
}

#[test]
fn test_dicriminant_size_calculation() {
    let x = vec![-4, 3];
    assert_eq!(discriminant_width(&x), DiscriminantType::Signed(3));
    let x = vec![0, 3];
    assert_eq!(discriminant_width(&x), DiscriminantType::Unsigned(2));
    let x = vec![0, 3, 4];
    assert_eq!(discriminant_width(&x), DiscriminantType::Unsigned(3));
    let x = vec![-5, 3, 4, 5];
    assert_eq!(discriminant_width(&x), DiscriminantType::Signed(4));
    let x = vec![-1, 0, 1];
    assert_eq!(discriminant_width(&x), DiscriminantType::Signed(2));
    assert_eq!(discriminant_width(&[0, 1]), DiscriminantType::Unsigned(1));
    assert_eq!(discriminant_width(&[0, 3]), DiscriminantType::Unsigned(2));
    assert_eq!(discriminant_width(&[1, 7]), DiscriminantType::Unsigned(3));
    assert_eq!(discriminant_width(&[-8, 0]), DiscriminantType::Signed(4));
}

#[test]
fn test_parse_attributes() {
    let decl = quote! {
        #[derive(Digital)]
        #[rhdl(discriminant_align = "msb")]
        enum Test {
            Start = 1,
            Stop = 2,
            Boom = 3,
        }
    };
    let input: syn::DeriveInput = syn::parse2(decl).unwrap();
    let digital = derive_digital_enum(input).unwrap();
    assert!(digital.to_string().contains("Msb"));
    let decl = quote! {
        #[derive(Digital)]
        #[rhdl(discriminant_align = "lsb")]
        enum Test {
            Start = 1,
            Stop = 2,
            Boom = 3,
        }
    };
    let input: syn::DeriveInput = syn::parse2(decl).unwrap();
    let digital = derive_digital_enum(input).unwrap();
    assert!(digital.to_string().contains("Lsb"));
}

#[test]
fn test_width_override() {
    let decl = quote! {
        #[derive(Digital)]
        #[rhdl(discriminant_width = 8)]
        enum Test {
            Start = 1,
            Stop = 2,
            Boom = 3
        }
    };
    let input: syn::DeriveInput = syn::parse2(decl).unwrap();
    let digital = derive_digital_enum(input).unwrap();
    assert!(digital.to_string().contains("8usize"));
}
