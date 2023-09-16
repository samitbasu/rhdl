use anyhow::anyhow;
use anyhow::bail;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_digital_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|x| &x.ident);
            for variant in &e.variants.clone() {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    bail!("Only unit variants are supported for digital enums")
                }
            }
            Ok(quote! {
                impl #impl_generics rhdl_core::Digital for #enum_name #ty_generics #where_clause {
                    fn allocate<L: rhdl_core::Digital>(tag: rhdl_core::TagID<L>, builder: impl rhdl_core::LogBuilder) {
                        builder.allocate(tag, 0);
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
