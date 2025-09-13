use quote::ToTokens;

#[derive(Clone, Hash, PartialEq)]
pub struct BoolParameter(pub bool);

impl std::fmt::Display for BoolParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 {
            write!(f, "TRUE")
        } else {
            write!(f, "FALSE")
        }
    }
}

impl ToTokens for BoolParameter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = if self.0 { "TRUE" } else { "FALSE" };
        tokens.extend(quote::quote! { #s });
    }
}
