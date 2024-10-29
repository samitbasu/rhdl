use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr};

pub(crate) fn get_fqdn(decl: &DeriveInput) -> TokenStream {
    let struct_name = &decl.ident;
    if decl.generics.type_params().count() > 0 {
        let generics_names = decl
            .generics
            .type_params()
            .map(|x| &x.ident)
            .map(|x| quote!(std::any::type_name::<#x>().to_string()));
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
        &syn::parse_file(&actual)
            .unwrap_or_else(|err| panic!("Actual string is not valid rust code: {actual}  {err}")),
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

pub(crate) fn evaluate_const_expression(expr: &syn::Expr) -> syn::Result<i64> {
    let expr_as_string = quote!(#expr).to_string();
    match evalexpr::eval_int(&expr_as_string) {
        Ok(x) => Ok(x),
        Err(err) => Err(syn::Error::new(
            expr.span(),
            format!("Failed to evaluate expression: {}", err),
        )),
    }
}

pub struct FieldSet<'a> {
    pub(crate) component_name: Vec<syn::Ident>,
    pub(crate) component_ty: Vec<&'a syn::Type>,
}

impl<'a> TryFrom<&'a syn::Fields> for FieldSet<'a> {
    type Error = syn::Error;

    fn try_from(fields: &'a syn::Fields) -> syn::Result<Self> {
        let mut component_name = Vec::new();
        let mut component_ty = Vec::new();
        for field in fields.iter() {
            component_name.push(field.ident.clone().ok_or_else(|| {
                syn::Error::new(
                    field.span(),
                    "Synchronous components (fields) must have names",
                )
            })?);
            component_ty.push(&field.ty);
        }
        Ok(FieldSet {
            component_name,
            component_ty,
        })
    }
}

pub(crate) fn parse_rhdl_skip_attribute(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("rhdl") {
            if let Ok(Expr::Path(path)) = attr.parse_args() {
                if path.path.is_ident("skip") {
                    return true;
                }
            }
        }
    }
    false
}
