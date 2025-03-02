use expect_test::expect_file;

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
            #[rhdl(unmatched)]
            Unknown,
        }
    };
    let output = derive_digital_enum(input).unwrap().to_string();
    let expected = expect_file!["expect/enum_derive.expect"];
    expected.assert_eq(&output);
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
            #[rhdl(unmatched)]
            Unknown,
        }
    );
    let output = derive_digital_enum(syn::parse2(decl).unwrap())
        .unwrap()
        .to_string();
    let expected = expect_file!["expect/enum_no_payloads.expect"];
    expected.assert_eq(&output);
}

#[test]
fn test_enum_with_signed_discriminants() {
    let decl = quote! {
        enum Test {
            A = 1,
            B = 3 + 6,
            C = -8,
            #[rhdl(unmatched)]
            Unknown,
        }
    };
    let output = derive_digital_enum(syn::parse2(decl).unwrap())
        .unwrap()
        .to_string();
    let expected = expect_file!["expect/enum_with_signed_discriminants.expect"];
    expected.assert_eq(&output);
}

#[test]
fn test_enum_with_discriminants() {
    let decl = quote! {
        enum Test {
            A = 1,
            B = 7-1,
            C = 8,
            #[rhdl(unmatched)]
            Unknown,
        }
    };
    let output = derive_digital_enum(syn::parse2(decl).unwrap())
        .unwrap()
        .to_string();
    let expected = expect_file!["expect/enum_with_discriminants.expect"];
    expected.assert_eq(&output);
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
            Krack = 0,
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
            #[rhdl(unmatched)]
            Unknown
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
            Boom = 3,
            #[rhdl(unmatched)]
            Unknown
        }
    };
    let input: syn::DeriveInput = syn::parse2(decl).unwrap();
    let digital = derive_digital_enum(input).unwrap();
    assert!(digital.to_string().contains("8usize"));
}

#[test]
fn test_unmatched_fits_in_width() {
    let decl = quote! {
        #[derive(Digital)]
        #[rhdl(discriminant_width = 3)]
        enum Test {
            Start = 1,
            Stop = 2,
            Boom = 3,
            #[rhdl(unmatched)]
            Unknown
        }
    };
    let input: syn::DeriveInput = syn::parse2(decl).unwrap();
    let _digital = derive_digital_enum(input).unwrap();
}
