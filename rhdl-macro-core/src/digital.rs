use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

use crate::digital_enum::derive_digital_enum;

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
                .enumerate()
                .map(|(ndx, _)| syn::Index::from(ndx));
            let fields2 = fields.clone();
            let fields_3 = fields.clone();
            let fields_4 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            let field_types_3 = field_types.clone();
            Ok(quote! {
                impl #impl_generics rhdl_core::Digital for #struct_name #ty_generics #where_clause {
                    fn static_kind() -> rhdl_core::Kind {
                        rhdl_core::Kind::make_struct(
                            #fqdn,
                            vec![
                            #(
                                rhdl_core::Kind::make_field(stringify!(#fields_4), <#field_types_3 as rhdl_core::Digital>::static_kind()),
                            )*
                        ])
                    }
                    fn bin(self) -> Vec<bool> {
                        let mut result = vec![];
                        #(
                            result.extend(self.#fields_3.bin());
                        )*
                        result
                    }
                    fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                        #(
                            rhdl_core::Digital::note(self.#fields2, (key, stringify!(#fields)), &mut writer);
                        )*
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
            let fields = s.fields.iter().map(|field| &field.ident);
            let fields2 = fields.clone();
            let fields_3 = fields.clone();
            let fields_4 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            let field_types_3 = field_types.clone();
            Ok(quote! {
                impl #impl_generics rhdl_core::Digital for #struct_name #ty_generics #where_clause {
                    fn static_kind() -> rhdl_core::Kind {
                        rhdl_core::Kind::make_struct(
                            #fqdn,
                            vec![
                            #(
                                rhdl_core::Kind::make_field(stringify!(#fields_4), <#field_types_3 as rhdl_core::Digital>::static_kind()),
                            )*
                        ])
                    }
                    fn bin(self) -> Vec<bool> {
                        let mut result = vec![];
                        #(
                            result.extend(self.#fields_3.bin());
                        )*
                        result
                    }
                    fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                        #(
                            rhdl_core::Digital::note(&self.#fields2, (key, stringify!(#fields)), &mut writer);
                        )*
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
            impl rhdl_core::Digital for NestedBits {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::make_struct(
                        concat!(module_path!(), stringify!(NestedBits)),
                        vec![
                        rhdl_core::Kind::make_field(stringify!(nest_1), <bool as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(nest_2), <u8 as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(nest_3), <TwoBits as rhdl_core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.nest_1.bin());
                    result.extend(self.nest_2.bin());
                    result.extend(self.nest_3.bin());
                    result
                }
                fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                    rhdl_core::Digital::note(&self.nest_1, (key, stringify!(nest_1)), &mut writer);
                    rhdl_core::Digital::note(&self.nest_2, (key, stringify!(nest_2)), &mut writer);
                    rhdl_core::Digital::note(&self.nest_3, (key, stringify!(nest_3)), &mut writer);
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
            impl rhdl_core::Digital for Inputs {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::make_struct(
                        concat!(module_path!(), stringify!(Inputs)),
                        vec![
                        rhdl_core::Kind::make_field(stringify!(input), <u32 as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(write), <bool as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(read), <bool as rhdl_core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.input.bin());
                    result.extend(self.write.bin());
                    result.extend(self.read.bin());
                    result
                }
                fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                    rhdl_core::Digital::note(&self.input, (key, stringify!(input)), &mut writer);
                    rhdl_core::Digital::note(&self.write, (key, stringify!(write)), &mut writer);
                    rhdl_core::Digital::note(&self.read, (key, stringify!(read)), &mut writer);
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
            impl<T: Digital> rhdl_core::Digital for Inputs<T> {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::make_struct(
                        &vec![
                            module_path!().to_string(),
                            stringify!(Inputs).to_string(), "<".to_string(), <T as rhdl_core::Digital>::static_kind().get_name(), ">".to_string()
                        ].join(""),
                        vec![
                        rhdl_core::Kind::make_field(stringify!(input), <T as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(write), <bool as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(read), <bool as rhdl_core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.input.bin());
                    result.extend(self.write.bin());
                    result.extend(self.read.bin());
                    result
                }
                fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                    rhdl_core::Digital::note(&self.input, (key, stringify!(input)), &mut writer);
                    rhdl_core::Digital::note(&self.write, (key, stringify!(write)), &mut writer);
                    rhdl_core::Digital::note(&self.read, (key, stringify!(read)), &mut writer);
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
            impl rhdl_core::Digital for Inputs {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::make_struct(
                        concat!(module_path!(), stringify!(Inputs)),
                        vec![
                        rhdl_core::Kind::make_field(stringify!(input), <u32 as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(write), <bool as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(read), <(bool, bool) as rhdl_core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.input.bin());
                    result.extend(self.write.bin());
                    result.extend(self.read.bin());
                    result
                }
                fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                    rhdl_core::Digital::note(&self.input, (key, stringify!(input)), &mut writer);
                    rhdl_core::Digital::note(&self.write, (key, stringify!(write)), &mut writer);
                    rhdl_core::Digital::note(&self.read, (key, stringify!(read)), &mut writer);
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
            impl rhdl_core::Digital for Inputs {
                fn static_kind() -> rhdl_core::Kind {
                    rhdl_core::Kind::make_struct(
                        concat!(module_path!(), stringify!(Inputs)),
                        vec![
                        rhdl_core::Kind::make_field(stringify!(0), <u32 as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(1), <bool as rhdl_core::Digital>::static_kind()),
                        rhdl_core::Kind::make_field(stringify!(2), <bool as rhdl_core::Digital>::static_kind()),
                    ])
                }
                fn bin(self) -> Vec<bool> {
                    let mut result = vec![];
                    result.extend(self.0.bin());
                    result.extend(self.1.bin());
                    result.extend(self.2.bin());
                    result
                }
                fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                    rhdl_core::Digital::note(self.0, (key, stringify!(0)), &mut writer);
                    rhdl_core::Digital::note(self.1, (key, stringify!(1)), &mut writer);
                    rhdl_core::Digital::note(self.2, (key, stringify!(2)), &mut writer);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }
}
