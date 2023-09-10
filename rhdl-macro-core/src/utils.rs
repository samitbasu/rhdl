#[cfg(test)]
pub(crate) fn assert_tokens_eq(
    expected: &proc_macro2::TokenStream,
    actual: &proc_macro2::TokenStream,
) {
    let expected = expected.to_string();
    let expected = prettyplease::unparse(&syn::parse_file(&expected).unwrap());
    let actual = actual.to_string();
    let actual = prettyplease::unparse(&syn::parse_file(&actual).unwrap());

    if expected != actual {
        println!("expected: {}", expected);
        println!("actual:   {}", actual);
        panic!("expected != actual");
    }
}
