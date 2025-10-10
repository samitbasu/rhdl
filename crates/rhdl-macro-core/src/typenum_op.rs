use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

// The op! macro converts an expression like
//   op!(max(A, B) + U4)
// into the equivalent type operators.  So in this case, it would become
//   Sum<Maximum<A, B>, U4>
// The only operations allowed are min, max, add, sub, and the comparison operators >, >=, ==, <=, < and !=.
pub fn typenum_op(input: TokenStream) -> syn::Result<TokenStream> {
    let expr = syn::parse2::<Expr>(input)?;
    handle_expr(&expr)
}

fn handle_expr(expr: &Expr) -> syn::Result<TokenStream> {
    match expr {
        Expr::Binary(expr) => handle_binary_expr(expr),
        Expr::Path(expr) => handle_path_expr(expr),
        Expr::Paren(expr) => handle_expr(&expr.expr),
        Expr::Call(expr) => handle_call(expr),
        _ => Err(syn::Error::new_spanned(
            expr,
            "Only binary expressions are supported".to_string(),
        )),
    }
}

// Handle the min and max functions
fn handle_call(expr: &syn::ExprCall) -> syn::Result<TokenStream> {
    let func = expr.func.as_ref();
    let args = &expr.args;
    let op = match func {
        Expr::Path(expr) => {
            let ident =
                expr.path.get_ident().cloned().ok_or_else(|| {
                    syn::Error::new_spanned(func, "Only identifiers are supported")
                })?;
            match ident.to_string().as_str() {
                "min" => quote::quote! { Minimum },
                "max" => quote::quote! { Maximum },
                _ => {
                    return Err(syn::Error::new_spanned(
                        func,
                        "Only min and max are supported".to_string(),
                    ));
                }
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                func,
                "Only identifiers are supported".to_string(),
            ));
        }
    };
    if args.len() != 2 {
        return Err(syn::Error::new_spanned(
            args,
            "Only two arguments are supported".to_string(),
        ));
    }
    let left = handle_expr(&args[0])?;
    let right = handle_expr(&args[1])?;
    Ok(quote::quote! { #op<#left, #right> })
}

fn handle_binary_expr(expr: &syn::ExprBinary) -> syn::Result<TokenStream> {
    let left = handle_expr(&expr.left)?;
    let right = handle_expr(&expr.right)?;
    let op = match expr.op {
        syn::BinOp::Add(_) => quote::quote! { Sum },
        syn::BinOp::Sub(_) => quote::quote! { Diff },
        syn::BinOp::Gt(_) => quote::quote! {IsGreaterThan},
        syn::BinOp::Lt(_) => quote::quote! {IsLessThan},
        syn::BinOp::Eq(_) => quote::quote! {IsEqualTo},
        syn::BinOp::Ne(_) => quote::quote! {IsNotEqualTo},
        syn::BinOp::Le(_) => quote::quote! {IsLessThanOrEqualTo},
        syn::BinOp::Ge(_) => quote::quote! {IsGreaterThanOrEqualTo},
        _ => {
            return Err(syn::Error::new_spanned(
                expr.op,
                "Only + and - are supported",
            ));
        }
    };
    Ok(quote::quote! { #op<#left, #right> })
}
fn handle_path_expr(expr: &syn::ExprPath) -> syn::Result<TokenStream> {
    let ident = expr
        .path
        .get_ident()
        .ok_or_else(|| syn::Error::new_spanned(expr, "Only identifiers are supported"))?;
    Ok(quote! { #ident })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_typenum_op() {
        let input = parse_quote! { (A - B) + U4 };
        let output = typenum_op(input).unwrap();
        assert_eq!(
            output.to_string(),
            quote! { Sum<Diff<A, B>, U4> }.to_string()
        );
    }

    #[test]
    fn test_order_of_ops() {
        let input = parse_quote! { (A - B) + (C - D - U4)};
        let output = typenum_op(input).unwrap();
        assert_eq!(
            output.to_string(),
            quote! { Sum<Diff<A, B>, Diff<Diff<C, D>, U4> > }.to_string()
        );
    }

    #[test]
    fn test_max_works() {
        let input = parse_quote! { max(A, B) + U4 };
        let output = typenum_op(input).unwrap();
        assert_eq!(
            output.to_string(),
            quote! { Sum<Maximum<A, B>, U4> }.to_string()
        );
    }

    #[test]
    fn test_min_works() {
        let input = parse_quote! { min(A, max(B, U0)) + U4 };
        let output = typenum_op(input).unwrap();
        assert_eq!(
            output.to_string(),
            quote! { Sum<Minimum<A, Maximum<B, U0> >, U4> }.to_string()
        );
    }
}
