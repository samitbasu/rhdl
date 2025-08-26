use quote::{ToTokens, quote};
use serde::Serialize;
use syn::parse::Parse;

use crate::formatter::Pretty;

mod basic;
mod coverage;
mod testbench;

/// Test that a string can be parsed as the given type.
/// Return a `miette` friendly error report if parsing fails.
pub fn test_parse<T: Parse>(text: impl AsRef<str>) -> std::result::Result<T, miette::Report> {
    let text = text.as_ref();
    syn::parse_str::<T>(text).map_err(|err| syn_miette::Error::new(err, text).into())
}

/// Test that a string can be parsed into the given type,
/// and the converted back into a TokenStream.  The
/// resulting TokenStream is returned as a string.
pub fn test_parse_quote<T: Parse + Serialize + ToTokens + Pretty>(
    text: impl AsRef<str>,
) -> std::result::Result<String, miette::Report> {
    let text = text.as_ref();
    let val = syn::parse_str::<T>(text).map_err(|err| syn_miette::Error::new(err, text))?;
    let tokens = quote! {#val};
    let val2 = syn::parse2::<T>(tokens).map_err(|err| syn_miette::Error::new(err, text))?;
    let val = val2.pretty();
    Ok(val)
}
