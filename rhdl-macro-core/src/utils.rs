use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn get_fqdn(decl: &DeriveInput) -> TokenStream {
    let struct_name = &decl.ident;
    if decl.generics.type_params().count() > 0 {
        let generics_names = decl
            .generics
            .type_params()
            .map(|x| &x.ident)
            .map(|x| quote!(<#x as rhdl_core::Digital>::static_kind().get_name()));
        quote!(&vec![module_path!().to_string(), "::".to_string(), stringify!(#struct_name).to_string(), "<".to_string(),  #(#generics_names),*, ">".to_string()].join(""))
    } else {
        quote!(concat!(module_path!(), "::", stringify! (#struct_name)))
    }
}

#[cfg(test)]
pub(crate) fn assert_tokens_eq(
    expected: &proc_macro2::TokenStream,
    actual: &proc_macro2::TokenStream,
) {
    let expected = expected.to_string();
    let expected = prettyplease::unparse(
        &syn::parse_file(&expected).expect("Expected string is not valid rust code"),
    );
    let actual = actual.to_string();
    let actual = prettyplease::unparse(
        &syn::parse_file(&actual).expect("Actual string is not valid rust code"),
    );

    if expected != actual {
        println!("expected: {}", expected);
        println!("actual:   {}", actual);
        // Print the lines that are different
        let expected_lines = expected.lines().collect::<Vec<_>>();
        let actual_lines = actual.lines().collect::<Vec<_>>();
        for (i, (expected_line, actual_line)) in
            expected_lines.iter().zip(actual_lines.iter()).enumerate()
        {
            if expected_line != actual_line {
                println!("line {}: expected: {}", i + 1, expected_line);
                println!("line {}: actual:   {}", i + 1, actual_line);
            }
        }
        panic!("expected != actual");
    }
}

#[cfg(test)]
pub(crate) fn assert_frag_eq(
    expected: &proc_macro2::TokenStream,
    actual: &proc_macro2::TokenStream,
) {
    assert_tokens_eq(
        &quote::quote!(fn foo() { #expected }),
        &quote::quote!(fn foo() { #actual }),
    );
}
