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
