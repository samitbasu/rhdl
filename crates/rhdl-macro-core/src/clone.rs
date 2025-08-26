// Adapted from the crate `fallacy-clone`
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, Fields};

pub(crate) fn derive_clone_from_inner(ast: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    match &ast.data {
        Data::Struct(data_struct) => {
            let all_fields = match &data_struct.fields {
                Fields::Named(fields) => {
                    let fields = fields.named.iter().map(|field| &field.ident);
                    quote! {
                        Self {#( #fields: self.#fields.clone(), )*}
                    }
                }
                Fields::Unnamed(fields) => {
                    let fields = (0..fields.unnamed.len()).map(syn::Index::from);
                    quote! {
                        Self(#(self.#fields.clone(),)*)
                    }
                }
                Fields::Unit => quote!(Self),
            };

            Ok(quote! {
                impl #impl_generics Clone for #name #type_generics #where_clause {
                    #[inline]
                    fn clone(&self) -> Self {
                        #all_fields
                    }
                }
            })
        }
        Data::Enum(data_enum) => {
            let all_variants = data_enum.variants.iter().map(|var| match &var.fields {
                Fields::Named(fields) => {
                    let fields = fields.named.iter().map(|field| &field.ident);
                    let fields2 = fields.clone();
                    let fields3 = fields.clone();
                    let variant = &var.ident;
                    quote! {
                        #name::#variant{#(#fields,)*} => #name::#variant {
                            #(#fields2: #fields3.clone(),)*
                        },
                    }
                }
                Fields::Unnamed(fields) => {
                    let fields = (0..fields.unnamed.len()).map(|i| {
                        let mut ident = [0];
                        syn::Ident::new(
                            ((b'a' + i as u8) as char).encode_utf8(&mut ident),
                            var.span(),
                        )
                    });
                    let fields2 = fields.clone();
                    let variant = &var.ident;
                    quote! {
                        #name::#variant(#(#fields,)*) => #name::#variant(
                            #(#fields2.clone(),)*
                        ),
                    }
                }
                Fields::Unit => {
                    let variant = &var.ident;
                    quote! {
                        #name::#variant => #name::#variant,
                    }
                }
            });

            Ok(quote! {
                impl #impl_generics Clone for #name #type_generics #where_clause {
                    #[inline]
                    fn clone(&self) -> Self {
                        match self {
                            #(#all_variants)*
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(syn::Error::new(
            ast.span(),
            "Deriving Clone is not supported for unions",
        )),
    }
}
