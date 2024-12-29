use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, LitInt};

pub fn impl_mul_trait(input: TokenStream) -> syn::Result<TokenStream> {
    let bits = syn::parse2::<LitInt>(input)?;
    let bits = bits.base10_parse::<usize>().map_err(|_| {
        syn::Error::new(
            bits.span(),
            "only integers with base 10 are supported for specifying the number of bits",
        )
    })?;
    let impl_txts = (1..40).map(|arg| {
        let output_width = arg + bits;
        quote! {
            impl Mul<SignedBits<#arg>> for SignedBits<#bits> {
                type Output = SignedBits<#output_width>;
                fn mul(self, rhs: SignedBits<#arg>) -> Self::Output {
                    SignedBits::from(self.0 * rhs.0)
                }
            }

            impl Mul<Bits<#arg>> for Bits<#bits> {
                type Output = Bits<#output_width>;
                fn mul(self, rhs: Bits<#arg>) -> Self::Output {
                    Bits::from(self.0 * rhs.0)
                }
            }
        }
    });
    let impl_txt = quote! {
        #(#impl_txts)*
    };
    Ok(impl_txt)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_impl_mul_trait() {
        let input = quote! { 8 };
        let expected = expect![[""]];
        let result = impl_mul_trait(input).unwrap();
        expected.assert_eq(&result.to_string());
    }
}
