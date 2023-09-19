use proc_macro::TokenStream;

#[proc_macro_derive(Digital)]
pub fn digital(input: TokenStream) -> TokenStream {
    rhdl_macro_core::derive_digital(input.into())
        .unwrap()
        .into()
}
