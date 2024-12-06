use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

use crate::{
    clone::derive_clone_from_inner, digital_enum::derive_digital_enum,
    partial_eq::derive_partial_eq_from_inner, utils::parse_rhdl_skip_attribute,
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
    let partial_eq = derive_partial_eq_from_inner(decl.clone())?;
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

                #partial_eq

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
    let partial_eq = derive_partial_eq_from_inner(decl.clone())?;
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

                #partial_eq

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
    use expect_test::expect;

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
        let expected = expect![[r#"impl core :: marker :: Copy for NestedBits { } impl Clone for NestedBits { # [inline] fn clone (& self) -> Self { Self { nest_1 : self . nest_1 . clone () , nest_2 : self . nest_2 . clone () , nest_3 : self . nest_3 . clone () , } } } impl PartialEq for NestedBits { # [inline] fn eq (& self , other : & Self) -> bool { self . nest_1 . eq (& other . nest_1) && self . nest_2 . eq (& other . nest_2) && self . nest_3 . eq (& other . nest_3) && true } } impl rhdl :: core :: Digital for NestedBits { const BITS : usize = < bool as rhdl :: core :: Digital > :: BITS + < u8 as rhdl :: core :: Digital > :: BITS + < TwoBits as rhdl :: core :: Digital > :: BITS ; const TRACE_BITS : usize = < bool as rhdl :: core :: Digital > :: TRACE_BITS + < u8 as rhdl :: core :: Digital > :: TRACE_BITS + < TwoBits as rhdl :: core :: Digital > :: TRACE_BITS ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_struct (concat ! (module_path ! () , "::" , stringify ! (NestedBits)) , vec ! [rhdl :: core :: Kind :: make_field (stringify ! (nest_1) , < bool as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (nest_2) , < u8 as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (nest_3) , < TwoBits as rhdl :: core :: Digital > :: static_kind ()) ,] ,) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_struct (concat ! (module_path ! () , "::" , stringify ! (NestedBits)) , vec ! [rhdl :: rtt :: make_field (stringify ! (nest_1) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (nest_2) , < u8 as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (nest_3) , < TwoBits as rhdl :: core :: Digital > :: static_trace_type ()) ,] ,) } fn bin (self) -> Vec < rhdl :: core :: BitX > { [self . nest_1 . bin () . as_slice () , self . nest_2 . bin () . as_slice () , self . nest_3 . bin () . as_slice () ,] . concat () } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { [self . nest_1 . trace () . as_slice () , self . nest_2 . trace () . as_slice () , self . nest_3 . trace () . as_slice () ,] . concat () } fn dont_care () -> Self { Self { nest_1 : < bool as rhdl :: core :: Digital > :: dont_care () , nest_2 : < u8 as rhdl :: core :: Digital > :: dont_care () , nest_3 : < TwoBits as rhdl :: core :: Digital > :: dont_care () , } } }"#]];
        expected.assert_eq(&output);
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
        let expected = expect![[r#"impl core :: marker :: Copy for Inputs { } impl Clone for Inputs { # [inline] fn clone (& self) -> Self { Self { input : self . input . clone () , write : self . write . clone () , read : self . read . clone () , } } } impl PartialEq for Inputs { # [inline] fn eq (& self , other : & Self) -> bool { self . input . eq (& other . input) && self . write . eq (& other . write) && self . read . eq (& other . read) && true } } impl rhdl :: core :: Digital for Inputs { const BITS : usize = < u32 as rhdl :: core :: Digital > :: BITS + < bool as rhdl :: core :: Digital > :: BITS + < bool as rhdl :: core :: Digital > :: BITS ; const TRACE_BITS : usize = < u32 as rhdl :: core :: Digital > :: TRACE_BITS + < bool as rhdl :: core :: Digital > :: TRACE_BITS + < bool as rhdl :: core :: Digital > :: TRACE_BITS ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_struct (concat ! (module_path ! () , "::" , stringify ! (Inputs)) , vec ! [rhdl :: core :: Kind :: make_field (stringify ! (input) , < u32 as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (write) , < bool as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (read) , < bool as rhdl :: core :: Digital > :: static_kind ()) ,] ,) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_struct (concat ! (module_path ! () , "::" , stringify ! (Inputs)) , vec ! [rhdl :: rtt :: make_field (stringify ! (input) , < u32 as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (write) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (read) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) ,] ,) } fn bin (self) -> Vec < rhdl :: core :: BitX > { [self . input . bin () . as_slice () , self . write . bin () . as_slice () , self . read . bin () . as_slice () ,] . concat () } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { [self . input . trace () . as_slice () , self . write . trace () . as_slice () , self . read . trace () . as_slice () ,] . concat () } fn dont_care () -> Self { Self { input : < u32 as rhdl :: core :: Digital > :: dont_care () , write : < bool as rhdl :: core :: Digital > :: dont_care () , read : < bool as rhdl :: core :: Digital > :: dont_care () , } } }"#]];
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
        let expected = expect![[r#"impl < T : Digital > core :: marker :: Copy for Inputs < T > { } impl < T : Digital > Clone for Inputs < T > { # [inline] fn clone (& self) -> Self { Self { input : self . input . clone () , write : self . write . clone () , read : self . read . clone () , } } } impl < T : Digital > PartialEq for Inputs < T > { # [inline] fn eq (& self , other : & Self) -> bool { self . input . eq (& other . input) && self . write . eq (& other . write) && self . read . eq (& other . read) && true } } impl < T : Digital > rhdl :: core :: Digital for Inputs < T > { const BITS : usize = < T as rhdl :: core :: Digital > :: BITS + < bool as rhdl :: core :: Digital > :: BITS + < bool as rhdl :: core :: Digital > :: BITS ; const TRACE_BITS : usize = < T as rhdl :: core :: Digital > :: TRACE_BITS + < bool as rhdl :: core :: Digital > :: TRACE_BITS + < bool as rhdl :: core :: Digital > :: TRACE_BITS ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_struct (& vec ! [module_path ! () . to_string () , "::" . to_string () , stringify ! (Inputs) . to_string () , "<" . to_string () , std :: any :: type_name :: < T > () . to_string () , ">" . to_string ()] . join ("") , vec ! [rhdl :: core :: Kind :: make_field (stringify ! (input) , < T as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (write) , < bool as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (read) , < bool as rhdl :: core :: Digital > :: static_kind ()) ,] ,) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_struct (& vec ! [module_path ! () . to_string () , "::" . to_string () , stringify ! (Inputs) . to_string () , "<" . to_string () , std :: any :: type_name :: < T > () . to_string () , ">" . to_string ()] . join ("") , vec ! [rhdl :: rtt :: make_field (stringify ! (input) , < T as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (write) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (read) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) ,] ,) } fn bin (self) -> Vec < rhdl :: core :: BitX > { [self . input . bin () . as_slice () , self . write . bin () . as_slice () , self . read . bin () . as_slice () ,] . concat () } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { [self . input . trace () . as_slice () , self . write . trace () . as_slice () , self . read . trace () . as_slice () ,] . concat () } fn dont_care () -> Self { Self { input : < T as rhdl :: core :: Digital > :: dont_care () , write : < bool as rhdl :: core :: Digital > :: dont_care () , read : < bool as rhdl :: core :: Digital > :: dont_care () , } } }"#]];
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
        let expected = expect![[r#"impl core :: marker :: Copy for Inputs { } impl Clone for Inputs { # [inline] fn clone (& self) -> Self { Self { input : self . input . clone () , write : self . write . clone () , read : self . read . clone () , } } } impl PartialEq for Inputs { # [inline] fn eq (& self , other : & Self) -> bool { self . input . eq (& other . input) && self . write . eq (& other . write) && self . read . eq (& other . read) && true } } impl rhdl :: core :: Digital for Inputs { const BITS : usize = < u32 as rhdl :: core :: Digital > :: BITS + < bool as rhdl :: core :: Digital > :: BITS + < (bool , bool) as rhdl :: core :: Digital > :: BITS ; const TRACE_BITS : usize = < u32 as rhdl :: core :: Digital > :: TRACE_BITS + < bool as rhdl :: core :: Digital > :: TRACE_BITS + < (bool , bool) as rhdl :: core :: Digital > :: TRACE_BITS ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_struct (concat ! (module_path ! () , "::" , stringify ! (Inputs)) , vec ! [rhdl :: core :: Kind :: make_field (stringify ! (input) , < u32 as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (write) , < bool as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (read) , < (bool , bool) as rhdl :: core :: Digital > :: static_kind ()) ,] ,) } fn static_trace_type () -> rhdl :: core :: TraceType { rhdl :: rtt :: make_struct (concat ! (module_path ! () , "::" , stringify ! (Inputs)) , vec ! [rhdl :: rtt :: make_field (stringify ! (input) , < u32 as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (write) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (read) , < (bool , bool) as rhdl :: core :: Digital > :: static_trace_type ()) ,] ,) } fn bin (self) -> Vec < rhdl :: core :: BitX > { [self . input . bin () . as_slice () , self . write . bin () . as_slice () , self . read . bin () . as_slice () ,] . concat () } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { [self . input . trace () . as_slice () , self . write . trace () . as_slice () , self . read . trace () . as_slice () ,] . concat () } fn dont_care () -> Self { Self { input : < u32 as rhdl :: core :: Digital > :: dont_care () , write : < bool as rhdl :: core :: Digital > :: dont_care () , read : < (bool , bool) as rhdl :: core :: Digital > :: dont_care () , } } }"#]];
        expected.assert_eq(&output);
    }

    #[test]
    fn test_digital_with_tuple_struct() {
        let decl = quote!(
            pub struct Inputs(pub u32, pub bool, pub bool);
        );
        let output = derive_digital(decl).unwrap().to_string();
        let expected = expect![[r#"impl core :: marker :: Copy for Inputs { } impl Clone for Inputs { # [inline] fn clone (& self) -> Self { Self (self . 0 . clone () , self . 1 . clone () , self . 2 . clone () ,) } } impl PartialEq for Inputs { # [inline] fn eq (& self , other : & Self) -> bool { self . 0 . eq (& other . 0) && self . 1 . eq (& other . 1) && self . 2 . eq (& other . 2) && true } } impl rhdl :: core :: Digital for Inputs { const BITS : usize = < u32 as rhdl :: core :: Digital > :: BITS + < bool as rhdl :: core :: Digital > :: BITS + < bool as rhdl :: core :: Digital > :: BITS ; const TRACE_BITS : usize = < u32 as rhdl :: core :: Digital > :: TRACE_BITS + < bool as rhdl :: core :: Digital > :: TRACE_BITS + < bool as rhdl :: core :: Digital > :: TRACE_BITS ; fn static_kind () -> rhdl :: core :: Kind { rhdl :: core :: Kind :: make_struct (concat ! (module_path ! () , "::" , stringify ! (Inputs)) , vec ! [rhdl :: core :: Kind :: make_field (stringify ! (0) , < u32 as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (1) , < bool as rhdl :: core :: Digital > :: static_kind ()) , rhdl :: core :: Kind :: make_field (stringify ! (2) , < bool as rhdl :: core :: Digital > :: static_kind ()) ,]) } fn static_trace_type () -> rhdl :: rtt :: TraceType { rhdl :: rtt :: make_struct (concat ! (module_path ! () , "::" , stringify ! (Inputs)) , vec ! [rhdl :: rtt :: make_field (stringify ! (0) , < u32 as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (1) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) , rhdl :: rtt :: make_field (stringify ! (2) , < bool as rhdl :: core :: Digital > :: static_trace_type ()) ,]) } fn bin (self) -> Vec < rhdl :: core :: BitX > { [self . 0 . bin () . as_slice () , self . 1 . bin () . as_slice () , self . 2 . bin () . as_slice () ,] . concat () } fn trace (self) -> Vec < rhdl :: core :: TraceBit > { [self . 0 . trace () . as_slice () , self . 1 . trace () . as_slice () , self . 2 . trace () . as_slice () ,] . concat () } fn dont_care () -> Self { Self (< u32 as rhdl :: core :: Digital > :: dont_care () , < bool as rhdl :: core :: Digital > :: dont_care () , < bool as rhdl :: core :: Digital > :: dont_care () ,) } } impl rhdl :: core :: DigitalFn for Inputs { fn kernel_fn () -> Option < rhdl :: core :: KernelFnKind > { Some (rhdl :: core :: KernelFnKind :: TupleStructConstructor (< Self as rhdl :: core :: Digital > :: static_kind () . place_holder ())) } }"#]];
        expected.assert_eq(&output);
    }
}
