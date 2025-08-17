use quote::{ToTokens, format_ident, quote};
use syn::{
    Ident, Lifetime, LitInt, LitStr, Result, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Brace, Bracket, Paren},
};

#[cfg(test)]
mod tests;

syn::custom_punctuation!(PlusColon, +:);
syn::custom_punctuation!(LeftArrow, <=);

syn::custom_punctuation!(CaseUnequal, !==);

syn::custom_punctuation!(CaseEqual, ===);

syn::custom_punctuation!(SignedRightShift, >>>);

mod kw {
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
}

#[derive(Debug, Clone, Hash, Copy, PartialEq, syn_derive::Parse)]
enum HDLKind {
    #[parse(peek = kw::wire)]
    Wire(kw::wire),
    #[parse(peek = kw::reg)]
    Reg(kw::reg),
}

impl ToTokens for HDLKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            HDLKind::Reg(_) => quote! {rhdl::vlog::HDLKind::Reg},
            HDLKind::Wire(_) => quote! {rhdl::vlog::HDLKind::Wire},
        })
    }
}

#[derive(Debug, Clone, Hash, Copy, PartialEq, syn_derive::Parse)]
enum Direction {
    #[parse(peek = kw::input)]
    Input(kw::input),
    #[parse(peek = kw::output)]
    Output(kw::output),
    #[parse(peek = kw::inout)]
    Inout(kw::inout),
}

impl ToTokens for Direction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Direction::Input(_) => quote! {rhdl::vlog::Direction::Input},
            Direction::Output(_) => quote! {rhdl::vlog::Direction::Output},
            Direction::Inout(_) => quote! {rhdl::vlog::Direction::Inout},
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, syn_derive::Parse)]
struct BitRange {
    start: LitInt,
    colon: token::Colon,
    end: LitInt,
}

