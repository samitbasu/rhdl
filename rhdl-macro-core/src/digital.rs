use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Attribute, Data, DeriveInput, Expr};

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

fn parse_rhdl_skip_attribute(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("rhdl") {
            if let Ok(Expr::Path(path)) = attr.parse_args() {
                if path.path.is_ident("skip") {
                    return true;
                }
            }
        }
    }
    false
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
                    fn bin(self) -> Vec<bool> {
                        let mut result = vec![];
                        #(
                            result.extend(self.#fields.bin());
                        )*
                        result
                    }
                    fn random() -> Self {
                        Self(
                            #(
                                <#field_types as rhdl::core::Digital>::random(),
                            )*
                        )
                    }
                }
                impl #impl_generics rhdl::core::Notable for #struct_name #ty_generics #where_clause {
                    fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                        #(
                            rhdl::core::Notable::note(&self.#fields, (key, stringify!(#fields)), &mut writer);
                        )*
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
                    fn bin(self) -> Vec<bool> {
                        let mut result = vec![];
                        #(
                            result.extend(self.#fields.bin());
                        )*
                        result
                    }
                    fn random() -> Self {
                        Self {
                            #(
                                #fields: <#field_types as rhdl::core::Digital>::random(),
                            )*
                        }
                    }
                }

                impl #impl_generics rhdl::core::Notable for #struct_name #ty_generics #where_clause {
                    fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                        #(
                            rhdl::core::Notable::note(&self.#fields, (key, stringify!(#fields)), &mut writer);
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
            impl rhdl::core::Notable for NestedBits {
                fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                    rhdl::core::Notable::note(&self.nest_1, (key, stringify!(nest_1)), &mut writer);
                    rhdl::core::Notable::note(&self.nest_2, (key, stringify!(nest_2)), &mut writer);
                    rhdl::core::Notable::note(&self.nest_3, (key, stringify!(nest_3)), &mut writer);
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
            impl rhdl::core::Notable for Inputs {
                fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                    rhdl::core::Notable::note(&self.input, (key, stringify!(input)), &mut writer);
                    rhdl::core::Notable::note(&self.write, (key, stringify!(write)), &mut writer);
                    rhdl::core::Notable::note(&self.read, (key, stringify!(read)), &mut writer);
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
            impl<T: Digital> rhdl::core::Notable for Inputs<T> {
                fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                    rhdl::core::Notable::note(&self.input, (key, stringify!(input)), &mut writer);
                    rhdl::core::Notable::note(&self.write, (key, stringify!(write)), &mut writer);
                    rhdl::core::Notable::note(&self.read, (key, stringify!(read)), &mut writer);
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
            impl rhdl::core::Notable for Inputs {
                fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                    rhdl::core::Notable::note(&self.input, (key, stringify!(input)), &mut writer);
                    rhdl::core::Notable::note(&self.write, (key, stringify!(write)), &mut writer);
                    rhdl::core::Notable::note(&self.read, (key, stringify!(read)), &mut writer);
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
            impl rhdl::core::Notable for Inputs {
                fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                    rhdl::core::Notable::note(&self.0, (key, stringify!(0)), &mut writer);
                    rhdl::core::Notable::note(&self.1, (key, stringify!(1)), &mut writer);
                    rhdl::core::Notable::note(&self.2, (key, stringify!(2)), &mut writer);
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
