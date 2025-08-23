use quote::ToTokens;
use quote::quote;
use syn::parse::Parse;

use crate::cst::ModuleList;

/// Test that a string can be parsed as the given type.
/// Return a `miette` friendly error report if parsing fails.
pub fn test_parse<T: Parse>(text: impl AsRef<str>) -> std::result::Result<T, miette::Report> {
    let text = text.as_ref();
    syn::parse_str::<T>(text).map_err(|err| syn_miette::Error::new(err, text).into())
}

/// Test that a string can be parsed into the given type,
/// and the converted back into a TokenStream.  The
/// resulting TokenStream is returned as a string.
pub fn test_parse_quote<T: Parse + ToTokens>(
    text: impl AsRef<str>,
) -> std::result::Result<String, miette::Report> {
    let text = text.as_ref();
    let val = syn::parse_str::<T>(text).map_err(|err| syn_miette::Error::new(err, text))?;
    let quoted = quote! {#val};
    Ok(quoted.to_string())
}

/// Test the compilation of the given `ModuleList`.  Includes the
/// text of `ast.rs` into the compilation unit, formats things
/// with prettyplease, and then uses `trybuild` to test it.
pub fn test_compilation(test_case: &str, module: ModuleList) {
    let test = quote! {
        pub mod vlog {
            include!("../src/ast.rs");
        }
        pub mod formatter {
        include!("../src/formatter.rs");
        }

        fn main() {
            let _ = #module;
        }
    };
    let file: syn::File = syn::parse2(test).unwrap();
    let pretty = prettyplease::unparse(&file);

    // Use CARGO_MANIFEST_DIR to get absolute path to the crate root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable should be set during tests");
    let gen_dir = std::path::Path::new(&manifest_dir).join("gen");

    // Create the gen directory if it doesn't exist
    std::fs::create_dir_all(&gen_dir).unwrap();

    let filename = gen_dir.join(format!("{}.rs", test_case));
    std::fs::write(&filename, pretty).unwrap();
    let builder = trybuild::TestCases::new();
    builder.pass(filename);
}
