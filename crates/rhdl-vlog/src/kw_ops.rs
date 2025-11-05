#![allow(missing_docs)]
/// Operators and keywords used in Verilog HDL.
///
/// This module defines various operators and keywords used in Verilog HDL,
/// along with their parsing and pretty-printing implementations.
use quote::{ToTokens, quote};
use serde::{Deserialize, Serialize};
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};

use crate::formatter::{Formatter, Pretty};

syn::custom_punctuation!(PlusColon, +:);

syn::custom_punctuation!(MinusColon, -:);
syn::custom_punctuation!(LeftArrow, <=);

syn::custom_punctuation!(CaseUnequal, !==);
syn::custom_punctuation!(CaseEqual, ===);

syn::custom_punctuation!(SignedRightShift, >>>);

pub(crate) mod kw {
    syn::custom_keyword!(input);
    syn::custom_keyword!(output);
    syn::custom_keyword!(inout);
    syn::custom_keyword!(reg);
    syn::custom_keyword!(wire);
    syn::custom_keyword!(signed);
    syn::custom_keyword!(assign);
    syn::custom_keyword!(always);
    syn::custom_keyword!(negedge);
    syn::custom_keyword!(posedge);
    syn::custom_keyword!(localparam);
    syn::custom_keyword!(begin);
    syn::custom_keyword!(end);
    syn::custom_keyword!(function);
    syn::custom_keyword!(endfunction);
    syn::custom_keyword!(case);
    syn::custom_keyword!(endcase);
    syn::custom_keyword!(default);
    syn::custom_keyword!(module);
    syn::custom_keyword!(endmodule);
    syn::custom_keyword!(initial);
    syn::custom_keyword!(doc);
}

/// Dynamic indexing operators in Verilog HDL.
#[derive(Copy, Clone, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub enum DynOp {
    /// `+:` operator
    PlusColon,
    /// `-:` operator
    MinusColon,
}

impl Parse for DynOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(PlusColon) {
            let _: PlusColon = input.parse()?;
            Ok(DynOp::PlusColon)
        } else if input.peek(MinusColon) {
            let _: MinusColon = input.parse()?;
            Ok(DynOp::MinusColon)
        } else if input.peek(Token![+]) && input.peek2(Token![:]) {
            let _: Token![+] = input.parse()?;
            let _: Token![:] = input.parse()?;
            Ok(DynOp::PlusColon)
        } else if input.peek(Token![-]) && input.peek2(Token![:]) {
            let _: Token![-] = input.parse()?;
            let _: Token![:] = input.parse()?;
            Ok(DynOp::MinusColon)
        } else {
            Err(input.error("expected dynamic operator"))
        }
    }
}

impl Pretty for DynOp {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            DynOp::PlusColon => formatter.write("+:"),
            DynOp::MinusColon => formatter.write("-:"),
        }
    }
}

impl ToTokens for DynOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DynOp::PlusColon => {
                let op = PlusColon::default();
                tokens.extend(quote! { #op });
            }
            DynOp::MinusColon => {
                let op = MinusColon::default();
                tokens.extend(quote! { #op });
            }
        }
    }
}

/// Unary operators in Verilog HDL.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// `+` operator
    Plus,
    /// `-` operator
    Minus,
    /// `!` operator
    Bang,
    /// `~` operator
    Not,
    /// `&` operator
    And,
    /// `|` operator
    Or,
    /// `^` operator
    Xor,
}

impl Parse for UnaryOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![+]) {
            let _: Token![+] = input.parse()?;
            Ok(UnaryOp::Plus)
        } else if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(UnaryOp::Minus)
        } else if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            Ok(UnaryOp::Bang)
        } else if input.peek(Token![~]) {
            let _: Token![~] = input.parse()?;
            Ok(UnaryOp::Not)
        } else if input.peek(Token![&]) {
            let _: Token![&] = input.parse()?;
            Ok(UnaryOp::And)
        } else if input.peek(Token![|]) {
            let _: Token![|] = input.parse()?;
            Ok(UnaryOp::Or)
        } else if input.peek(Token![^]) {
            let _: Token![^] = input.parse()?;
            Ok(UnaryOp::Xor)
        } else {
            Err(input.error(format!(
                "expected unary operator, found {:?}",
                input.fork().parse::<proc_macro2::TokenTree>()
            )))
        }
    }
}

impl Pretty for UnaryOp {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            UnaryOp::Plus => formatter.write("+"),
            UnaryOp::Minus => formatter.write("-"),
            UnaryOp::Bang => formatter.write("!"),
            UnaryOp::Not => formatter.write("~"),
            UnaryOp::And => formatter.write("&"),
            UnaryOp::Or => formatter.write("|"),
            UnaryOp::Xor => formatter.write("^"),
        }
    }
}

