use expect_test::expect;

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
    let expected = expect![[r#"impl core :: marker :: Copy for Test { } impl Clone for Test { # [inline] fn clone (& self) -> Self { match self { Test :: A => Test :: A , Test :: B (a ,) => Test :: B (a . clone () ,) , Test :: C { a , b , } => Test :: C { a : a . clone () , b : b . clone () , } , Test :: Unknown => Test :: Unknown , } } } impl rhdl :: core :: Digital for Test { const BITS : usize = 3usize + rhdl :: core :: const_max ! (0_usize , < Bits :: < 16 > as rhdl :: core :: Digital > :: BITS , < Bits :: < 32 > as rhdl :: core :: Digital > :: BITS + < Bits :: < 8 > as rhdl :: core :: Digital > :: BITS , 0_usize) ; const TRACE_BITS : usize = 3usize + rhdl :: core :: const_max ! (0_usize , < Bits :: < 16 > as rhdl :: core :: Digital > :: TRACE_BITS , < Bits :: < 32 > as rhdl :: core :: Digital > :: TRACE_BITS + < Bits :: < 8 > as rhdl :: core :: Digital > :: TRACE_BITS , 0_usize) ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_enum (concat ! (module_path ! () , "::" , stringify ! (Test)) , vec ! [rhdl :: core :: Kind :: make_variant (stringify ! (A) , rhdl :: core :: Kind :: Empty , 1i64) , rhdl :: core :: Kind :: make_variant (stringify ! (B) , rhdl :: core :: Kind :: make_tuple (vec ! [< Bits :: < 16 > as rhdl :: core :: Digital > :: static_kind ()]) , 2i64) , rhdl :: core :: Kind :: make_variant (stringify ! (C) , rhdl :: core :: Kind :: make_struct (stringify ! (_Test__C) , vec ! [rhdl :: core :: Kind :: make_field (stringify ! (a) , < Bits :: < 32 > as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (b) , < Bits :: < 8 > as rhdl :: core :: Digital > :: static_kind ())]) , 3i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Unknown) , rhdl :: core :: Kind :: Empty , 4i64)] , rhdl :: core :: Kind :: make_discriminant_layout (3usize , rhdl :: core :: DiscriminantAlignment :: Msb , rhdl :: core :: DiscriminantType :: Unsigned)) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_enum (concat ! (module_path ! () , "::" , stringify ! (Test)) , vec ! [rhdl :: rtt :: make_variant (stringify ! (A) , rhdl :: rtt :: TraceType :: Empty , 1i64) , rhdl :: rtt :: make_variant (stringify ! (B) , rhdl :: rtt :: make_tuple (vec ! [< Bits :: < 16 > as rhdl :: core :: Digital > :: static_trace_type ()]) , 2i64) , rhdl :: rtt :: make_variant (stringify ! (C) , rhdl :: rtt :: make_struct (stringify ! (_Test__C) , vec ! [rhdl :: rtt :: make_field (stringify ! (a) , < Bits :: < 32 > as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (b) , < Bits :: < 8 > as rhdl :: core :: Digital > :: static_trace_type ())]) , 3i64) , rhdl :: rtt :: make_variant (stringify ! (Unknown) , rhdl :: rtt :: TraceType :: Empty , 4i64)] , rhdl :: rtt :: make_discriminant_layout (3usize , rhdl :: core :: DiscriminantAlignment :: Msb . into () , rhdl :: core :: DiscriminantType :: Unsigned . into ())) } fn bin (self) -> Vec < rhdl :: core :: BitX > { let mut raw = match self { Self :: A => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (1i64 as u128) . to_bools ()) } Self :: B (_0) => { let mut v = rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (2i64 as u128) . to_bools ()) ; v . extend (_0 . bin ()) ; v } Self :: C { a , b } => { let mut v = rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (3i64 as u128) . to_bools ()) ; v . extend (a . bin ()) ; v . extend (b . bin ()) ; v } Self :: Unknown => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (4i64 as u128) . to_bools ()) } } ; raw . resize (Self :: BITS , rhdl :: core :: BitX :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 3usize) } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { let mut raw = match self { Self :: A => { rhdl :: bits :: bits :: < W3 > (1i64 as u128) . trace () } Self :: B (_0) => { let mut v = rhdl :: bits :: bits :: < W3 > (2i64 as u128) . trace () ; v . extend (_0 . trace ()) ; v } Self :: C { a , b } => { let mut v = rhdl :: bits :: bits :: < W3 > (3i64 as u128) . trace () ; v . extend (a . trace ()) ; v . extend (b . trace ()) ; v } Self :: Unknown => { rhdl :: bits :: bits :: < W3 > (4i64 as u128) . trace () } } ; raw . resize (Self :: TRACE_BITS , rhdl :: core :: TraceBit :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 3usize) } fn discriminant (self) -> rhdl :: core :: TypedBits { match self { Self :: A => { rhdl :: bits :: bits :: < W3 > (1i64 as u128) . typed_bits () } Self :: B (_0) => { rhdl :: bits :: bits :: < W3 > (2i64 as u128) . typed_bits () } Self :: C { a , b } => { rhdl :: bits :: bits :: < W3 > (3i64 as u128) . typed_bits () } Self :: Unknown => { rhdl :: bits :: bits :: < W3 > (4i64 as u128) . typed_bits () } } } fn variant_kind (self) -> rhdl :: core :: Kind { match self { Self :: A => { rhdl :: core :: Kind :: Empty } Self :: B (_0) => { rhdl :: core :: Kind :: make_tuple (vec ! [< Bits :: < 16 > as rhdl :: core :: Digital > :: static_kind ()]) } Self :: C { a , b } => { rhdl :: core :: Kind :: make_struct (stringify ! (_Test__C) , vec ! [rhdl :: core :: Kind :: make_field (stringify ! (a) , < Bits :: < 32 > as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (b) , < Bits :: < 8 > as rhdl :: core :: Digital > :: static_kind ())]) } Self :: Unknown => { rhdl :: core :: Kind :: Empty } } } fn dont_care () -> Self { < Self as Default > :: default () } }"#]];
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
    let expected = expect![[r#"impl core :: marker :: Copy for State { } impl Clone for State { # [inline] fn clone (& self) -> Self { match self { State :: Init => State :: Init , State :: Boot => State :: Boot , State :: Running => State :: Running , State :: Stop => State :: Stop , State :: Boom => State :: Boom , State :: Unknown => State :: Unknown , } } } impl rhdl :: core :: Digital for State { const BITS : usize = 3usize + rhdl :: core :: const_max ! (0_usize , 0_usize , 0_usize , 0_usize , 0_usize , 0_usize) ; const TRACE_BITS : usize = 3usize + rhdl :: core :: const_max ! (0_usize , 0_usize , 0_usize , 0_usize , 0_usize , 0_usize) ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_enum (concat ! (module_path ! () , "::" , stringify ! (State)) , vec ! [rhdl :: core :: Kind :: make_variant (stringify ! (Init) , rhdl :: core :: Kind :: Empty , 0i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Boot) , rhdl :: core :: Kind :: Empty , 1i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Running) , rhdl :: core :: Kind :: Empty , 2i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Stop) , rhdl :: core :: Kind :: Empty , 3i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Boom) , rhdl :: core :: Kind :: Empty , 4i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Unknown) , rhdl :: core :: Kind :: Empty , 5i64)] , rhdl :: core :: Kind :: make_discriminant_layout (3usize , rhdl :: core :: DiscriminantAlignment :: Msb , rhdl :: core :: DiscriminantType :: Unsigned)) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_enum (concat ! (module_path ! () , "::" , stringify ! (State)) , vec ! [rhdl :: rtt :: make_variant (stringify ! (Init) , rhdl :: rtt :: TraceType :: Empty , 0i64) , rhdl :: rtt :: make_variant (stringify ! (Boot) , rhdl :: rtt :: TraceType :: Empty , 1i64) , rhdl :: rtt :: make_variant (stringify ! (Running) , rhdl :: rtt :: TraceType :: Empty , 2i64) , rhdl :: rtt :: make_variant (stringify ! (Stop) , rhdl :: rtt :: TraceType :: Empty , 3i64) , rhdl :: rtt :: make_variant (stringify ! (Boom) , rhdl :: rtt :: TraceType :: Empty , 4i64) , rhdl :: rtt :: make_variant (stringify ! (Unknown) , rhdl :: rtt :: TraceType :: Empty , 5i64)] , rhdl :: rtt :: make_discriminant_layout (3usize , rhdl :: core :: DiscriminantAlignment :: Msb . into () , rhdl :: core :: DiscriminantType :: Unsigned . into ())) } fn bin (self) -> Vec < rhdl :: core :: BitX > { let mut raw = match self { Self :: Init => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (0i64 as u128) . to_bools ()) } Self :: Boot => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (1i64 as u128) . to_bools ()) } Self :: Running => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (2i64 as u128) . to_bools ()) } Self :: Stop => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (3i64 as u128) . to_bools ()) } Self :: Boom => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (4i64 as u128) . to_bools ()) } Self :: Unknown => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W3 > (5i64 as u128) . to_bools ()) } } ; raw . resize (Self :: BITS , rhdl :: core :: BitX :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 3usize) } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { let mut raw = match self { Self :: Init => { rhdl :: bits :: bits :: < W3 > (0i64 as u128) . trace () } Self :: Boot => { rhdl :: bits :: bits :: < W3 > (1i64 as u128) . trace () } Self :: Running => { rhdl :: bits :: bits :: < W3 > (2i64 as u128) . trace () } Self :: Stop => { rhdl :: bits :: bits :: < W3 > (3i64 as u128) . trace () } Self :: Boom => { rhdl :: bits :: bits :: < W3 > (4i64 as u128) . trace () } Self :: Unknown => { rhdl :: bits :: bits :: < W3 > (5i64 as u128) . trace () } } ; raw . resize (Self :: TRACE_BITS , rhdl :: core :: TraceBit :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 3usize) } fn discriminant (self) -> rhdl :: core :: TypedBits { match self { Self :: Init => { rhdl :: bits :: bits :: < W3 > (0i64 as u128) . typed_bits () } Self :: Boot => { rhdl :: bits :: bits :: < W3 > (1i64 as u128) . typed_bits () } Self :: Running => { rhdl :: bits :: bits :: < W3 > (2i64 as u128) . typed_bits () } Self :: Stop => { rhdl :: bits :: bits :: < W3 > (3i64 as u128) . typed_bits () } Self :: Boom => { rhdl :: bits :: bits :: < W3 > (4i64 as u128) . typed_bits () } Self :: Unknown => { rhdl :: bits :: bits :: < W3 > (5i64 as u128) . typed_bits () } } } fn variant_kind (self) -> rhdl :: core :: Kind { match self { Self :: Init => { rhdl :: core :: Kind :: Empty } Self :: Boot => { rhdl :: core :: Kind :: Empty } Self :: Running => { rhdl :: core :: Kind :: Empty } Self :: Stop => { rhdl :: core :: Kind :: Empty } Self :: Boom => { rhdl :: core :: Kind :: Empty } Self :: Unknown => { rhdl :: core :: Kind :: Empty } } } fn dont_care () -> Self { < Self as Default > :: default () } }"#]];
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
    let expected = expect![[r#"impl core :: marker :: Copy for Test { } impl Clone for Test { # [inline] fn clone (& self) -> Self { match self { Test :: A => Test :: A , Test :: B => Test :: B , Test :: C => Test :: C , Test :: Unknown => Test :: Unknown , } } } impl rhdl :: core :: Digital for Test { const BITS : usize = 5usize + rhdl :: core :: const_max ! (0_usize , 0_usize , 0_usize , 0_usize) ; const TRACE_BITS : usize = 5usize + rhdl :: core :: const_max ! (0_usize , 0_usize , 0_usize , 0_usize) ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_enum (concat ! (module_path ! () , "::" , stringify ! (Test)) , vec ! [rhdl :: core :: Kind :: make_variant (stringify ! (A) , rhdl :: core :: Kind :: Empty , 1i64) , rhdl :: core :: Kind :: make_variant (stringify ! (B) , rhdl :: core :: Kind :: Empty , 9i64) , rhdl :: core :: Kind :: make_variant (stringify ! (C) , rhdl :: core :: Kind :: Empty , - 8i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Unknown) , rhdl :: core :: Kind :: Empty , - 7i64)] , rhdl :: core :: Kind :: make_discriminant_layout (5usize , rhdl :: core :: DiscriminantAlignment :: Msb , rhdl :: core :: DiscriminantType :: Signed)) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_enum (concat ! (module_path ! () , "::" , stringify ! (Test)) , vec ! [rhdl :: rtt :: make_variant (stringify ! (A) , rhdl :: rtt :: TraceType :: Empty , 1i64) , rhdl :: rtt :: make_variant (stringify ! (B) , rhdl :: rtt :: TraceType :: Empty , 9i64) , rhdl :: rtt :: make_variant (stringify ! (C) , rhdl :: rtt :: TraceType :: Empty , - 8i64) , rhdl :: rtt :: make_variant (stringify ! (Unknown) , rhdl :: rtt :: TraceType :: Empty , - 7i64)] , rhdl :: rtt :: make_discriminant_layout (5usize , rhdl :: core :: DiscriminantAlignment :: Msb . into () , rhdl :: core :: DiscriminantType :: Signed . into ())) } fn bin (self) -> Vec < rhdl :: core :: BitX > { let mut raw = match self { Self :: A => { rhdl :: core :: bitx_vec (& rhdl :: bits :: signed :: < W5 > (1i64 as i128) . to_bools ()) } Self :: B => { rhdl :: core :: bitx_vec (& rhdl :: bits :: signed :: < W5 > (9i64 as i128) . to_bools ()) } Self :: C => { rhdl :: core :: bitx_vec (& rhdl :: bits :: signed :: < W5 > (- 8i64 as i128) . to_bools ()) } Self :: Unknown => { rhdl :: core :: bitx_vec (& rhdl :: bits :: signed :: < W5 > (- 7i64 as i128) . to_bools ()) } } ; raw . resize (Self :: BITS , rhdl :: core :: BitX :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 5usize) } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { let mut raw = match self { Self :: A => { rhdl :: bits :: signed :: < W5 > (1i64 as i128) . trace () } Self :: B => { rhdl :: bits :: signed :: < W5 > (9i64 as i128) . trace () } Self :: C => { rhdl :: bits :: signed :: < W5 > (- 8i64 as i128) . trace () } Self :: Unknown => { rhdl :: bits :: signed :: < W5 > (- 7i64 as i128) . trace () } } ; raw . resize (Self :: TRACE_BITS , rhdl :: core :: TraceBit :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 5usize) } fn discriminant (self) -> rhdl :: core :: TypedBits { match self { Self :: A => { rhdl :: bits :: signed :: < W5 > (1i128) . typed_bits () } Self :: B => { rhdl :: bits :: signed :: < W5 > (9i128) . typed_bits () } Self :: C => { rhdl :: bits :: signed :: < W5 > (- 8i128) . typed_bits () } Self :: Unknown => { rhdl :: bits :: signed :: < W5 > (- 7i128) . typed_bits () } } } fn variant_kind (self) -> rhdl :: core :: Kind { match self { Self :: A => { rhdl :: core :: Kind :: Empty } Self :: B => { rhdl :: core :: Kind :: Empty } Self :: C => { rhdl :: core :: Kind :: Empty } Self :: Unknown => { rhdl :: core :: Kind :: Empty } } } fn dont_care () -> Self { < Self as Default > :: default () } }"#]];
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
    let expected = expect![[r#"impl core :: marker :: Copy for Test { } impl Clone for Test { # [inline] fn clone (& self) -> Self { match self { Test :: A => Test :: A , Test :: B => Test :: B , Test :: C => Test :: C , Test :: Unknown => Test :: Unknown , } } } impl rhdl :: core :: Digital for Test { const BITS : usize = 4usize + rhdl :: core :: const_max ! (0_usize , 0_usize , 0_usize , 0_usize) ; const TRACE_BITS : usize = 4usize + rhdl :: core :: const_max ! (0_usize , 0_usize , 0_usize , 0_usize) ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_enum (concat ! (module_path ! () , "::" , stringify ! (Test)) , vec ! [rhdl :: core :: Kind :: make_variant (stringify ! (A) , rhdl :: core :: Kind :: Empty , 1i64) , rhdl :: core :: Kind :: make_variant (stringify ! (B) , rhdl :: core :: Kind :: Empty , 6i64) , rhdl :: core :: Kind :: make_variant (stringify ! (C) , rhdl :: core :: Kind :: Empty , 8i64) , rhdl :: core :: Kind :: make_variant (stringify ! (Unknown) , rhdl :: core :: Kind :: Empty , 9i64)] , rhdl :: core :: Kind :: make_discriminant_layout (4usize , rhdl :: core :: DiscriminantAlignment :: Msb , rhdl :: core :: DiscriminantType :: Unsigned)) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_enum (concat ! (module_path ! () , "::" , stringify ! (Test)) , vec ! [rhdl :: rtt :: make_variant (stringify ! (A) , rhdl :: rtt :: TraceType :: Empty , 1i64) , rhdl :: rtt :: make_variant (stringify ! (B) , rhdl :: rtt :: TraceType :: Empty , 6i64) , rhdl :: rtt :: make_variant (stringify ! (C) , rhdl :: rtt :: TraceType :: Empty , 8i64) , rhdl :: rtt :: make_variant (stringify ! (Unknown) , rhdl :: rtt :: TraceType :: Empty , 9i64)] , rhdl :: rtt :: make_discriminant_layout (4usize , rhdl :: core :: DiscriminantAlignment :: Msb . into () , rhdl :: core :: DiscriminantType :: Unsigned . into ())) } fn bin (self) -> Vec < rhdl :: core :: BitX > { let mut raw = match self { Self :: A => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W4 > (1i64 as u128) . to_bools ()) } Self :: B => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W4 > (6i64 as u128) . to_bools ()) } Self :: C => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W4 > (8i64 as u128) . to_bools ()) } Self :: Unknown => { rhdl :: core :: bitx_vec (& rhdl :: bits :: bits :: < W4 > (9i64 as u128) . to_bools ()) } } ; raw . resize (Self :: BITS , rhdl :: core :: BitX :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 4usize) } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { let mut raw = match self { Self :: A => { rhdl :: bits :: bits :: < W4 > (1i64 as u128) . trace () } Self :: B => { rhdl :: bits :: bits :: < W4 > (6i64 as u128) . trace () } Self :: C => { rhdl :: bits :: bits :: < W4 > (8i64 as u128) . trace () } Self :: Unknown => { rhdl :: bits :: bits :: < W4 > (9i64 as u128) . trace () } } ; raw . resize (Self :: TRACE_BITS , rhdl :: core :: TraceBit :: Zero) ; rhdl :: core :: move_nbits_to_msb (& raw , 4usize) } fn discriminant (self) -> rhdl :: core :: TypedBits { match self { Self :: A => { rhdl :: bits :: bits :: < W4 > (1i64 as u128) . typed_bits () } Self :: B => { rhdl :: bits :: bits :: < W4 > (6i64 as u128) . typed_bits () } Self :: C => { rhdl :: bits :: bits :: < W4 > (8i64 as u128) . typed_bits () } Self :: Unknown => { rhdl :: bits :: bits :: < W4 > (9i64 as u128) . typed_bits () } } } fn variant_kind (self) -> rhdl :: core :: Kind { match self { Self :: A => { rhdl :: core :: Kind :: Empty } Self :: B => { rhdl :: core :: Kind :: Empty } Self :: C => { rhdl :: core :: Kind :: Empty } Self :: Unknown => { rhdl :: core :: Kind :: Empty } } } fn dont_care () -> Self { < Self as Default > :: default () } }"#]];
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
