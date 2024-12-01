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

use crate::utils::evaluate_const_expression;

// To determine the number of bits needed to represent the discriminant, we
// need to consider the case where the discriminant is unsigned vs signed.
// For an unsigned discriminant, we simply find the smallest power of two
// larger than the the largest discriminant value.
// For a signed discriminant make sure we test both ends.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiscriminantType {
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
pub(crate) enum DiscriminantAlignment {
    Lsb,
    Msb,
}

pub(crate) fn override_width(
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

pub(crate) fn parse_discriminant_width_attribute(
    attrs: &[Attribute],
) -> syn::Result<Option<(usize, Span)>> {
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

pub(crate) fn discriminant_kind(discriminants: &[i64]) -> DiscriminantType {
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

pub(crate) fn allocate_discriminants(discriminants: &[Option<i64>]) -> Vec<i64> {
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

fn variant_trace_mapping(enum_name: &Ident, variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {rhdl::rtt::TraceType::Empty},
        syn::Fields::Unnamed(fields) => {
            let field_types = fields.unnamed.iter().map(|f| &f.ty);
            quote! {
                rhdl::rtt::make_tuple(vec![#(
                    <#field_types as rhdl::core::Digital>::static_trace_type()
                ),*])
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            let field_types = fields.named.iter().map(|f| &f.ty);
            let struct_name = format_ident!("_{}__{}", enum_name, variant.ident);
            quote! {
                rhdl::rtt::make_struct(
                    stringify!(#struct_name),
                    vec![#(
                        rhdl::rtt::make_field(stringify!(#field_names), <#field_types as rhdl::core::Digital>::static_trace_type())
                    ),*]
                )
            }
        }
    }
}

fn variant_kind_mapping(enum_name: &Ident, variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {rhdl::core::Kind::Empty},
        syn::Fields::Unnamed(fields) => {
            let field_types = fields.unnamed.iter().map(|f| &f.ty);
            quote! {
                rhdl::core::Kind::make_tuple(vec![#(
                    <#field_types as rhdl::core::Digital>::static_kind()
                ),*])
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            let field_types = fields.named.iter().map(|f| &f.ty);
            let struct_name = format_ident!("_{}__{}", enum_name, variant.ident);
            quote! {
                rhdl::core::Kind::make_struct(
                    stringify!(#struct_name),
                    vec![#(
                    rhdl::core::Kind::make_field(stringify!(#field_names), <#field_types as rhdl::core::Digital>::static_kind())
                ),*]
            )
            }
        }
    }
}

fn variant_bits_mapping(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {0_usize},
        syn::Fields::Unnamed(fields) => {
            let field_types = fields.unnamed.iter().map(|f| &f.ty);
            quote! {
                #(<#field_types as rhdl::core::Digital>::BITS)+*
            }
        }
        syn::Fields::Named(fields) => {
            let field_types = fields.named.iter().map(|f| &f.ty);
            quote! {
                #(<#field_types as rhdl::core::Digital>::BITS)+*
            }
        }
    }
}

fn variant_trace_bits_mapping(variant: &Variant) -> TokenStream {
    match &variant.fields {
        syn::Fields::Unit => quote! {0_usize},
        syn::Fields::Unnamed(fields) => {
            let field_types = fields.unnamed.iter().map(|f| &f.ty);
            quote! {
                #(<#field_types as rhdl::core::Digital>::TRACE_BITS)+*
            }
        }
        syn::Fields::Named(fields) => {
            let field_types = fields.named.iter().map(|f| &f.ty);
            quote! {
                #(<#field_types as rhdl::core::Digital>::TRACE_BITS)+*
            }
        }
    }
}

fn make_discriminant_values_into_typed_bits(
    kind: DiscriminantType,
    values: &[i64],
) -> impl Iterator<Item = TokenStream> + '_ {
    values.iter().map(move |x| match kind {
        DiscriminantType::Unsigned(width) => quote! {
            rhdl::bits::bits::<#width>(#x as u128).typed_bits()
        },
        DiscriminantType::Signed(width) => {
            let x = *x as i128;
            quote! {
                rhdl::bits::signed::<#width>(#x).typed_bits()
            }
        }
    })
}

fn variant_payload_trace(
    variant: &Variant,
    kind: DiscriminantType,
    discriminant: i64,
) -> TokenStream {
    let discriminant = match kind {
        DiscriminantType::Unsigned(x) => {
            quote! {
                rhdl::bits::bits::<#x>(#discriminant as u128).trace()
            }
        }
        DiscriminantType::Signed(x) => {
            quote! {
                rhdl::bits::signed::<#x>(#discriminant as i128).trace()
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
                    v.extend(#field_names.trace());
                )*
                v
            }
        }
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);
            quote! {
                let mut v = #discriminant;
                #(
                    v.extend(#field_names.trace());
                )*
                v
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
                rhdl::core::bitx_vec(&rhdl::bits::bits::<#x>(#discriminant as u128).to_bools())
            }
        }
        DiscriminantType::Signed(x) => {
            quote! {
                rhdl::core::bitx_vec(&rhdl::bits::signed::<#x>(#discriminant as i128).to_bools())
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

// Generate the payload destructure arguments used in the
// match
pub(crate) fn variant_destructure_args(variant: &Variant) -> TokenStream {
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
    let discriminant_alignment =
        parse_discriminant_alignment_attribute(&decl.attrs)?.unwrap_or(DiscriminantAlignment::Msb);
    let discriminant_alignment_expr = match parse_discriminant_alignment_attribute(&decl.attrs)?
        .unwrap_or(DiscriminantAlignment::Msb)
    {
        DiscriminantAlignment::Lsb => quote! { rhdl::core::DiscriminantAlignment::Lsb },
        DiscriminantAlignment::Msb => quote! { rhdl::core::DiscriminantAlignment::Msb },
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
    let kind_mapping = e
        .variants
        .iter()
        .map(|v| variant_kind_mapping(enum_name, v));
    let trace_mapping = e
        .variants
        .iter()
        .map(|v| variant_trace_mapping(enum_name, v));
    let variant_kind_mapping = kind_mapping.clone();
    let variant_bits_mapping = e.variants.iter().map(variant_bits_mapping);
    let variant_trace_bits_mapping = e.variants.iter().map(variant_trace_bits_mapping);
    let kind = discriminant_kind(&discriminants_values);
    let width_override = parse_discriminant_width_attribute(&decl.attrs)?;
    let kind = override_width(kind, width_override)?;
    let width_bits = kind.bits();
    let swap_endian_fn = if discriminant_alignment == DiscriminantAlignment::Lsb {
        quote!(raw)
    } else {
        quote!(rhdl::core::move_nbits_to_msb(&raw, #width_bits))
    };
    let discriminants = discriminants_values
        .iter()
        .map(|x| quote! { #x })
        .collect::<Vec<_>>();
    let bin_fns = e
        .variants
        .iter()
        .zip(discriminants_values.iter())
        .map(|(variant, discriminant)| variant_payload_bin(variant, kind, *discriminant));
    let trace_bin_fns = e
        .variants
        .iter()
        .zip(discriminants_values.iter())
        .map(|(variant, discriminant)| variant_payload_trace(variant, kind, *discriminant));
    let discriminants_as_typed_bits =
        make_discriminant_values_into_typed_bits(kind, &discriminants_values);
    let discriminant_ty = match kind {
        DiscriminantType::Unsigned(_) => quote! { rhdl::core::DiscriminantType::Unsigned },
        DiscriminantType::Signed(_) => quote! { rhdl::core::DiscriminantType::Signed },
    };
    Ok(quote! {
        impl #impl_generics rhdl::core::Digital for #enum_name #ty_generics #where_clause {
            // BITS is the width of the discriminant (#width_bits) plus the maximum width
            // of the variant payloads.  This is calculated by taking the maximum width of
            // all the variant payloads and adding #width_bits.
            const BITS: usize = #width_bits + rhdl::core::const_max!(#(#variant_bits_mapping),*);
            const TRACE_BITS: usize = #width_bits + rhdl::core::const_max!(#(#variant_trace_bits_mapping),*);
            fn static_kind() -> rhdl::core::Kind {
                rhdl::core::Kind::make_enum(
                    #fqdn,
                    vec![
                        #(
                            rhdl::core::Kind::make_variant(stringify!(#variant_names), #kind_mapping, #discriminants)
                        ),*
                    ],
                    rhdl::core::Kind::make_discriminant_layout(
                        #width_bits,
                        #discriminant_alignment_expr,
                        #discriminant_ty
                    )
                )
            }
            fn static_trace_type() -> rhdl::core::TraceType {
                rhdl::rtt::make_enum(
                    #fqdn,
                    vec![
                        #(
                            rhdl::rtt::make_variant(stringify!(#variant_names), #trace_mapping, #discriminants)
                        ),*
                    ],
                    rhdl::rtt::make_discriminant_layout(
                        #width_bits,
                        #discriminant_alignment_expr.into(),
                        #discriminant_ty.into()
                    )
                )
            }
            fn bin(self) -> Vec<rhdl::core::BitX> {
                let mut raw =
                    match self {
                    #(
                        Self::#variant_names #variant_destructure_args => {#bin_fns}
                    )*
                };
                raw.resize(Self::BITS, rhdl::core::BitX::Zero);
                #swap_endian_fn
            }
            fn trace(self) -> Vec<rhdl::core::TraceBit> {
                let mut raw =
                match self {
                    #(
                        Self::#variant_names #variant_destructure_args => {#trace_bin_fns}
                    )*
                };
                raw.resize(Self::TRACE_BITS, rhdl::core::TraceBit::Zero);
                #swap_endian_fn
            }
            fn discriminant(self) -> rhdl::core::TypedBits {
                match self {
                    #(
                        Self::#variant_names #variant_destructure_args => {#discriminants_as_typed_bits}
                    )*
                }
            }
            fn variant_kind(self) -> rhdl::core::Kind {
                match self {
                    #(
                        Self::#variant_names #variant_destructure_args => {#variant_kind_mapping}
                    )*
                }
            }
            fn dont_care() -> Self {
                <Self as Default>::default()
            }
        }
    })
}

#[cfg(test)]
mod test;