impl ToTokens for UnaryOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            UnaryOp::Plus => {
                tokens.extend(quote! { + });
            }
            UnaryOp::Minus => {
                tokens.extend(quote! { - });
            }
            UnaryOp::Bang => {
                tokens.extend(quote! { ! });
            }
            UnaryOp::Not => {
                tokens.extend(quote! { ~ });
            }
            UnaryOp::And => {
                tokens.extend(quote! { & });
            }
            UnaryOp::Or => {
                tokens.extend(quote! { | });
            }
            UnaryOp::Xor => {
                tokens.extend(quote! { ^ });
            }
        }
    }
}

impl UnaryOp {
    /// Returns the binding power of the unary operator.
    pub fn binding_power(&self) -> u8 {
        50
    }
}

/// Binary operators in Verilog HDL.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    /// `<<` operator
    Shl,
    /// `>>>` operator (signed right shift)
    SignedRightShift,
    /// `>>` operator
    Shr,
    /// `&&` operator
    ShortAnd,
    /// `||` operator
    ShortOr,
    /// `===` operator
    CaseEq,
    /// `!==` operator
    CaseNe,
    /// `!=` operator
    Ne,
    /// `==` operator
    Eq,
    /// `>=` operator
    Ge,
    /// `<=` operator
    Le,
    /// `>` operator
    Gt,
    /// `<` operator
    Lt,
    /// `+` operator
    Plus,
    /// `-` operator
    Minus,
    /// `&` operator
    And,
    /// `|` operator
    Or,
    /// `^` operator
    Xor,
    /// `%` operator
    Mod,
    /// `*` operator
    Mul,
}

impl Parse for BinaryOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<<]) {
            let _: Token![<<] = input.parse()?;
            Ok(BinaryOp::Shl)
        } else if lookahead.peek(SignedRightShift) {
            let _: SignedRightShift = input.parse()?;
            Ok(BinaryOp::SignedRightShift)
        } else if lookahead.peek(Token![>>]) {
            let _: Token![>>] = input.parse()?;
            Ok(BinaryOp::Shr)
        } else if lookahead.peek(Token![&&]) {
            let _: Token![&&] = input.parse()?;
            Ok(BinaryOp::ShortAnd)
        } else if lookahead.peek(Token![||]) {
            let _: Token![||] = input.parse()?;
            Ok(BinaryOp::ShortOr)
        } else if lookahead.peek(CaseEqual)
            || (lookahead.peek(Token![==]) && input.peek3(Token![=]))
        {
            if lookahead.peek(Token![==]) {
                let _: Token![==] = input.parse()?;
                let _: Token![=] = input.parse()?;
            } else {
                let _: CaseEqual = input.parse()?;
            }
            Ok(BinaryOp::CaseEq)
        } else if lookahead.peek(CaseUnequal)
            || (lookahead.peek(Token![!=]) && input.peek3(Token![=]))
        {
            if lookahead.peek(Token![!=]) {
                let _: Token![!=] = input.parse()?;
                let _: Token![=] = input.parse()?;
            } else {
                let _: CaseUnequal = input.parse()?;
            }
            Ok(BinaryOp::CaseNe)
        } else if lookahead.peek(Token![!=]) {
            let _: Token![!=] = input.parse()?;
            Ok(BinaryOp::Ne)
        } else if lookahead.peek(Token![==]) {
            let _: Token![==] = input.parse()?;
            Ok(BinaryOp::Eq)
        } else if lookahead.peek(Token![>=]) {
            let _: Token![>=] = input.parse()?;
            Ok(BinaryOp::Ge)
        } else if lookahead.peek(Token![<=]) {
            let _: Token![<=] = input.parse()?;
            Ok(BinaryOp::Le)
        } else if lookahead.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            Ok(BinaryOp::Gt)
        } else if lookahead.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            Ok(BinaryOp::Lt)
        } else if lookahead.peek(Token![+]) && !input.peek2(Token![:]) {
            let _: Token![+] = input.parse()?;
            Ok(BinaryOp::Plus)
        } else if lookahead.peek(Token![-]) && !input.peek2(Token![:]) {
            let _: Token![-] = input.parse()?;
            Ok(BinaryOp::Minus)
        } else if lookahead.peek(Token![&]) {
            let _: Token![&] = input.parse()?;
            Ok(BinaryOp::And)
        } else if lookahead.peek(Token![|]) {
            let _: Token![|] = input.parse()?;
            Ok(BinaryOp::Or)
        } else if lookahead.peek(Token![^]) {
            let _: Token![^] = input.parse()?;
            Ok(BinaryOp::Xor)
        } else if lookahead.peek(Token![%]) {
            let _: Token![%] = input.parse()?;
            Ok(BinaryOp::Mod)
        } else if lookahead.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            Ok(BinaryOp::Mul)
        } else {
            Err(input.error("expected binary operator"))
        }
    }
}

