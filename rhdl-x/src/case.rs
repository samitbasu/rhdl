use rhdl_bits::alias::{b4, s4};
use rhdl_bits::Bits;
use syn::token::{Brace, Comma, FatArrow};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Arm, Expr,
};
use syn::{Pat, Result};

mod kw {
    syn::custom_keyword!(case);
}

#[derive(Debug)]
pub struct CaseArm {
    pub pat: Pat,
    pub fat_arrow_token: FatArrow,
    pub body: Box<Expr>,
    pub comma: Option<Comma>,
}

impl Parse for CaseArm {
    fn parse(input: ParseStream) -> Result<Self> {
        let pat = Pat::parse_multi(input)?;
        let fat_arrow_token = input.parse()?;
        let body = Box::new(input.parse()?);
        let comma = input.parse().ok();
        Ok(CaseArm {
            pat,
            fat_arrow_token,
            body,
            comma,
        })
    }
}

#[derive(Debug)]
pub struct ExprCase {
    pub case_token: kw::case,
    pub expr: Box<Expr>,
    pub brace_token: Brace,
    pub arms: Vec<CaseArm>,
}

impl Parse for ExprCase {
    fn parse(input: ParseStream) -> Result<Self> {
        let case_token = input.parse()?;
        let expr = Expr::parse_without_eager_brace(input)?;
        let content;
        let brace_token = braced!(content in input);
        let mut arms = Vec::new();
        while !content.is_empty() {
            arms.push(content.parse()?);
        }
        Ok(ExprCase {
            case_token,
            expr: Box::new(expr),
            brace_token,
            arms,
        })
    }
}

#[test]
fn test_expr_of_case() {
    use syn::parse::Parser;
    use syn::parse_quote;
    let input = parse_quote! {
        case x {
            1 => 1,
            2 => 2,
            _ => 3,
        }
    };
    let expr = ExprCase::parse.parse2(input).unwrap();
    assert_eq!(expr.arms.len(), 3);
    eprintln!("{:#?}", expr);
}

/*

Let's accept some more conventional syntax for the case statement:

case!{ x,
    1 => 1,
    2 => 2,
    _ => 3,
}
*/

// The "arms" of the case macro are either literals or a
// wildcard.  For the literal case, we replace the literal
// with a wrapped expression so that `3 => 3` becomes
// `Bits::<_>(3) => 3`.  And the wildcard case is `_ => 3`
// becomes `_ => 3`.
macro_rules! case {
    ($x:expr, $($pat:expr => $expr:expr),* $(,)?, $(_ => $default:expr $(,)?)?) => {
        {
        let x = $x;
        match x {
            $(
                _ if x == $pat => $expr,
            )*
            $(
                _ => $default,
            )?
        }
        }
    };
}

#[test]
fn test_case_macro() {
    let x = 1;
    let y = case! { x,
        1 => 1,
        2 => 2,
        _ => 3,
    };
    assert_eq!(y, 1);
}

#[test]
fn test_match_stuff() {
    let x = b4(4);
    let y = case! {x ,
        1 => b4(1),
        2 => b4(2),
        3 => b4(3),
        _ => b4(0),
    };
    assert_eq!(y, b4(0));
    let x = s4(2);
    let y = case! { x,
        1 => s4(1),
        2 => s4(2),
        3 => s4(-1),
        _ => s4(0),
    };
    assert_eq!(y, s4(2));
    let z = case!(x,
        1 => x + 4,
        2 => x + 3,
        3 => x + 2,
        _ => x + 1,
    );
    assert_eq!(z, s4(5));
}
