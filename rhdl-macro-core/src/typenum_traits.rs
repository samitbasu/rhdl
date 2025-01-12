use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::LitInt;

fn get_usize_arg(input: TokenStream) -> syn::Result<usize> {
    let bits = syn::parse2::<LitInt>(input)?;
    bits.base10_parse::<usize>().map_err(|_| {
        syn::Error::new(
            bits.span(),
            "only integers with base 10 are supported for specifying the number of bits",
        )
    })
}

fn tn_name(x: usize) -> syn::Ident {
    format_ident!("W{}", x)
}

/// We want to build the table for a trait that
/// is defined computationally.  The only difference
/// between different traits is the name of the trait
/// the name of the function it wraps, and the actual operation.
struct ImplBinOp {
    trait_name: TokenStream,
    func_name: &'static str,
    op: fn(usize, usize) -> Option<usize>,
}

struct ImplUnaryOp {
    trait_name: TokenStream,
    op: fn(usize) -> Option<usize>,
}

fn impl_unary_op(params: ImplUnaryOp, input: TokenStream) -> syn::Result<TokenStream> {
    let bits = get_usize_arg(input)?;
    let impl_txts = if let Some(output) = (params.op)(bits) {
        let target_type_num = tn_name(bits);
        let output_type_num = tn_name(output);
        let trait_name = &params.trait_name;
        Some(quote! {
            impl #trait_name for #target_type_num {
                type Output = #output_type_num;
            }
        })
    } else {
        None
    };
    let impl_txt = quote! {
        #impl_txts
    };
    Ok(impl_txt)
}

fn impl_bin_op(params: ImplBinOp, input: TokenStream) -> syn::Result<TokenStream> {
    let bits = get_usize_arg(input)?;
    let impl_txts = (1..=128).filter_map(|arg| {
        if let Some(output) = (params.op)(bits, arg) {
            let target_type_num = tn_name(bits);
            let rhs_type_num = tn_name(arg);
            let output_type_num = tn_name(output);
            let trait_name = &params.trait_name;
            let func_name = format_ident!("{}", params.func_name);
            Some(quote! {
                impl #trait_name<#rhs_type_num> for #target_type_num {
                    type Output = #output_type_num;
                    fn #func_name(self, _: #rhs_type_num) -> Self::Output {
                        #output_type_num
                    }
                }
            })
        } else {
            None
        }
    });
    let impl_txt = quote! {
        #(#impl_txts)*
    };
    Ok(impl_txt)
}

pub fn impl_add_trait(input: TokenStream) -> syn::Result<TokenStream> {
    fn add(x: usize, y: usize) -> Option<usize> {
        ((x > y) && (x + y <= 128)).then(|| x + y)
    }

    impl_bin_op(
        ImplBinOp {
            trait_name: quote!(std::ops::Add),
            func_name: "add",
            op: add,
        },
        input,
    )
}

pub fn impl_sub_trait(input: TokenStream) -> syn::Result<TokenStream> {
    fn sub(x: usize, y: usize) -> Option<usize> {
        (x > y).then_some(x.saturating_sub(y))
    }

    impl_bin_op(
        ImplBinOp {
            trait_name: quote!(std::ops::Sub),
            func_name: "sub",
            op: sub,
        },
        input,
    )
}

pub fn impl_max_trait(input: TokenStream) -> syn::Result<TokenStream> {
    fn max(x: usize, y: usize) -> Option<usize> {
        if x != y {
            Some(x.max(y))
        } else {
            None
        }
    }

    impl_bin_op(
        ImplBinOp {
            trait_name: quote!(crate::Max),
            func_name: "max",
            op: max,
        },
        input,
    )
}

pub fn impl_min_trait(input: TokenStream) -> syn::Result<TokenStream> {
    fn min(x: usize, y: usize) -> Option<usize> {
        if x != y {
            Some(x.min(y))
        } else {
            None
        }
    }

    impl_bin_op(
        ImplBinOp {
            trait_name: quote!(crate::Min),
            func_name: "min",
            op: min,
        },
        input,
    )
}

pub fn impl_log2_trait(input: TokenStream) -> syn::Result<TokenStream> {
    fn log2(x: usize) -> Option<usize> {
        let mut shift = 0;
        while (1 << shift) < x {
            shift += 1;
        }
        (shift > 0).then_some(shift)
    }

    impl_unary_op(
        ImplUnaryOp {
            trait_name: quote!(typenum::Logarithm2),
            op: log2,
        },
        input,
    )
}
