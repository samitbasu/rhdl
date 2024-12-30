use proc_macro2::TokenStream;
use quote::quote;
use syn::LitInt;

pub fn impl_pad_trait(input: TokenStream) -> syn::Result<TokenStream> {
    let bits = syn::parse2::<LitInt>(input)?;
    let bits = bits.base10_parse::<usize>().map_err(|_| {
        syn::Error::new(
            bits.span(),
            "only integers with base 10 are supported for specifying the number of bits",
        )
    })?;
    let output_bits = bits + 1;
    Ok(quote! {
    impl Pad for SignedBits<#bits> {
        type Output = SignedBits<#output_bits>;
        fn pad(self) -> Self::Output {
            self.resize()
        }
    }
    impl Pad for Bits<#bits> {
        type Output = Bits<#output_bits>;
        fn pad(self) -> Self::Output {
            self.resize()
        }
    }
    })
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_impl_pad_trait() {
        let input = quote! {8};
        let expected = expect![[""]];
        let result = impl_pad_trait(input).unwrap();
        expected.assert_eq(&result.to_string());
    }
}
