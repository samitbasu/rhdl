use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr};

pub(crate) fn get_fqdn(decl: &DeriveInput) -> TokenStream {
    let struct_name = &decl.ident;
    if decl.generics.type_params().count() > 0 {
        let mut generics_names = decl
            .generics
            .type_params()
            .map(|x| &x.ident)
            .flat_map(|x| {
                [
                    quote!(std::any::type_name::<#x>().to_string()),
                    quote!(",".to_string()),
                ]
            })
            .collect::<Punctuated<_, syn::Token![,]>>();
        if !generics_names.is_empty() {
            generics_names.pop(); // Remove last comma string
            generics_names.pop_punct(); // Remove last punctuation
        }
        let generics_names = quote!(#generics_names);
        quote!(&vec![module_path!().to_string(), "::".to_string(), stringify!(#struct_name).to_string(), "<".to_string(),  #generics_names, ">".to_string()].join(""))
    } else {
        quote!(concat!(module_path!(), "::", stringify! (#struct_name)))
    }
}

#[cfg(test)]
pub(crate) fn pretty_print(tokens: &proc_macro2::TokenStream) -> String {
    let tokens_str = tokens.to_string();
    prettyplease::unparse(
        &syn::parse_file(&tokens_str)
            .unwrap_or_else(|err| panic!("Tokens are not valid rust code: {tokens_str}  {err}")),
    )
}

pub(crate) fn evaluate_const_expression(expr: &syn::Expr) -> syn::Result<i64> {
    let expr_as_string = quote!(#expr).to_string();
    match evalexpr::eval_int(&expr_as_string) {
        Ok(x) => Ok(x),
        Err(err) => Err(syn::Error::new(
            expr.span(),
            format!("Failed to evaluate expression: {err}"),
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
            if parse_rhdl_skip_attribute(&field.attrs) {
                continue;
            }
            if let Some(name) = &field.ident {
                component_name.push(name.clone());
                component_ty.push(&field.ty);
            }
        }
        Ok(FieldSet {
            component_name,
            component_ty,
        })
    }
}

pub(crate) fn parse_rhdl_skip_attribute(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("rhdl")
            && let Ok(Expr::Path(path)) = attr.parse_args()
            && path.path.is_ident("skip")
        {
            return true;
        }
    }
    false
}

pub(crate) fn parse_dq_no_prefix_attribute(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("rhdl")
            && let Ok(Expr::Path(path)) = attr.parse_args()
            && path.path.is_ident("dq_no_prefix")
        {
            return true;
        }
    }
    false
}
