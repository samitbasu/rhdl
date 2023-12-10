use crate::utils::{assert_frag_eq, assert_tokens_eq};

use super::*;

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
                    rhdl_core::Kind::make_enum(
                        concat!(module_path!(), "::", stringify!(Test)),
                        vec![
                            rhdl_core::Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1i64),
                            rhdl_core::Kind::make_variant(stringify!(B),
                            rhdl_core::Kind::make_tuple(vec![< Bits:: < 16 > as
                            rhdl_core::Digital > ::static_kind()]), 2i64),
                            rhdl_core::Kind::make_variant(stringify!(C),
                            rhdl_core::Kind::make_struct(stringify!(_Test__C), vec![rhdl_core::Kind::make_field(stringify!(a),
                            < Bits:: < 32 > as rhdl_core::Digital > ::static_kind()),
                            rhdl_core::Kind::make_field(stringify!(b), < Bits:: < 8 > as
                            rhdl_core::Digital > ::static_kind())]), 3i64)
                        ],
                        2usize,
                        rhdl_core::DiscriminantAlignment::Msb,
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
                fn discriminant(self) -> rhdl_core::TypedBits {
                    match self {
                        Self::A => rhdl_bits::bits::<2usize>(1i64 as u128).typed_bits(),
                        Self::B(_0) => rhdl_bits::bits::<2usize>(2i64 as u128).typed_bits(),
                        Self::C { a, b } => {
                            rhdl_bits::bits::<2usize>(3i64 as u128).typed_bits()
                        }
                    }
                }
                fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                    match self {
                        Self::A => {
                            writer.write_string(key, stringify!(A));
                            writer.write_bits((key, ".__disc"), 1i64 as u128, 2u8);
                        }
                        Self::B(_0) => {
                            writer.write_string(key, stringify!(B));
                            writer.write_bits((key, ".__disc"), 2i64 as u128, 2u8);
                            rhdl_core::Digital::note(_0, (key, 0usize), &mut writer);
                        }
                        Self::C { a, b } => {
                            writer.write_string(key, stringify!(C));
                            writer.write_bits((key, ".__disc"), 3i64 as u128, 2u8);
                            rhdl_core::Digital::note(a, (key, stringify!(a)), &mut writer);
                            rhdl_core::Digital::note(b, (key, stringify!(b)), &mut writer);
                        }
                    }
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
                rhdl_core::Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(State)),
                    vec![
                        rhdl_core::Kind::make_variant(stringify!(Init), rhdl_core::Kind::Empty, 0i64),
                        rhdl_core::Kind::make_variant(stringify!(Boot), rhdl_core::Kind::Empty, 1i64),
                        rhdl_core::Kind::make_variant(stringify!(Running), rhdl_core::Kind::Empty, 2i64),
                        rhdl_core::Kind::make_variant(stringify!(Stop), rhdl_core::Kind::Empty, 3i64),
                        rhdl_core::Kind::make_variant(stringify!(Boom), rhdl_core::Kind::Empty, 4i64)
                    ],
                    3usize,
                    rhdl_core::DiscriminantAlignment::Msb,
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
            fn discriminant(self) -> rhdl_core::TypedBits {
                match self {
                    Self::Init => rhdl_bits::bits::<3usize>(0i64 as u128).typed_bits(),
                    Self::Boot => rhdl_bits::bits::<3usize>(1i64 as u128).typed_bits(),
                    Self::Running => rhdl_bits::bits::<3usize>(2i64 as u128).typed_bits(),
                    Self::Stop => rhdl_bits::bits::<3usize>(3i64 as u128).typed_bits(),
                    Self::Boom => rhdl_bits::bits::<3usize>(4i64 as u128).typed_bits(),
                }
            }
            fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                match self {
                    Self::Init => {
                        writer.write_string(key, stringify!(Init));
                        writer.write_bits((key, ".__disc"), 0i64 as u128, 3u8);
                    }
                    Self::Boot => {
                        writer.write_string(key, stringify!(Boot));
                        writer.write_bits((key, ".__disc"), 1i64 as u128, 3u8);
                    }
                    Self::Running => {
                        writer.write_string(key, stringify!(Running));
                        writer.write_bits((key, ".__disc"), 2i64 as u128, 3u8);
                    }
                    Self::Stop => {
                        writer.write_string(key, stringify!(Stop));
                        writer.write_bits((key, ".__disc"), 3i64 as u128, 3u8);
                    }
                    Self::Boom => {
                        writer.write_string(key, stringify!(Boom));
                        writer.write_bits((key, ".__disc"), 4i64 as u128, 3u8);
                    }
                }
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
                rhdl_core::Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Test)),
                    vec![
                        rhdl_core::Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1i64),
                        rhdl_core::Kind::make_variant(stringify!(B), rhdl_core::Kind::Empty, 9i64),
                        rhdl_core::Kind::make_variant(stringify!(C), rhdl_core::Kind::Empty, -8i64)
                    ],
                    5usize,
                    rhdl_core::DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind()
                    .pad(
                        match self {
                            Self::A => {
                                rhdl_bits::signed::<5usize>(1i64 as i128).to_bools()
                            }
                            Self::B => {
                                rhdl_bits::signed::<5usize>(9i64 as i128).to_bools()
                            }
                            Self::C => {
                                rhdl_bits::signed::<5usize>(-8i64 as i128).to_bools()
                            }
                        },
                    )
            }
            fn discriminant(self) -> rhdl_core::TypedBits {
                match self {
                    Self::A => rhdl_bits::signed::<5usize>(1i128).typed_bits(),
                    Self::B => rhdl_bits::signed::<5usize>(9i128).typed_bits(),
                    Self::C => rhdl_bits::signed::<5usize>(-8i128).typed_bits(),
                }
            }
            fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                match self {
                    Self::A => {
                        writer.write_string(key, stringify!(A));
                        writer.write_signed((key, ".__disc"), 1i64 as i128, 5u8);
                    }
                    Self::B => {
                        writer.write_string(key, stringify!(B));
                        writer.write_signed((key, ".__disc"), 9i64 as i128, 5u8);
                    }
                    Self::C => {
                        writer.write_string(key, stringify!(C));
                        writer.write_signed((key, ".__disc"), -8i64 as i128, 5u8);
                    }
                }
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
                rhdl_core::Kind::make_enum(concat!(module_path!(), "::", stringify!(Test)), vec![rhdl_core::Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1i64),
                rhdl_core::Kind::make_variant(stringify!(B), rhdl_core::Kind::Empty, 6i64),
                rhdl_core::Kind::make_variant(stringify!(C), rhdl_core::Kind::Empty, 8i64)], 4usize, rhdl_core::DiscriminantAlignment::Msb)
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
            fn discriminant(self) -> rhdl_core::TypedBits {
                match self {
                    Self::A => rhdl_bits::bits::<4usize>(1i64 as u128).typed_bits(),
                    Self::B => rhdl_bits::bits::<4usize>(6i64 as u128).typed_bits(),
                    Self::C => rhdl_bits::bits::<4usize>(8i64 as u128).typed_bits(),
                }
            }
            fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                match self {
                    Self::A => {
                        writer.write_string(key, stringify!(A));
                        writer.write_bits((key, ".__disc"), 1i64 as u128, 4u8);
                    }
                    Self::B => {
                        writer.write_string(key, stringify!(B));
                        writer.write_bits((key, ".__disc"), 6i64 as u128, 4u8);
                    }
                    Self::C => {
                        writer.write_string(key, stringify!(C));
                        writer.write_bits((key, ".__disc"), 8i64 as u128, 4u8);
                    }
                }
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
    assert_eq!(discriminant_kind(&x), DiscriminantType::Signed(3));
    let x = vec![0, 3];
    assert_eq!(discriminant_kind(&x), DiscriminantType::Unsigned(2));
    let x = vec![0, 3, 4];
    assert_eq!(discriminant_kind(&x), DiscriminantType::Unsigned(3));
    let x = vec![-5, 3, 4, 5];
    assert_eq!(discriminant_kind(&x), DiscriminantType::Signed(4));
    let x = vec![-1, 0, 1];
    assert_eq!(discriminant_kind(&x), DiscriminantType::Signed(2));
    assert_eq!(discriminant_kind(&[0, 1]), DiscriminantType::Unsigned(1));
    assert_eq!(discriminant_kind(&[0, 3]), DiscriminantType::Unsigned(2));
    assert_eq!(discriminant_kind(&[1, 7]), DiscriminantType::Unsigned(3));
    assert_eq!(discriminant_kind(&[-8, 0]), DiscriminantType::Signed(4));
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
