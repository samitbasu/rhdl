// Adapted from the crate `fallacy-clone`
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, Fields};

pub(crate) fn derive_partial_eq_from_inner(ast: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    match &ast.data {
        Data::Struct(data_struct) => {
            let all_fields = match &data_struct.fields {
                Fields::Named(fields) => {
                    let fields = fields.named.iter().map(|field| &field.ident);
                    let fields2 = fields.clone();
                    quote! {
                        #(self.#fields.eq(&other.#fields2) &&)* true
                    }
                }
                Fields::Unnamed(fields) => {
                    let fields = (0..fields.unnamed.len()).map(syn::Index::from);
                    let fields2 = fields.clone();
                    quote! {
                        #(self.#fields.eq(&other.#fields2) &&)* true
                    }
                }
                Fields::Unit => quote!(Self),
            };

            Ok(quote! {
                impl #impl_generics PartialEq for #name #type_generics #where_clause {
                    #[inline]
                    fn eq(&self, other: &Self) -> bool {
                        #all_fields
                    }
                }
            })
        }
        /*

           (
               SimpleEnum::Point {
                   x: x_self,
                   y: y_self,
               },
               SimpleEnum::Point {
                   x: x_other,
                   y: y_other,
               },
           )

        */
        Data::Enum(data_enum) => {
            let all_variants = data_enum.variants.iter().map(|var| match &var.fields {
                Fields::Named(fields) => {
                    let fields = fields
                        .named
                        .iter()
                        .map(|field| field.ident.clone().unwrap())
                        .collect::<Vec<_>>();
                    let fields_renamed_self = fields
                        .iter()
                        .map(|x| format_ident!("{}_self", x))
                        .collect::<Vec<_>>();
                    let fields_renamed_other = fields
                        .iter()
                        .map(|x| format_ident!("{}_other", x))
                        .collect::<Vec<_>>();
                    let variant = &var.ident;
                    quote! {
                        (
                            #name::#variant{#(#fields: #fields_renamed_self,)*},
                            #name::#variant{#(#fields: #fields_renamed_other,)*},
                        ) =>
                        {
                            #(#fields_renamed_self.eq(#fields_renamed_other) &&)*
                            true
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let fields = (0..fields.unnamed.len())
                        .map(|i| {
                            let mut ident = [0];
                            syn::Ident::new(
                                ((b'a' + i as u8) as char).encode_utf8(&mut ident),
                                var.span(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let fields_renamed_self = fields
                        .iter()
                        .map(|x| format_ident!("{}_self", x))
                        .collect::<Vec<_>>();
                    let fields_renamed_other = fields
                        .iter()
                        .map(|x| format_ident!("{}_other", x))
                        .collect::<Vec<_>>();
                    let variant = &var.ident;
                    quote! {
                        (#name::#variant(#(#fields_renamed_self,)*),
                        #name::#variant(#(#fields_renamed_other,)*)) =>
                        {
                            #(#fields_renamed_self.eq(#fields_renamed_other) &&)*
                            true
                        }
                    }
                }
                Fields::Unit => {
                    let variant = &var.ident;
                    quote! {
                        (#name::#variant, #name::#variant) => true,
                    }
                }
            });

            Ok(quote! {
                impl #impl_generics PartialEq for #name #type_generics #where_clause {
                    #[inline]
                    fn eq(&self, other: &Self) -> bool {
                        match (self, other) {
                            #(#all_variants)*
                            _ => false
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(syn::Error::new(
            ast.span(),
            "Deriving PartialEq is not supported for unions",
        )),
    }
}
