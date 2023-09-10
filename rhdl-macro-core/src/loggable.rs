use anyhow::anyhow;
use anyhow::bail;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

pub fn derive_loggable(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_loggable_struct(decl),
        Data::Enum(_e) => derive_loggable_enum(decl),
        _ => Err(anyhow!("Only structs, enums can be derived loggable")),
    }
}

fn derive_loggable_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|x| &x.ident);
            for variant in &e.variants.clone() {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    bail!("Only unit variants are supported for loggable enums")
                }
            }
            Ok(quote! {
                impl #impl_generics rhdl_log::Loggable for #enum_name #ty_generics #where_clause {
                    fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                        builder.allocate(tag, 0);
                    }
                    fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                        match self {
                            #(
                                Self::#variants => logger.write_string(tag, stringify!(#variants)),
                            )*
                        }
                    }
                }
            })
        }
        _ => Err(anyhow!("Only enums can be loggable")),
    }
}

fn derive_loggable_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    match &decl.data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(_) => derive_loggable_named_struct(decl),
            syn::Fields::Unnamed(_) => derive_loggable_tuple_struct(decl),
            syn::Fields::Unit => Err(anyhow!("Unit structs are not loggable")),
        },
        _ => Err(anyhow!("Only structs can be loggable")),
    }
}

fn derive_loggable_tuple_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .enumerate()
                .map(|(ndx, field)| syn::LitInt::new(&ndx.to_string(), field.span()));
            let fields2 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            Ok(quote! {
                impl #impl_generics rhdl_log::Loggable for #struct_name #ty_generics #where_clause {
                    fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                        #(
                            <#field_types as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(#fields)));
                        )*
                    }
                    fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                        #(
                            self.#fields2.record(tag, &mut logger);
                        )*
                    }
                }
            })
        }
        _ => Err(anyhow!("Only structs can be loggable")),
    }
}

fn derive_loggable_named_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s.fields.iter().map(|field| &field.ident);
            let fields2 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            Ok(quote! {
                impl #impl_generics rhdl_log::Loggable for #struct_name #ty_generics #where_clause {
                    fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                        #(
                            <#field_types as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(#fields)));
                        )*
                    }
                    fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                        #(
                            self.#fields2.record(tag, &mut logger);
                        )*
                    }
                }
            })
        }
        _ => Err(anyhow!("Only structs can be loggable")),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::assert_tokens_eq;

    #[test]
    fn test_loggable_enum() {
        let decl = quote!(
            pub enum State {
                Init,
                Boot,
                Running,
                Stop,
                Boom,
            }
        );
        let output = derive_loggable(decl).unwrap();
        let expected = quote! {
            impl rhdl_log::Loggable for State {
                fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                    builder.allocate(tag, 0);
                }
                fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                    match self {
                        Self::Init => logger.write_string(tag, stringify!(Init)),
                        Self::Boot => logger.write_string(tag, stringify!(Boot)),
                        Self::Running => logger.write_string(tag, stringify!(Running)),
                        Self::Stop => logger.write_string(tag, stringify!(Stop)),
                        Self::Boom => logger.write_string(tag, stringify!(Boom)),
                    }
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_loggable_proc_macro() {
        let decl = quote!(
            pub struct NestedBits {
                nest_1: bool,
                nest_2: u8,
                nest_3: TwoBits,
            }
        );
        let output = derive_loggable(decl).unwrap();
        let expected = quote! {
            impl rhdl_log::Loggable for NestedBits {
                fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                    <bool as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(nest_1)));
                    <u8 as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(nest_2)));
                    <TwoBits as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(nest_3)));
                }
                fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                    self.nest_1.record(tag, &mut logger);
                    self.nest_2.record(tag, &mut logger);
                    self.nest_3.record(tag, &mut logger);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_loggable_with_struct() {
        let decl = quote!(
            pub struct Inputs {
                pub input: u32,
                pub write: bool,
                pub read: bool,
            }
        );
        let output = derive_loggable(decl).unwrap();
        let expected = quote! {
            impl rhdl_log::Loggable for Inputs {
                fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                    <u32 as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(input)));
                    <bool as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(write)));
                    <bool as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(read)));
                }

                fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                    self.input.record(tag, &mut logger);
                    self.write.record(tag, &mut logger);
                    self.read.record(tag, &mut logger);
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
        let output = derive_loggable(decl).unwrap();
        let expected = quote! {
            impl rhdl_log::Loggable for Inputs {
                fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                    <u32 as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(input)));
                    <bool as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(write)));
                    <(bool, bool) as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(read)));
                }

                fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                    self.input.record(tag, &mut logger);
                    self.write.record(tag, &mut logger);
                    self.read.record(tag, &mut logger);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }

    #[test]
    fn test_loggable_with_tuple_struct() {
        let decl = quote!(
            pub struct Inputs(pub u32, pub bool, pub bool);
        );
        let output = derive_loggable(decl).unwrap();
        let expected = quote! {
            impl rhdl_log::Loggable for Inputs {
                fn allocate<L: rhdl_log::Loggable>(tag: rhdl_log::TagID<L>, builder: impl rhdl_log::LogBuilder) {
                    <u32 as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(0)));
                    <bool as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(1)));
                    <bool as rhdl_log::Loggable>::allocate(tag, builder.namespace(stringify!(2)));
                }

                fn record<L: rhdl_log::Loggable>(&self, tag: rhdl_log::TagID<L>, mut logger: impl rhdl_log::LoggerImpl) {
                    self.0.record(tag, &mut logger);
                    self.1.record(tag, &mut logger);
                    self.2.record(tag, &mut logger);
                }
            }
        };
        assert_tokens_eq(&expected, &output);
    }
}
