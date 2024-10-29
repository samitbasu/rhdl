use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Data, DeriveInput, Variant};

use crate::{
    digital_enum::{
        allocate_discriminants, discriminant_kind, override_width,
        parse_discriminant_width_attribute, variant_destructure_args, DiscriminantType,
    },
    utils::evaluate_const_expression,
};

fn variant_note_case(variant: &Variant, kind: DiscriminantType, disc: &i64) -> TokenStream {
    let variant_name = &variant.ident;
    let discriminant = match kind {
        DiscriminantType::Unsigned(x) => {
            let x = x as u8;
            quote! {
                writer.write_bits((key,"__disc"), #disc as u128, #x);
            }
        }
        DiscriminantType::Signed(x) => {
            let x = x as u8;
            quote! {
                writer.write_signed((key,"__disc"), #disc as i128, #x);
            }
        }
    };
    let payloads = match &variant.fields {
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("_{}", i));
            let field_numbers = fields.unnamed.iter().enumerate().map(|(i, _)| i);
            quote! {
                #(
                    rhdl::core::Notable::note(#field_names, (key, #field_numbers), &mut writer);
                )*
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                #(
                    rhdl::core::Notable::note(#field_names, (key, stringify!(#field_names)), &mut writer);
                )*
            }
        }
    };
    quote! {
        writer.write_string(key, stringify!(#variant_name));
        #discriminant
        #payloads
    }
}

pub(crate) fn derive_notable_enum(decl: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let Data::Enum(e) = decl.data else {
        return Err(syn::Error::new(decl.span(), "Only enums can be digital"));
    };
    let variant_names = e.variants.iter().map(|x| &x.ident).collect::<Vec<_>>();
    let variant_destructure_args = e
        .variants
        .iter()
        .map(variant_destructure_args)
        .collect::<Vec<_>>();
    let discriminants: Vec<Option<i64>> = e
        .variants
        .iter()
        .map(|x| {
            x.discriminant
                .as_ref()
                .map(|x| &x.1)
                .map(evaluate_const_expression)
        })
        .map(|x| x.transpose())
        .collect::<Result<Vec<_>, _>>()?;
    let discriminants_values = allocate_discriminants(&discriminants);
    let kind = discriminant_kind(&discriminants_values);
    let width_override = parse_discriminant_width_attribute(&decl.attrs)?;
    let kind = override_width(kind, width_override)?;
    let note_fns = e
        .variants
        .iter()
        .zip(discriminants_values.iter())
        .map(|(variant, discriminant)| variant_note_case(variant, kind, discriminant));
    Ok(quote! {
        impl #impl_generics rhdl::core::Notable for #enum_name #ty_generics #where_clause {
            fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                match self {
                    #(
                        Self::#variant_names #variant_destructure_args => {#note_fns},
                    )*
                }
            }
        }
    })
}
