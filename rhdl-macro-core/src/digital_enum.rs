use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Expr;
use syn::ExprLit;
use syn::Ident;
use syn::Lit;
use syn::Variant;
use syn::{Data, DeriveInput};

// To determine the number of bits needed to represent the discriminant, we
// need to consider the case where the discriminant is unsigned vs signed.
// For an unsigned discriminant, we simply find the smallest power of two
// larger than the the largest discriminant value.
// For a signed discriminant make sure we test both ends.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscriminantType {
    Unsigned(usize),
    Signed(usize),
}

impl DiscriminantType {
    fn bits(self) -> usize {
        match self {
            DiscriminantType::Unsigned(x) => x,
            DiscriminantType::Signed(x) => x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscriminantAlignment {
    Lsb,
    Msb,
}

fn override_width(
    width: DiscriminantType,
    new_width: Option<(usize, Span)>,
) -> syn::Result<DiscriminantType> {
    if let Some((new_width, span)) = new_width {
        if new_width == 0 {
            return Err(syn::Error::new(
                span,
                "Override discriminant width cannot be zero",
            ));
        }
        match width {
            DiscriminantType::Unsigned(old_width) => {
                if old_width > new_width {
                    return Err(syn::Error::new(span, format!("Override discriminant width of {new_width} is too small.  At least {old_width} bits are required.")));
                }
                Ok(DiscriminantType::Unsigned(new_width))
            }
            DiscriminantType::Signed(old_width) => {
                if old_width > new_width {
                    return Err(syn::Error::new(span, format!("Override discriminant width of {new_width} is too small.  At least {old_width} bits are required.")));
                }
                Ok(DiscriminantType::Signed(new_width))
            }
        }
    } else {
        Ok(width)
    }
}

fn parse_discriminant_alignment_attribute(
    attrs: &[Attribute],
) -> syn::Result<Option<DiscriminantAlignment>> {
    for attr in attrs {
        if attr.path().is_ident("rhdl") {
            if let Ok(Expr::Assign(assign)) = attr.parse_args::<Expr>() {
                if let Expr::Path(path) = *assign.left {
                    if path.path.is_ident("discriminant_align") {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(value),
                            ..
                        }) = *assign.right
                        {
                            if value.value() == "lsb" {
                                return Ok(Some(DiscriminantAlignment::Lsb));
                            } else if value.value() == "msb" {
                                return Ok(Some(DiscriminantAlignment::Msb));
                            } else {
                                return Err(syn::Error::new(
                                    value.span(),
                                    "Unknown discriminant alignment value (expected either lsb or msb)",
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

fn parse_discriminant_width_attribute(attrs: &[Attribute]) -> syn::Result<Option<(usize, Span)>> {
    for attr in attrs {
        if attr.path().is_ident("rhdl") {
            if let Ok(Expr::Assign(assign)) = attr.parse_args::<Expr>() {
                if let Expr::Path(path) = *assign.left {
                    if path.path.is_ident("discriminant_width") {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Int(value),
                            ..
                        }) = *assign.right
                        {
                            return Ok(Some((value.base10_parse::<usize>()?, value.span())));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

fn discriminant_kind(discriminants: &[i64]) -> DiscriminantType {
    let min = discriminants.iter().min().unwrap();
    let max = discriminants.iter().max().unwrap();
    if *min >= 0 {
        DiscriminantType::Unsigned(clog2(*max as u128 + 1))
    } else {
        let min = *min as i128;
        let max = *max as i128;
        for bit in 1..=127 {
            let min_val = (-1_i128) << (bit - 1);
            let max_val = -min_val - 1;
            if min_val <= min && max <= max_val {
                return DiscriminantType::Signed(bit);
            }
        }
        panic!("Discriminant is too large");
    }
}

fn evaluate_const_expression(expr: &syn::Expr) -> syn::Result<i64> {
    let expr_as_string = quote!(#expr).to_string();
    match evalexpr::eval_int(&expr_as_string) {
        Ok(x) => Ok(x),
        Err(err) => Err(syn::Error::new(
            expr.span(),
            format!("Failed to evaluate expression: {}", err),
        )),
    }
}

fn allocate_discriminants(discriminants: &[Option<i64>]) -> Vec<i64> {
    discriminants
        .iter()
        .scan(0, |state, x| {
            let value;
            if let Some(x) = x {
                value = *x;
                *state = *x + 1;
            } else {
                value = *state;
                *state += 1;
            }
            Some(value)
        })
        .collect()
}

// By convention, fields of a tuple are named _0, _1, ...
fn variant_payload_record(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("_{}", i));
            quote! {
                #(
                    #field_names.record(tag, &mut logger);
                )*
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                #(
                    #field_names.record(tag, &mut logger);
                )*
            }
        }
    }
}

fn variant_kind_mapping(enum_name: &Ident, variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {rhdl_core::Kind::Empty},
        syn::Fields::Unnamed(fields) => {
            let field_types = fields.unnamed.iter().map(|f| &f.ty);
            quote! {
                rhdl_core::Kind::make_tuple(vec![#(
                    <#field_types as rhdl_core::Digital>::static_kind()
                ),*])
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            let field_types = fields.named.iter().map(|f| &f.ty);
            let struct_name = format_ident!("_{}__{}", enum_name, variant.ident);
            quote! {
                rhdl_core::Kind::make_struct(
                    stringify!(#struct_name),
                    vec![#(
                    rhdl_core::Kind::make_field(stringify!(#field_names), <#field_types as rhdl_core::Digital>::static_kind())
                ),*])
            }
        }
    }
}

fn variant_payload_bin(
    variant: &Variant,
    kind: DiscriminantType,
    discriminant: i64,
) -> TokenStream {
    let discriminant = match kind {
        DiscriminantType::Unsigned(x) => {
            quote! {
                rhdl_bits::bits::<#x>(#discriminant as u128).to_bools()
            }
        }
        DiscriminantType::Signed(x) => {
            quote! {
                rhdl_bits::signed::<#x>(#discriminant as i128).to_bools()
            }
        }
    };
    match &variant.fields {
        syn::Fields::Unit => quote! {
            #discriminant
        },
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("_{}", i));
            quote! {
                let mut v = #discriminant;
                #(
                    v.extend(#field_names.bin());
                )*
                v
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                let mut v = #discriminant;
                #(
                    v.extend(#field_names.bin());
                )*
                v
            }
        }
    }
}

fn variant_note_case(variant: &Variant, kind: DiscriminantType, disc: &i64) -> TokenStream {
    let variant_name = &variant.ident;
    let discriminant = match kind {
        DiscriminantType::Unsigned(x) => {
            let x = x as u8;
            quote! {
                writer.write_bits((key,".__disc"), #disc as u128, #x);
            }
        }
        DiscriminantType::Signed(x) => {
            let x = x as u8;
            quote! {
                writer.write_signed((key,".__disc"), #disc as i128, #x);
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
                    rhdl_core::Digital::note(&#field_names, (key, #field_numbers), &mut writer);
                )*
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                #(
                    rhdl_core::Digital::note(&#field_names, (key, stringify!(#field_names)), &mut writer);
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

// Generate the payload destructure arguments used in the
// match
fn variant_destructure_args(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {},
        syn::Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("_{}", i));
            quote! {
                (#(
                    #field_names
                ),*)
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                {
                    #(
                        #field_names
                    ),*
                }
            }
        }
    }
}

pub const fn clog2(t: u128) -> usize {
    let mut p = 0;
    let mut b = 1;
    while b < t {
        p += 1;
        b *= 2;
    }
    p
}

pub fn derive_digital_enum(decl: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &decl.ident;
    let fqdn = crate::utils::get_fqdn(&decl);
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let Data::Enum(e) = decl.data else {
        return Err(syn::Error::new(decl.span(), "Only enums can be digital"));
    };
    let discriminant_alignment = match parse_discriminant_alignment_attribute(&decl.attrs)?
        .unwrap_or(DiscriminantAlignment::Msb)
    {
        DiscriminantAlignment::Lsb => quote! { rhdl_core::DiscriminantAlignment::Lsb },
        DiscriminantAlignment::Msb => quote! { rhdl_core::DiscriminantAlignment::Msb },
    };
    let variants = e.variants.iter().map(|x| &x.ident);
    let variant_destructure_args = e.variants.iter().map(variant_destructure_args);
    let variant_destructure_args_for_bin = variant_destructure_args.clone();
    // For each variant, we need to create the allocate and record functions if the variant has fields
    let kind_mapping = e
        .variants
        .iter()
        .map(|v| variant_kind_mapping(enum_name, v));
    let variant_names_for_kind = variants.clone();
    let variant_names_for_bin = variants.clone();
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
    let width_bits = kind.bits();
    let discriminants = discriminants_values.iter().map(|x| quote! { #x });
    let bin_fns = e
        .variants
        .iter()
        .zip(discriminants_values.iter())
        .map(|(variant, discriminant)| variant_payload_bin(variant, kind, *discriminant));
    Ok(quote! {
        impl #impl_generics rhdl_core::Digital for #enum_name #ty_generics #where_clause {
            fn static_kind() -> rhdl_core::Kind {
                rhdl_core::Kind::make_enum(
                    #fqdn,
                    vec![
                        #(
                            rhdl_core::Kind::make_variant(stringify!(#variant_names_for_kind), #kind_mapping, #discriminants)
                        ),*
                    ],
                    #width_bits,
                    #discriminant_alignment,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind().pad(match self {
                    #(
                        Self::#variant_names_for_bin #variant_destructure_args_for_bin => {#bin_fns}
                    )*
                })

            }
            fn note(&self, key: impl NoteKey, mut writer: impl rhdl_core::NoteWriter) {
                match self {
                    #(
                        Self::#variants #variant_destructure_args => {#note_fns},
                    )*
                }
            }
        }
    })
}

#[cfg(test)]
mod test;