impl Pretty for BinaryOp {
    fn pretty_print(&self, formatter: &mut Formatter) {
        match self {
            BinaryOp::Shl => formatter.write("<<"),
            BinaryOp::SignedRightShift => formatter.write(">>>"),
            BinaryOp::Shr => formatter.write(">>"),
            BinaryOp::ShortAnd => formatter.write("&&"),
            BinaryOp::ShortOr => formatter.write("||"),
            BinaryOp::CaseEq => formatter.write("==="),
            BinaryOp::CaseNe => formatter.write("!=="),
            BinaryOp::Ne => formatter.write("!="),
            BinaryOp::Eq => formatter.write("=="),
            BinaryOp::Ge => formatter.write(">="),
            BinaryOp::Le => formatter.write("<="),
            BinaryOp::Gt => formatter.write(">"),
            BinaryOp::Lt => formatter.write("<"),
            BinaryOp::Plus => formatter.write("+"),
            BinaryOp::Minus => formatter.write("-"),
            BinaryOp::And => formatter.write("&"),
            BinaryOp::Or => formatter.write("|"),
            BinaryOp::Xor => formatter.write("^"),
            BinaryOp::Mod => formatter.write("%"),
            BinaryOp::Mul => formatter.write("*"),
        }
    }
}

impl ToTokens for BinaryOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            BinaryOp::Shl => {
                tokens.extend(quote! { << });
            }
            BinaryOp::SignedRightShift => {
                let op = SignedRightShift::default();
                tokens.extend(quote! { #op });
            }
            BinaryOp::Shr => {
                tokens.extend(quote! { >> });
            }
            BinaryOp::ShortAnd => {
                tokens.extend(quote! { && });
            }
            BinaryOp::ShortOr => {
                tokens.extend(quote! { || });
            }
            BinaryOp::CaseEq => {
                let op = CaseEqual::default();
                tokens.extend(quote! { #op });
            }
            BinaryOp::CaseNe => {
                let op = CaseUnequal::default();
                tokens.extend(quote! { #op });
            }
            BinaryOp::Ne => {
                tokens.extend(quote! { != });
            }
            BinaryOp::Eq => {
                tokens.extend(quote! { == });
            }
            BinaryOp::Ge => {
                tokens.extend(quote! { >= });
            }
            BinaryOp::Le => {
                tokens.extend(quote! { <= });
            }
            BinaryOp::Gt => {
                tokens.extend(quote! { > });
            }
            BinaryOp::Lt => {
                tokens.extend(quote! { < });
            }
            BinaryOp::Plus => {
                tokens.extend(quote! { + });
            }
            BinaryOp::Minus => {
                tokens.extend(quote! { - });
            }
            BinaryOp::And => {
                tokens.extend(quote! { & });
            }
            BinaryOp::Or => {
                tokens.extend(quote! { | });
            }
            BinaryOp::Xor => {
                tokens.extend(quote! { ^ });
            }
            BinaryOp::Mod => {
                tokens.extend(quote! { % });
            }
            BinaryOp::Mul => {
                tokens.extend(quote! { * });
            }
        }
    }
}

impl BinaryOp {
    /// Returns the binding power of the binary operator.
    /// The first element of the tuple is the left binding power,
    /// and the second element is the right binding power.
    pub fn binding_power(&self) -> (u8, u8) {
        match self {
            BinaryOp::Mod | BinaryOp::Mul => (20, 21),
            BinaryOp::Plus | BinaryOp::Minus => (18, 19),
            BinaryOp::Shl | BinaryOp::Shr | BinaryOp::SignedRightShift => (16, 17),
            BinaryOp::Ge | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Lt => (14, 15),
            BinaryOp::Ne | BinaryOp::Eq | BinaryOp::CaseNe | BinaryOp::CaseEq => (12, 13),
            BinaryOp::And => (10, 11),
            BinaryOp::Xor => (9, 10),
            BinaryOp::Or => (7, 8),
            BinaryOp::ShortAnd => (5, 6),
            BinaryOp::ShortOr => (3, 4),
        }
    }
}