impl ToTokens for BitRange {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let first = &self.end;
        let last = &self.start;
        tokens.extend(quote! {#first..=#last})
    }
}

#[derive(Debug, Clone, Hash, PartialEq, syn_derive::Parse)]
struct WidthSpec {
    #[syn(bracketed)]
    bracket_token: token::Bracket,
    #[syn(in = bracket_token)]
    bit_range: BitRange,
}

impl ToTokens for WidthSpec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let range = &self.bit_range;
        tokens.extend(quote! {#range})
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct SignedWidth {
    signed: Option<kw::signed>,
    width: WidthSpec,
}

impl ToTokens for SignedWidth {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let width = &self.width;
        tokens.extend(if self.signed.is_some() {
            quote! {rhdl::vlog::SignedWidth::Signed(#width)}
        } else {
            quote! {rhdl::vlog::SignedWidth::Unsigned(#width)}
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Declaration {
    kind: HDLKind,
    signed_width: SignedWidth,
    name: Ident,
    _term: Option<Token![;]>,
}

impl ToTokens for Declaration {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let kind = &self.kind;
        let signed_width = &self.signed_width;
        let name = &self.name;
        tokens.extend(quote! {
            rhdl::vlog::Declaration {
                kind: #kind,
                signed_width: #signed_width,
                name: stringify!(#name).into()
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Port {
    direction: Direction,
    decl: Declaration,
}

impl ToTokens for Port {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let direction = &self.direction;
        let decl = &self.decl;
        tokens.extend(quote! {
            rhdl::vlog::Port {
                direction: #direction,
                decl: #decl,
            }
        })
    }
}

#[derive(Default, Debug, Clone)]
enum StmtKind {
    If(If),
    Always(Always),
    Case(Case),
    LocalParam(LocalParam),
    Block(Block),
    ContinuousAssign(ContinuousAssign),
    FunctionCall(FunctionCall),
    NonblockAssign(NonblockAssign),
    Assign(Assign),
    Instance(Instance),
    Splice(Splice),
    DynamicSplice(DynamicSplice),
    Delay(Delay),
    ConcatAssign(ConcatAssign),
    #[default]
    /// Required because the parser for if/else uses it as a placeholder
    Noop,
}

impl ToTokens for StmtKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            StmtKind::If(inner) => quote! {
                rhdl::vlog::Stmt::If(#inner)
            },
            StmtKind::Always(inner) => quote! {
                rhdl::vlog::Stmt::Always(#inner)
            },
            StmtKind::Case(inner) => quote! {
                rhdl::vlog::Stmt::Case(#inner)
            },
            StmtKind::LocalParam(inner) => quote! {
                rhdl::vlog::Stmt::LocalParam(#inner)
            },
            StmtKind::Block(inner) => quote! {
                rhdl::vlog::Stmt::Block(#inner)
            },
            StmtKind::ContinuousAssign(inner) => quote! {
                rhdl::vlog::Stmt::ContinuousAssign(#inner)
            },
            StmtKind::FunctionCall(inner) => quote! {
                rhdl::vlog::Stmt::FunctionCall(#inner)
            },
            StmtKind::NonblockAssign(inner) => quote! {
                rhdl::vlog::Stmt::NonblockAssign(#inner)
            },
            StmtKind::Assign(inner) => quote! {
                rhdl::vlog::Stmt::Assign(#inner)
            },
            StmtKind::Instance(inner) => quote! {
                rhdl::vlog::Stmt::Instance(#inner)
            },
            StmtKind::Splice(inner) => quote! {
                rhdl::vlog::Stmt::Splice(#inner)
            },
            StmtKind::DynamicSplice(inner) => quote! {
                rhdl::vlog::Stmt::DynamicSplice(#inner)
            },
            StmtKind::Delay(inner) => quote! {
                rhdl::vlog::Stmt::Delay(#inner)
            },
            StmtKind::ConcatAssign(inner) => quote! {
                rhdl::vlog::Stmt::ConcatAssign(#inner)
            },
            StmtKind::Noop => quote! {
                rhdl::vlog::Stmt::Noop
            },
        })
    }
}

impl Parse for StmtKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::If) {
            input.parse().map(StmtKind::If)
        } else if lookahead.peek(kw::always) {
            input.parse().map(StmtKind::Always)
        } else if lookahead.peek(kw::case) {
            input.parse().map(StmtKind::Case)
        } else if lookahead.peek(kw::localparam) {
            input.parse().map(StmtKind::LocalParam)
        } else if lookahead.peek(kw::begin) {
            input.parse().map(StmtKind::Block)
        } else if lookahead.peek(kw::assign) {
            input.parse().map(StmtKind::ContinuousAssign)
        } else if lookahead.peek(Token![$]) {
            input.parse().map(StmtKind::FunctionCall)
        } else if lookahead.peek(Ident) && input.peek2(LeftArrow) {
            input.parse().map(StmtKind::NonblockAssign)
        } else if lookahead.peek(Ident) && input.peek2(Token![=]) {
            input.parse().map(StmtKind::Assign)
        } else if lookahead.peek(Ident) && input.peek2(Ident) && input.peek3(token::Paren) {
            input.parse().map(StmtKind::Instance)
        } else if lookahead.peek(Ident) && input.peek2(Bracket) {
            if input.fork().parse::<Splice>().is_ok() {
                input.parse().map(StmtKind::Splice)
            } else {
                input.parse().map(StmtKind::DynamicSplice)
            }
        } else if lookahead.peek(token::Brace) {
            input.parse().map(StmtKind::ConcatAssign)
        } else if lookahead.peek(token::Pound) {
            input.parse().map(StmtKind::Delay)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Default, Clone, syn_derive::Parse)]
struct Stmt {
    kind: StmtKind,
    _terminator: Option<Token![;]>,
}

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.kind.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
enum Item {
    Statement(Stmt),
    Declaration(Declaration),
    FunctionDef(FunctionDef),
    Initial(Initial),
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Item::Statement(stmt) => quote! {
                rhdl::vlog::Item::Statement(#stmt)
            },
            Item::Declaration(decl) => quote! {
                rhdl::vlog::Item::Declaration(#decl)
            },
            Item::FunctionDef(func) => quote! {
                rhdl::vlog::Item::FunctionDef(#func)
            },
            Item::Initial(init) => quote! {
                rhdl::vlog::Item::Initial(#init)
            },
        })
    }
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::function) {
            input.parse().map(Item::FunctionDef)
        } else if input.peek(kw::reg) || input.peek(kw::wire) {
            input.parse().map(Item::Declaration)
        } else if input.peek(kw::initial) {
            input.parse().map(Item::Initial)
        } else {
            input.parse().map(Item::Statement)
        }
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Initial {
    _initial_kw: kw::initial,
    statment: Stmt,
}

impl ToTokens for Initial {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.statment.to_tokens(tokens);
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Delay {
    _hash_token: Token![#],
    length: LitInt,
}

impl ToTokens for Delay {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let length = &self.length;
        tokens.extend(quote! {
            rhdl::vlog::Stmt::Delay(#length)
        })
    }
}

#[derive(Debug, Clone)]
struct FunctionCall {
    dollar: Option<Token![$]>,
    name: Ident,
    args: Option<ParenCommaList<Expr>>,
}

impl ToTokens for FunctionCall {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = if self.dollar.is_some() {
            format_ident!("_{}", self.name)
        } else {
            self.name.clone()
        };
        let args = self.args.iter().flat_map(|x| x.inner.iter());
        tokens.extend(quote! {
            rhdl::vlog::FunctionCall {
                name: #name,
                args: vec![#(#args,)*]
            }
        })
    }
}

impl Parse for FunctionCall {
    fn parse(input: ParseStream) -> Result<Self> {
        let dollar = input.parse()?;
        let name = input.parse()?;
        let args = if input.peek(token::Paren) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(FunctionCall { dollar, name, args })
    }
}

#[derive(Debug, Clone)]
enum CaseItem {
    Literal(Pair<LitVerilog, Token![:]>),
    Wild(Pair<kw::default, Option<Token![:]>>),
}

impl ToTokens for CaseItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            CaseItem::Literal(pair) => {
                let lit = &pair.0;
                tokens.extend(quote! {
                    rhdl::vlog::CaseItem::Literal(#lit)
                });
            }
            CaseItem::Wild(_pair) => {
                tokens.extend(quote! {
                    rhdl::vlog::CaseItem::Wild
                });
            }
        }
    }
}

impl Parse for CaseItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::default) {
            input.parse().map(CaseItem::Wild)
        } else {
            input.parse().map(CaseItem::Literal)
        }
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct CaseLine {
    item: CaseItem,
    stmt: Box<Stmt>,
}

impl ToTokens for CaseLine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item;
        let stmt = &self.stmt;
        tokens.extend(quote! {
            rhdl::vlog::CaseLine {
                item: #item,
                stmt: Box::new(#stmt),
            }
        })
    }
}

struct Pair<S, T>(S, T);

impl<S: Parse, T: Parse> Parse for Pair<S, T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?, input.parse()?))
    }
}

impl<S: Clone, T: Clone> Clone for Pair<S, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<S: std::fmt::Debug, T: std::fmt::Debug> std::fmt::Debug for Pair<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pair").field(&self.0).field(&self.1).finish()
    }
}

impl<S: ToTokens, T: ToTokens> ToTokens for Pair<S, T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
        self.1.to_tokens(tokens);
    }
}

#[derive(syn_derive::Parse)]
struct ParenCommaList<T: Parse + ToTokens> {
    #[syn(parenthesized)]
    paren: Paren,
    #[syn(in = paren)]
    #[parse(Punctuated::parse_terminated)]
    inner: Punctuated<T, Token![,]>,
}

impl<T: Clone + Parse + ToTokens> Clone for ParenCommaList<T> {
    fn clone(&self) -> Self {
        Self {
            paren: self.paren.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + Parse + ToTokens + std::fmt::Debug> std::fmt::Debug for ParenCommaList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParenCommaList")
            .field("paren", &self.paren)
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(syn_derive::Parse)]
struct Parenthesized<T: Parse + ToTokens> {
    #[syn(parenthesized)]
    paren: Paren,
    #[syn(in = paren)]
    inner: T,
}

impl<T: Clone + Parse + ToTokens> Clone for Parenthesized<T> {
    fn clone(&self) -> Self {
        Self {
            paren: self.paren.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T: std::fmt::Debug + Parse + ToTokens> std::fmt::Debug for Parenthesized<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parenthesized")
            .field("paren", &self.paren)
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug, Clone)]
struct Case {
    _case: kw::case,
    _parens: Paren,
    discriminant: Box<Expr>,
    lines: Vec<CaseLine>,
    _endcase: kw::endcase,
}

impl ToTokens for Case {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let discriminant = &self.discriminant;
        let lines = &self.lines;
        tokens.extend(quote! {
            rhdl::vlog::Case {
                discriminant: Box::new(#discriminant),
                lines: vec![#(#lines),*],
            }
        })
    }
}

impl Parse for Case {
    fn parse(input: ParseStream) -> Result<Self> {
        let case = input.parse()?;
        let discriminant;
        let parens = parenthesized!(discriminant in input);
        let discriminant = discriminant.parse()?;
        let mut lines = Vec::new();
        loop {
            if input.peek(kw::endcase) {
                break;
            }
            lines.push(input.parse()?);
        }
        let endcase = input.parse()?;
        Ok(Self {
            _case: case,
            _parens: parens,
            discriminant,
            lines,
            _endcase: endcase,
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Connection {
    _dot: Token![.],
    target: Ident,
    local: Box<Expr>,
}

impl ToTokens for Connection {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let local = &self.local;
        tokens.extend(quote! {
            rhdl::vlog::Connection {
                target: stringify!(#target).into(),
                local: Box::new(#local),
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Instance {
    module: Ident,
    instance: Ident,
    connections: ParenCommaList<Connection>,
}

impl ToTokens for Instance {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module = &self.module;
        let instance = &self.instance;
        let connections = self.connections.inner.iter();
        tokens.extend(quote! {
            rhdl::vlog::Instance {
                module: stringify!(#module).into(),
                instance: stringify!(#instance).into(),
                connections: vec![#(#connections,)*],
            }
        })
    }
}

#[derive(Debug, Clone)]
struct Block {
    _begin: kw::begin,
    body: Vec<Stmt>,
    _end: kw::end,
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let statements = self.body.iter().enumerate().map(|(i, stmt)| {
            let var_name = format_ident!("stmt{}", i);
            quote! {let #var_name = #stmt;}
        });
        let variables = (0..self.body.len()).map(|i| {
            let var_name = format_ident!("stmt{}", i);
            quote! {#var_name}
        });
        tokens.extend(quote! {
            {
                #(#statements)*
                vec![
                    #(#variables,)*
                ]
            }
        })
    }
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let begin = input.parse()?;
        let mut body = Vec::new();
        loop {
            if input.peek(kw::end) {
                break;
            }
            body.push(input.parse()?);
        }
        let end = input.parse()?;
        Ok(Self {
            _begin: begin,
            body,
            _end: end,
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct DynamicSplice {
    lhs: Box<ExprDynIndex>,
    _eq: Token![=],
    rhs: Box<Expr>,
}

impl ToTokens for DynamicSplice {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.lhs.target;
        let base = &self.lhs.address.base;
        let width = &self.lhs.address.width;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::DynamicSplice {
                target: stringify!(#target).into(),
                base: Box::new(#base),
                width: Box::new(#width),
                rhs: Box::new(#rhs),
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Splice {
    lhs: Box<ExprIndex>,
    _eq: Token![=],
    rhs: Box<Expr>,
}

impl ToTokens for Splice {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.lhs.target;
        let msb = &self.lhs.address.msb;
        let lsb = &self.lhs.address.lsb;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::Splice {
                target: stringify!(#target).into(),
                msb: Box::new(#msb),
                lsb: #lsb.map(Box::new),
                rhs: Box::new(#rhs),
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
enum Sensitivity {
    #[parse(peek = kw::posedge)]
    PosEdge(PosEdgeSensitivity),
    #[parse(peek = kw::negedge)]
    NegEdge(NegEdgeSensitivity),
    #[parse(peek = Ident)]
    Signal(Ident),
    #[parse(peek = Token![*])]
    Star(Token![*]),
}

impl ToTokens for Sensitivity {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Sensitivity::PosEdge(sen) => {
                let ident = &sen.ident;
                quote! {rhdl::vlog::Sensitivity::PosEdge(stringify!(#ident).into())}
            }
            Sensitivity::NegEdge(sen) => {
                let ident = &sen.ident;
                quote! {rhdl::vlog::Sensitivity::NegEdge(stringify!(#ident).into())}
            }
            Sensitivity::Signal(ident) => {
                quote! {rhdl::vlog::Sensitivity::Signal(stringify!(#ident).into())}
            }
            Sensitivity::Star(inner) => {
                let _ = inner;
                quote! {rhdl::vlog::Sensitivity::Star}
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct PosEdgeSensitivity {
    _posedge: kw::posedge,
    ident: Ident,
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct NegEdgeSensitivity {
    _negedge: kw::negedge,
    ident: Ident,
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct SensitivityList {
    _at: Token![@],
    elements: ParenCommaList<Sensitivity>,
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Always {
    _always: kw::always,
    sensitivity: SensitivityList,
    body: Box<Stmt>,
}

impl ToTokens for Always {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sensitivity = self.sensitivity.elements.inner.iter();
        let body = &self.body;
        tokens.extend(quote! {
            rhdl::vlog::Always {
                sensitivity: vec![#(#sensitivity),*],
                body: Box::new(#body),
            }
        });
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct LocalParam {
    _localparam: kw::localparam,
    target: Ident,
    _eq: Token![=],
    rhs: LitVerilog,
}

impl ToTokens for LocalParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::LocalParam {
                target: stringify!(#target).into(),
                rhs: Box::new(#rhs),
            }
        });
    }
}

#[derive(Debug, Clone)]
struct ElseBranch {
    _else_token: Token![else],
    stmt: Box<Stmt>,
}

impl ToTokens for ElseBranch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.stmt.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
struct If {
    _if_token: Token![if],
    condition: Parenthesized<Box<Expr>>,
    true_stmt: Box<Stmt>,
    else_branch: Option<ElseBranch>,
}

impl ToTokens for If {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let condition = &self.condition.inner;
        let true_stmt = &self.true_stmt;
        let else_branch = if let Some(else_branch) = &self.else_branch {
            quote! { Some(Box::new(#else_branch)) }
        } else {
            quote! { None }
        };
        tokens.extend(quote! {
            rhdl::vlog::If {
                condition: Box::new(#condition),
                true_stmt: Box::new(#true_stmt),
                else_branch: #else_branch,
            }
        });
    }
}

impl Parse for If {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut clauses = Vec::new();
        let mut stmt;
        loop {
            let if_token: Token![if] = input.parse()?;
            let condition = input.parse()?;
            let true_stmt = input.parse()?;

            stmt = If {
                _if_token: if_token,
                condition,
                true_stmt,
                else_branch: None,
            };

            if !input.peek(Token![else]) {
                break;
            }

            let else_token: Token![else] = input.parse()?;
            if input.peek(Token![if]) {
                stmt.else_branch = Some(ElseBranch {
                    _else_token: else_token,
                    stmt: Box::new(Stmt::default()),
                });
                clauses.push(stmt);
            } else {
                stmt.else_branch = Some(ElseBranch {
                    _else_token: else_token,
                    stmt: input.parse()?,
                });
                break;
            }
        }

        while let Some(mut prev) = clauses.pop() {
            *prev.else_branch.as_mut().unwrap().stmt = Stmt {
                kind: StmtKind::If(stmt),
                _terminator: None,
            };
            stmt = prev;
        }
        Ok(stmt)
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct NonblockAssign {
    target: Ident,
    _left_arrow: LeftArrow,
    rhs: Box<Expr>,
}

impl ToTokens for NonblockAssign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::Assign {
                target: stringify!(#target).into(),
                rhs: Box::new(#rhs),
            }
        });
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ContinuousAssign {
    _kw_assign: kw::assign,
    assign: Assign,
}

impl ToTokens for ContinuousAssign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.assign.to_tokens(tokens);
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct Assign {
    target: Ident,
    _eq: Token![=],
    rhs: Box<Expr>,
}

impl ToTokens for Assign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::Assign {
                target: stringify!(#target).into(),
                rhs: Box::new(#rhs),
            }
        });
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ConcatAssign {
    target: ExprConcat,
    _eq: Token![=],
    rhs: Box<Expr>,
}

impl ToTokens for ConcatAssign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::ConcatAssign {
                target: #target
                rhs: Box::new(#rhs),
            }
        });
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Constant(LitVerilog),
    Literal(LitInt),
    String(LitStr),
    Ident(Ident),
    Paren(Box<Expr>),
    Ternary(ExprTernary),
    Concat(ExprConcat),
    Replica(ExprReplica),
    Index(ExprIndex),
    DynIndex(ExprDynIndex),
    Function(ExprFunction),
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Expr::Binary(expr) => quote! {rhdl::vlog::Expr::Binary(#expr)},
            Expr::Unary(expr) => quote! {rhdl::vlog::Expr::Unary(#expr)},
            Expr::Constant(lit) => quote! {rhdl::vlog::Expr::Constant(#lit)},
            Expr::Literal(lit) => quote! {rhdl::vlog::Expr::Literal(#lit)},
            Expr::String(lit) => quote! {rhdl::vlog::Expr::String(#lit)},
            Expr::Ident(ident) => quote! {rhdl::vlog::Expr::Ident(stringify!(#ident).into())},
            Expr::Paren(expr) => quote! {rhdl::vlog::Expr::Paren(Box::new(#expr))},
            Expr::Ternary(expr) => quote! {rhdl::vlog::Expr::Ternary(#expr)},
            Expr::Concat(expr) => quote! {rhdl::vlog::Expr::Concat(#expr)},
            Expr::Replica(expr) => quote! {rhdl::vlog::Expr::Replica(#expr)},
            Expr::Index(expr) => quote! {rhdl::vlog::Expr::Index(#expr)},
            Expr::DynIndex(expr) => quote! {rhdl::vlog::Expr::DynIndex(#expr)},
            Expr::Function(expr) => quote! {rhdl::vlog::Expr::Function(#expr)},
        })
    }
}

impl Parse for Expr {
    fn parse(mut input: ParseStream) -> Result<Self> {
        expr_bp(&mut input, 0)
    }
}

// matklad's Pratt parser.  From
// https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn expr_bp(input: &mut ParseStream, min_bp: u8) -> Result<Expr> {
    let lookahead = input.lookahead1();
    let mut lhs = if lookahead.peek(LitInt) && input.peek2(Lifetime) {
        input.parse().map(Expr::Constant)?
    } else if lookahead.peek(LitInt) {
        input.parse().map(Expr::Literal)?
    } else if lookahead.peek(LitStr) {
        input.parse().map(Expr::String)?
    } else if input.fork().parse::<ExprFunction>().is_ok() {
        input.parse().map(Expr::Function)?
    } else if input.fork().parse::<ExprIndex>().is_ok() {
        input.parse().map(Expr::Index)?
    } else if input.fork().parse::<ExprDynIndex>().is_ok() {
        input.parse().map(Expr::DynIndex)?
    } else if lookahead.peek(Ident) {
        input.parse().map(Expr::Ident)?
    } else if lookahead.peek(token::Paren) {
        let content;
        parenthesized!(content in input);
        let expr = content.parse::<Expr>()?;
        Expr::Paren(Box::new(expr))
    } else if lookahead.peek(token::Brace) {
        // Try to parse as a replica
        if input.fork().parse::<ExprReplica>().is_ok() {
            input.parse().map(Expr::Replica)?
        } else {
            input.parse().map(Expr::Concat)?
        }
    } else {
        let op = input.parse::<UnaryOp>()?;
        let r_bp = op.binding_power();
        let arg = Box::new(expr_bp(input, r_bp)?);
        Expr::Unary(ExprUnary { op, arg })
    };
    loop {
        if input.is_empty() {
            break;
        }
        // Check for a trinary operator
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![:])
            || lookahead.peek(Token![,])
            || lookahead.peek(PlusColon)
            || lookahead.peek(Token![;])
        {
            break;
        } else if lookahead.peek(Token![?]) {
            let (l_bp, r_bp) = TERNARY_BINDING;
            if l_bp < min_bp {
                break;
            }
            let op = input.parse::<Token![?]>()?;
            let mhs = Box::new(expr_bp(input, 0)?);
            let colon = input.parse::<Token![:]>()?;
            let rhs = Box::new(expr_bp(input, r_bp)?);
            lhs = Expr::Ternary(ExprTernary {
                lhs: Box::new(lhs),
                _op: op,
                mhs,
                _colon: colon,
                rhs,
            });
        } else {
            let op = input.fork().parse::<BinaryOp>()?;
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp {
                break;
            }
            let _ = input.parse::<BinaryOp>()?;
            let rhs = Box::new(expr_bp(input, r_bp)?);
            lhs = Expr::Binary(ExprBinary {
                op,
                lhs: Box::new(lhs),
                rhs,
            });
        }
    }
    Ok(lhs)
}

#[derive(Debug, Clone)]
struct ExprBinary {
    lhs: Box<Expr>,
    op: BinaryOp,
    rhs: Box<Expr>,
}

impl ToTokens for ExprBinary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let lhs = &self.lhs;
        let op = &self.op;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::ExprBinary {
                lhs: Box::new(#lhs),
                op: #op,
                rhs: Box::new(#rhs)
            }
        })
    }
}

#[derive(Debug, Clone)]
struct ExprUnary {
    op: UnaryOp,
    arg: Box<Expr>,
}

impl ToTokens for ExprUnary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let op = &self.op;
        let arg = &self.arg;
        tokens.extend(quote! {
            rhdl::vlog::ExprUnary {
                op: #op,
                arg: Box::new(#arg),
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprConcat {
    #[syn(braced)]
    _brace: Brace,
    #[syn(in = _brace)]
    #[parse(Punctuated::parse_terminated)]
    elements: Punctuated<Expr, Token![,]>,
}

impl ToTokens for ExprConcat {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let elements = self.elements.iter();
        tokens.extend(quote! {
            vec![#(#elements,)*]
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprReplicaInner {
    count: LitInt,
    concatenation: ExprConcat,
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprReplica {
    #[syn(braced)]
    _brace: Brace,
    #[syn(in = _brace)]
    inner: ExprReplicaInner,
}

impl ToTokens for ExprReplica {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let count = &self.inner.count;
        let concatenation = &self.inner.concatenation;
        tokens.extend(quote! {
            rhdl::vlog::ExprReplica {
                count: #count,
                concatenation: #concatenation,
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprFunction {
    dollar: Option<Token![$]>,
    name: Ident,
    args: ParenCommaList<Expr>,
}

impl ToTokens for ExprFunction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = if self.dollar.is_some() {
            format_ident!("_{}", self.name)
        } else {
            self.name.clone()
        };
        let args = self.args.inner.iter();
        tokens.extend(quote! {
            rhdl::vlog::ExprFunction {
                name: #name,
                args: vec![#(#args,)*]
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprDynIndexInner {
    base: Box<Expr>,
    _op: PlusColon,
    width: Box<Expr>,
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprDynIndex {
    target: Ident,
    #[syn(bracketed)]
    _bracket: Bracket,
    #[syn(in = _bracket)]
    address: ExprDynIndexInner,
}

impl ToTokens for ExprDynIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let base = &self.address.base;
        let width = &self.address.width;
        tokens.extend(quote! {
            rhdl::vlog::ExprDynIndex {
                target: stringify!(#target).into(),
                base: Box::new(#base),
                width: Box::new(#width),
            }
        })
    }
}

#[derive(Debug, Clone)]
struct ExprIndexAddress {
    msb: Box<Expr>,
    lsb: Option<Pair<Token![:], Box<Expr>>>,
}

impl Parse for ExprIndexAddress {
    fn parse(input: ParseStream) -> Result<Self> {
        let msb = input.parse()?;
        if !input.is_empty() {
            let colon = input.parse::<Token![:]>()?;
            let lsb = input.parse()?;
            Ok(ExprIndexAddress {
                msb,
                lsb: Some(Pair(colon, lsb)),
            })
        } else {
            Ok(ExprIndexAddress { msb, lsb: None })
        }
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprIndex {
    target: Ident,
    #[syn(bracketed)]
    _bracket: Bracket,
    #[syn(in = _bracket)]
    address: ExprIndexAddress,
}

impl ToTokens for ExprIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let target = &self.target;
        let msb = &self.address.msb;
        let lsb = self.address.lsb.as_ref().map(|x| &x.1);
        tokens.extend(quote! {
            rhdl::vlog::ExprIndex {
                target: stringify!(#target).into(),
                address: rhdl::vlog::ExprIndexAddress {
                    msb: Box::new(#msb),
                    lsb: #lsb.map(Box::new),
                },
            }
        })
    }
}

#[derive(Debug, Clone, syn_derive::Parse)]
struct ExprTernary {
    lhs: Box<Expr>,
    _op: Token![?],
    mhs: Box<Expr>,
    _colon: Token![:],
    rhs: Box<Expr>,
}

impl ToTokens for ExprTernary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let lhs = &self.lhs;
        let mhs = &self.mhs;
        let rhs = &self.rhs;
        tokens.extend(quote! {
            rhdl::vlog::ExprTernary {
                lhs: #lhs,
                mhs: #mhs,
                rhs: #rhs,
            }
        })
    }
}

#[derive(Debug, Clone)]
enum UnaryOp {
    Plus,
    Minus,
    Bang,
    Not,
    And,
    Or,
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
            Err(input.error("expected unary operator"))
        }
    }
}

impl ToTokens for UnaryOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            UnaryOp::Plus => quote! {rhdl::vlog::UnaryOp::Plus},
            UnaryOp::Minus => quote! {rhdl::vlog::UnaryOp::Minus},
            UnaryOp::Bang => quote! {rhdl::vlog::UnaryOp::Bang},
            UnaryOp::Not => quote! {rhdl::vlog::UnaryOp::Not},
            UnaryOp::And => quote! {rhdl::vlog::UnaryOp::And},
            UnaryOp::Or => quote! {rhdl::vlog::UnaryOp::Or},
            UnaryOp::Xor => quote! {rhdl::vlog::UnaryOp::Xor},
        })
    }
}

impl UnaryOp {
    fn binding_power(&self) -> u8 {
        50
    }
}

#[derive(Debug, Clone)]
enum BinaryOp {
    Shl,
    SignedRightShift,
    Shr,
    ShortAnd,
    ShortOr,
    CaseEq,
    CaseNe,
    Ne,
    Eq,
    Ge,
    Le,
    Gt,
    Lt,
    Plus,
    Minus,
    And,
    Or,
    Xor,
    Mod,
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
        } else if lookahead.peek(CaseEqual) {
            let _: CaseEqual = input.parse()?;
            Ok(BinaryOp::CaseEq)
        } else if lookahead.peek(CaseUnequal) {
            let _: CaseUnequal = input.parse()?;
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
        } else if lookahead.peek(Token![+]) {
            let _: Token![+] = input.parse()?;
            Ok(BinaryOp::Plus)
        } else if lookahead.peek(Token![-]) {
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

impl ToTokens for BinaryOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            BinaryOp::Shl => quote! {rhdl::vlog::BinaryOp::Shl},
            BinaryOp::SignedRightShift => quote! {rhdl::vlog::BinaryOp::SignedRightShift},
            BinaryOp::Shr => quote! {rhdl::vlog::BinaryOp::Shr},
            BinaryOp::ShortAnd => quote! {rhdl::vlog::BinaryOp::ShortAnd},
            BinaryOp::ShortOr => quote! {rhdl::vlog::BinaryOp::ShortOr},
            BinaryOp::CaseEq => quote! {rhdl::vlog::BinaryOp::CaseEq},
            BinaryOp::CaseNe => quote! {rhdl::vlog::BinaryOp::CaseNe},
            BinaryOp::Ne => quote! {rhdl::vlog::BinaryOp::Ne},
            BinaryOp::Eq => quote! {rhdl::vlog::BinaryOp::Eq},
            BinaryOp::Ge => quote! {rhdl::vlog::BinaryOp::Ge},
            BinaryOp::Le => quote! {rhdl::vlog::BinaryOp::Le},
            BinaryOp::Gt => quote! {rhdl::vlog::BinaryOp::Gt},
            BinaryOp::Lt => quote! {rhdl::vlog::BinaryOp::Lt},
            BinaryOp::Plus => quote! {rhdl::vlog::BinaryOp::Plus},
            BinaryOp::Minus => quote! {rhdl::vlog::BinaryOp::Minus},
            BinaryOp::And => quote! {rhdl::vlog::BinaryOp::And},
            BinaryOp::Or => quote! {rhdl::vlog::BinaryOp::Or},
            BinaryOp::Xor => quote! {rhdl::vlog::BinaryOp::Xor},
            BinaryOp::Mod => quote! {rhdl::vlog::BinaryOp::Mod},
            BinaryOp::Mul => quote! {rhdl::vlog::BinaryOp::Mul},
        })
    }
}

impl BinaryOp {
    fn binding_power(&self) -> (u8, u8) {
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
const TERNARY_BINDING: (u8, u8) = (2, 1);

#[derive(Debug, Clone, syn_derive::Parse)]
struct LitVerilog {
    width: LitInt,
    lifetime: Lifetime,
}

impl ToTokens for LitVerilog {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let width = &self.width;
        let lifetime = &self.lifetime.ident;
        tokens.extend(quote! {
            rhdl::vlog::LitVerilog {
                width: #width,
                lifetime: stringify!(#lifetime).into(),
            }
        })
    }
}

#[derive(Clone, Debug)]
struct ModuleList {
    modules: Vec<ModuleDef>,
}

impl ToTokens for ModuleList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let modules = self.modules.iter().enumerate().map(|(i, module)| {
            let var_name = format_ident!("module{}", i);
            quote! { let #var_name = #module; }
        });
        let module_names = (0..modules.len()).map(|i| {
            let var_name = format_ident!("module{}", i);
            quote! { #var_name }
        });
        tokens.extend(quote! {
            {
                #(#modules)*;
                rhdl::vlog::ModuleList(vec![#(#module_names,)*])
            }
        })
    }
}

impl Parse for ModuleList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut modules = Vec::new();
        loop {
            if input.is_empty() {
                break;
            }
            modules.push(input.parse()?);
        }
        Ok(Self { modules })
    }
}

#[derive(Clone, Debug)]
struct ModuleDef {
    _module: kw::module,
    name: Ident,
    args: Option<ParenCommaList<Port>>,
    _semi: Token![;],
    items: Vec<Item>,
    _end_module: kw::endmodule,
}

impl ToTokens for ModuleDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;

        // Generate let statements for args
        let args_construction = if let Some(ref args) = self.args {
            let ports = args.inner.iter().enumerate().map(|(i, port)| {
                let var_name = format_ident!("arg{}", i);
                quote! { let #var_name = #port; }
            });
            let var_names = (0..args.inner.len()).map(|i| format_ident!("arg{}", i));
            quote! {
                #(#ports)*
                let args_vec = vec![#(#var_names,)*];
            }
        } else {
            quote! { let args_vec = vec![]; }
        };

        // Generate let statements for items
        let items = &self.items;
        let items_construction = {
            let item_lets = items.iter().enumerate().map(|(i, item)| {
                let var_name = format_ident!("item{}", i);
                quote! { let #var_name = #item; }
            });
            let var_names = (0..items.len()).map(|i| format_ident!("item{}", i));
            quote! {
                #(#item_lets)*
                let items_vec = vec![#(#var_names,)*];
            }
        };

        tokens.extend(quote! {
            {
                #args_construction
                #items_construction
                rhdl::vlog::ModuleDef {
                    name: stringify!(#name).into(),
                    args: args_vec,
                    items: items_vec,
                }
            }
        })
    }
}

impl Parse for ModuleDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let module = input.parse()?;
        let name = input.parse()?;
        let args = if input.peek(token::Paren) {
            Some(input.parse()?)
        } else {
            None
        };
        let semi = input.parse()?;
        let mut items = Vec::new();
        while !input.peek(kw::endmodule) {
            items.push(input.parse()?);
        }
        let end_module = input.parse()?;
        Ok(Self {
            _module: module,
            name,
            args,
            _semi: semi,
            items,
            _end_module: end_module,
        })
    }
}

#[derive(Clone, Debug)]
struct FunctionDef {
    _function: kw::function,
    signed_width: SignedWidth,
    name: Ident,
    args: ParenCommaList<Port>,
    _semi: Token![;],
    items: Vec<Item>,
    _end_function: kw::endfunction,
}

impl ToTokens for FunctionDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let width = &self.signed_width;
        let name = &self.name;
        let args = self.args.inner.iter();
        let items = &self.items;
        tokens.extend(quote! {
            rhdl::vlog::FunctionDef {
                width: #width,
                name: stringify!(#name).into(),
                args: vec![#(#args,)*],
                items: vec![#(#items,)*],
            }
        })
    }
}

impl Parse for FunctionDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let function = input.parse()?;
        let signed_width = input.parse()?;
        let name = input.parse()?;
        let args = input.parse()?;
        let semi = input.parse()?;
        let mut items = Vec::new();
        while !input.peek(kw::endfunction) {
            items.push(input.parse()?);
        }
        let end_function = input.parse()?;
        Ok(FunctionDef {
            _function: function,
            signed_width,
            name,
            args,
            _semi: semi,
            items,
            _end_function: end_function,
        })
    }
}

#[cfg(test)]
mod itests {

    use super::*;
    use quote::quote;
    fn test_parse<T: Parse>(text: impl AsRef<str>) -> std::result::Result<T, miette::Report> {
        let text = text.as_ref();
        syn::parse_str::<T>(text).map_err(|err| syn_miette::Error::new(err, text).into())
    }

    fn test_parse_quote<T: Parse + ToTokens>(
        text: impl AsRef<str>,
    ) -> std::result::Result<String, miette::Report> {
        let text = text.as_ref();
        let val = syn::parse_str::<T>(text).map_err(|err| syn_miette::Error::new(err, text))?;
        let quoted = quote! {#val};
        Ok(quoted.to_string())
    }

    #[test]
    fn test_vlog_files() -> miette::Result<()> {
        let dir = std::fs::read_dir("vlog").unwrap();
        for entry in dir {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            };
            eprintln!("Path: {:?}", entry.path());
            let path = entry.path();
            let Some(extension) = path.extension() else {
                continue;
            };
            if extension != "v" {
                continue;
            }
            let Ok(module) = std::fs::read(entry.path()) else {
                continue;
            };
            let text = String::from_utf8_lossy(&module);
            let module_list = test_parse::<ModuleList>(text)?;
            let requote = quote! {#module_list ;}.to_string();
            let _ = syn::parse_str::<syn::Stmt>(&requote)
                .map_err(|err| syn_miette::Error::new(err, requote))?;
        }
        Ok(())
    }
}
