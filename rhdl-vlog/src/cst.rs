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

syn::custom_punctuation!(MinusColon, -:);
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
            HDLKind::Reg(_) => quote! {vlog::reg()},
            HDLKind::Wire(_) => quote! {vlog::wire()},
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
            Direction::Input(_) => quote! {vlog::input()},
            Direction::Output(_) => quote! {vlog::output()},
            Direction::Inout(_) => quote! {vlog::inout()},
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
            quote! {vlog::signed(#width)}
        } else {
            quote! {vlog::unsigned(#width)}
        })
    }
}

#[derive(Debug, Clone)]
struct DeclKind {
    name: Ident,
    width: Option<SignedWidth>,
}

impl Parse for DeclKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let width = if input.peek(token::Bracket) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(DeclKind { name, width })
    }
}

impl ToTokens for DeclKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let width = option_tokens(self.width.as_ref());
        tokens.extend(quote! {
            vlog::decl_kind(stringify!(#name), #width)
        })
    }
}

#[derive(Debug, Clone)]
struct DeclarationList {
    kind: HDLKind,
    signed_width: Option<SignedWidth>,
    items: Punctuated<DeclKind, Token![,]>,
    _term: Token![;],
}

impl Parse for DeclarationList {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse()?;
        let signed_width = if input.peek(token::Bracket) || input.peek(kw::signed) {
            Some(input.parse()?)
        } else {
            None
        };
        let items = Punctuated::parse_separated_nonempty(input)?;
        let _term = input.parse()?;
        Ok(DeclarationList {
            kind,
            signed_width,
            items,
            _term,
        })
    }
}

impl ToTokens for DeclarationList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let kind = &self.kind;
        let signed_width = if let Some(width) = self.signed_width.as_ref() {
            quote! {#width}
        } else {
            quote! {vlog::unsigned(0..=0)}
        };
        let items = iter_tokens(self.items.iter());
        tokens.extend(quote! {
            vlog::declaration_list(#kind, #signed_width, #items)
        })
    }
}

#[derive(Debug, Clone)]
struct Declaration {
    kind: HDLKind,
    signed_width: Option<SignedWidth>,
    name: Ident,
    _term: Option<Token![;]>,
}

impl Parse for Declaration {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse()?;
        let signed_width = if input.peek(token::Bracket) || input.peek(kw::signed) {
            Some(input.parse()?)
        } else {
            None
        };
        let name = input.parse()?;
        let _term = input.parse()?;
        Ok(Declaration {
            kind,
            signed_width,
            name,
            _term,
        })
    }
}

impl ToTokens for Declaration {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let kind = &self.kind;
        let signed_width = if let Some(width) = self.signed_width.as_ref() {
            quote! {#width}
        } else {
            quote! {vlog::unsigned(0..=0)}
        };
        let name = &self.name;
        tokens.extend(quote! {
            vlog::declaration(#kind, #signed_width, stringify!(#name))
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
            vlog::port(#direction, #decl)
        })
    }
}

#[derive(Default)]
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
    DynamicSplice(DynamicSplice),
    Delay(Delay),
    ConcatAssign(ConcatAssign),
    #[default]
    /// Required because the parser for if/else uses it as a placeholder
    Noop,
}

fn option_tokens<T: ToTokens>(t: Option<T>) -> proc_macro2::TokenStream {
    match t {
        Some(inner) => {
            quote! {Some(#inner)}
        }
        None => quote! {None},
    }
}

fn iter_tokens<T: ToTokens>(t: impl IntoIterator<Item = T>) -> proc_macro2::TokenStream {
    let t = t.into_iter();
    let elements = t.collect::<Vec<_>>();
    vec_tokens(&elements)
}

fn vec_tokens<T: ToTokens>(t: &[T]) -> proc_macro2::TokenStream {
    if t.len() == 0 {
        return quote! {vec![]};
    }
    let len = t.len();
    let ret = format_ident!("ret");
    let push = t.iter().map(|x| quote! {#ret.push(#x);});
    quote! {
        {
            let mut #ret = Vec::with_capacity(#len);
            #(#push)*
            #ret
        }
    }
}

impl ToTokens for StmtKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            StmtKind::If(inner) => {
                let condition = &inner.condition.inner;
                let true_stmt = &inner.true_stmt;
                let else_stmt = option_tokens(inner.else_branch.as_ref().map(|x| &x.stmt));
                quote! {
                    vlog::if_stmt(#condition, #true_stmt, #else_stmt)
                }
            }
            StmtKind::Always(inner) => {
                let sensitivity = &inner.sensitivity;
                let body = &inner.body;
                quote! {
                    vlog::always_stmt(#sensitivity, #body)
                }
            }
            StmtKind::Case(inner) => {
                let discriminant = &inner.discriminant;
                let lines = vec_tokens(&inner.lines);
                quote! {
                    vlog::case_stmt(#discriminant, #lines)
                }
            }
            StmtKind::LocalParam(inner) => {
                let target = &inner.target;
                let rhs = &inner.rhs;
                quote! {
                    vlog::local_param_stmt(stringify!(#target), #rhs)
                }
            }
            StmtKind::Block(inner) => {
                let inner = vec_tokens(&inner.body);
                quote! {
                    vlog::block_stmt(#inner)
                }
            }
            StmtKind::ContinuousAssign(inner) => {
                let target = &inner.assign.target;
                let rhs = &inner.assign.rhs;
                quote! {
                    vlog::continuous_assign_stmt(#target, #rhs)
                }
            }
            StmtKind::FunctionCall(inner) => {
                let name = format_ident!("_dollar_{}", inner.name);
                let args = inner
                    .args
                    .iter()
                    .flat_map(|x| x.inner.iter())
                    .collect::<Vec<_>>();
                let args = vec_tokens(&args);
                quote! {
                    vlog::function_call_stmt(stringify!(#name), #args)
                }
            }
            StmtKind::NonblockAssign(inner) => {
                let target = &inner.target;
                let rhs = &inner.rhs;
                quote! {
                    vlog::nonblock_assign_stmt(#target, #rhs)
                }
            }
            StmtKind::Assign(inner) => {
                let target = &inner.target;
                let rhs = &inner.rhs;
                quote! {
                    vlog::assign_stmt(#target, #rhs)
                }
            }
            StmtKind::Instance(inner) => {
                let module = &inner.module;
                let instance = &inner.instance;
                let connections = iter_tokens(&inner.connections.inner);
                quote! {
                    vlog::instance_stmt(stringify!(#module), stringify!(#instance), #connections)
                }
            }
            StmtKind::DynamicSplice(inner) => {
                let target = &inner.lhs.target;
                let base = &inner.lhs.address.base;
                let op = &inner.lhs.address.op;
                let width = &inner.lhs.address.width;
                let rhs = &inner.rhs;
                quote! {
                    vlog::dynamic_splice_stmt(stringify!(#target), #base, #op, #width, #rhs)
                }
            }
            StmtKind::Delay(inner) => {
                let delay = &inner.length;
                quote! {
                    vlog::delay_stmt(#delay)
                }
            }
            StmtKind::ConcatAssign(inner) => {
                let target = iter_tokens(&inner.target.elements);
                let rhs = &inner.rhs;
                quote! {
                    vlog::concat_assign_stmt(#target, #rhs)
                }
            }
            StmtKind::Noop => quote! {},
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
        } else if input.fork().parse::<NonblockAssign>().is_ok() {
            input.parse().map(StmtKind::NonblockAssign)
        } else if input.fork().parse::<Assign>().is_ok() {
            input.parse().map(StmtKind::Assign)
        } else if lookahead.peek(Ident) && input.peek2(Ident) && input.peek3(token::Paren) {
            input.parse().map(StmtKind::Instance)
        } else if lookahead.peek(Ident) && input.peek2(Bracket) {
            input.parse().map(StmtKind::DynamicSplice)
        } else if lookahead.peek(token::Brace) {
            input.parse().map(StmtKind::ConcatAssign)
        } else if lookahead.peek(token::Pound) {
            input.parse().map(StmtKind::Delay)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default, syn_derive::Parse)]
struct Stmt {
    kind: StmtKind,
    _terminator: Option<Token![;]>,
}

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.kind.to_tokens(tokens);
    }
}

enum Item {
    Statement(Stmt),
    Declaration(DeclarationList),
    FunctionDef(FunctionDef),
    Initial(Initial),
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Item::Statement(stmt) => quote! {
                vlog::stmt_item(#stmt)
            },
            Item::Declaration(decl) => quote! {
                vlog::declaration_item(#decl)
            },
            Item::FunctionDef(func) => quote! {
                vlog::function_def_item(#func)
            },
            Item::Initial(init) => quote! {
                vlog::initial_item(#init)
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

#[derive(syn_derive::Parse)]
struct Initial {
    _initial_kw: kw::initial,
    statment: Stmt,
}

impl ToTokens for Initial {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.statment.to_tokens(tokens);
    }
}

#[derive(syn_derive::Parse)]
struct Delay {
    _hash_token: Token![#],
    length: LitInt,
}

struct FunctionCall {
    _dollar: Token![$],
    name: Ident,
    args: Option<ParenCommaList<Expr>>,
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
        Ok(FunctionCall {
            _dollar: dollar,
            name,
            args,
        })
    }
}

enum CaseItem {
    Ident(Pair<Ident, Token![:]>),
    Literal(Pair<LitVerilog, Token![:]>),
    Wild(Pair<kw::default, Option<Token![:]>>),
}

impl ToTokens for CaseItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            CaseItem::Ident(ident) => {
                let ident = &ident.0;
                tokens.extend(quote! {
                    vlog::case_item_ident(stringify!(#ident))
                });
            }
            CaseItem::Literal(pair) => {
                let lit = &pair.0;
                tokens.extend(quote! {
                    vlog::case_item_literal(#lit)
                });
            }
            CaseItem::Wild(_pair) => {
                tokens.extend(quote! {
                    vlog::case_item_wild()
                });
            }
        }
    }
}

impl Parse for CaseItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::default) {
            input.parse().map(CaseItem::Wild)
        } else if input.peek(Ident) {
            input.parse().map(CaseItem::Ident)
        } else {
            input.parse().map(CaseItem::Literal)
        }
    }
}

#[derive(syn_derive::Parse)]
struct CaseLine {
    item: CaseItem,
    stmt: Box<Stmt>,
}

impl ToTokens for CaseLine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item;
        let stmt = &self.stmt;
        tokens.extend(quote! {
            vlog::case_line(#item, #stmt)
        })
    }
}

struct Pair<S, T>(S, T);

impl<S: Parse, T: Parse> Parse for Pair<S, T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?, input.parse()?))
    }
}

#[derive(syn_derive::Parse)]
struct ParenCommaList<T: Parse + ToTokens> {
    #[syn(parenthesized)]
    _paren: Paren,
    #[syn(in = _paren)]
    #[parse(Punctuated::parse_terminated)]
    inner: Punctuated<T, Token![,]>,
}

#[derive(syn_derive::Parse)]
struct Parenthesized<T: Parse + ToTokens> {
    #[syn(parenthesized)]
    _paren: Paren,
    #[syn(in = _paren)]
    inner: T,
}

struct Case {
    _case: kw::case,
    _parens: Paren,
    discriminant: Box<Expr>,
    lines: Vec<CaseLine>,
    _endcase: kw::endcase,
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

#[derive(syn_derive::Parse)]
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
            vlog::connection(stringify!(#target), #local)
        })
    }
}

#[derive(syn_derive::Parse)]
struct Instance {
    module: Ident,
    instance: Ident,
    connections: ParenCommaList<Connection>,
}

struct Block {
    _begin: kw::begin,
    body: Vec<Stmt>,
    _end: kw::end,
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

#[derive(syn_derive::Parse)]
struct DynamicSplice {
    lhs: Box<ExprDynIndex>,
    _eq: Token![=],
    rhs: Box<Expr>,
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
                quote! {vlog::pos_edge(stringify!(#ident))}
            }
            Sensitivity::NegEdge(sen) => {
                let ident = &sen.ident;
                quote! {vlog::neg_edge(stringify!(#ident))}
            }
            Sensitivity::Signal(ident) => {
                quote! {vlog::signal(stringify!(#ident))}
            }
            Sensitivity::Star(inner) => {
                let _ = inner;
                quote! {vlog::star()}
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

#[derive(syn_derive::Parse)]
struct SensitivityList {
    _at: Token![@],
    elements: ParenCommaList<Sensitivity>,
}

impl ToTokens for SensitivityList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let elements = iter_tokens(self.elements.inner.iter());
        tokens.extend(quote! {
            #elements
        });
    }
}

#[derive(syn_derive::Parse)]
struct Always {
    _always: kw::always,
    sensitivity: SensitivityList,
    body: Box<Stmt>,
}

#[derive(syn_derive::Parse)]
struct LocalParam {
    _localparam: kw::localparam,
    target: Ident,
    _eq: Token![=],
    rhs: ConstExpr,
}

enum ConstExpr {
    LitVerilog(LitVerilog),
    LitInt(LitInt),
    LitStr(LitStr),
}

impl ToTokens for ConstExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ConstExpr::LitVerilog(lit) => {
                tokens.extend(quote! { vlog::const_verilog(#lit) });
            }
            ConstExpr::LitInt(lit) => {
                tokens.extend(quote! { vlog::const_int(#lit) });
            }
            ConstExpr::LitStr(lit) => {
                tokens.extend(quote! { vlog::const_str(#lit) });
            }
        }
    }
}

impl Parse for ConstExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.fork().parse::<LitVerilog>().is_ok() {
            Ok(ConstExpr::LitVerilog(input.parse()?))
        } else if input.fork().parse::<LitInt>().is_ok() {
            Ok(ConstExpr::LitInt(input.parse()?))
        } else if input.fork().parse::<LitStr>().is_ok() {
            Ok(ConstExpr::LitStr(input.parse()?))
        } else {
            Err(input.error("expected constant expression"))
        }
    }
}

struct ElseBranch {
    _else_token: Token![else],
    stmt: Box<Stmt>,
}

struct If {
    _if_token: Token![if],
    condition: Parenthesized<Box<Expr>>,
    true_stmt: Box<Stmt>,
    else_branch: Option<ElseBranch>,
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

enum AssignTarget {
    Ident(Ident),
    Index(ExprIndex),
}

impl ToTokens for AssignTarget {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            AssignTarget::Ident(ident) => {
                tokens.extend(quote! {vlog::assign_target_ident(stringify!(#ident))});
            }
            AssignTarget::Index(expr) => {
                tokens.extend(quote! {vlog::assign_target_index(#expr)});
            }
        }
    }
}

impl Parse for AssignTarget {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident) && input.peek2(token::Bracket) {
            Ok(AssignTarget::Index(input.parse()?))
        } else if input.peek(Ident) {
            Ok(AssignTarget::Ident(input.parse()?))
        } else {
            Err(input.error("expected assignment target"))
        }
    }
}

#[derive(syn_derive::Parse)]
struct NonblockAssign {
    target: AssignTarget,
    _left_arrow: LeftArrow,
    rhs: Box<Expr>,
}

#[derive(syn_derive::Parse)]
struct ContinuousAssign {
    _kw_assign: kw::assign,
    assign: Assign,
}

#[derive(syn_derive::Parse)]
struct Assign {
    target: AssignTarget,
    _eq: Token![=],
    rhs: Box<Expr>,
}

#[derive(syn_derive::Parse)]
struct ConcatAssign {
    target: ExprConcat,
    _eq: Token![=],
    rhs: Box<Expr>,
}

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
            Expr::Binary(expr) => {
                let lhs = &expr.lhs;
                let op = &expr.op;
                let rhs = &expr.rhs;
                quote! {vlog::binary_expr(#lhs, #op, #rhs)}
            }
            Expr::Unary(expr) => {
                let op = &expr.op;
                let arg = &expr.arg;
                quote! {vlog::unary_expr(#op, #arg)}
            }
            Expr::Constant(lit) => {
                let width = &lit.width;
                let value = &lit.lifetime.ident;
                quote! {
                    vlog::constant_expr(vlog::lit_verilog(#width, stringify!(#value)))
                }
            }
            Expr::Literal(lit) => {
                quote! {
                    vlog::literal_expr(#lit)
                }
            }
            Expr::String(lit) => quote! {vlog::string_expr(#lit)},
            Expr::Ident(ident) => quote! {vlog::ident_expr(stringify!(#ident))},
            Expr::Paren(expr) => quote! {vlog::paren_expr(#expr)},
            Expr::Ternary(expr) => {
                let lhs = &expr.lhs;
                let mhs = &expr.mhs;
                let rhs = &expr.rhs;
                quote! {vlog::ternary_expr(#lhs, #mhs, #rhs)}
            }
            Expr::Concat(expr) => {
                let args = iter_tokens(expr.elements.iter());
                quote! {vlog::concat_expr(#args)}
            }
            Expr::Replica(expr) => {
                let count = &expr.inner.count;
                let concatenation = iter_tokens(expr.inner.concatenation.elements.iter());
                quote! {vlog::replica_expr(#count, #concatenation)}
            }
            Expr::Index(expr) => {
                let target = &expr.target;
                let msb = &expr.address.msb;
                let lsb = option_tokens(expr.address.lsb.as_ref().map(|x| &x.1));
                quote! {vlog::index_expr(stringify!(#target), #msb, #lsb)}
            }
            Expr::DynIndex(expr) => {
                let target = &expr.target;
                let base = &expr.address.base;
                let op = &expr.address.op;
                let width = &expr.address.width;
                quote! {vlog::dyn_index_expr(stringify!(#target), #base, #op, #width)}
            }
            Expr::Function(expr) => {
                let name = &expr.name;
                let args = iter_tokens(expr.args.inner.iter());
                quote! {vlog::function_expr(stringify!(#name), #args)}
            }
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
            || lookahead.peek(MinusColon)
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

struct ExprBinary {
    lhs: Box<Expr>,
    op: BinaryOp,
    rhs: Box<Expr>,
}

struct ExprUnary {
    op: UnaryOp,
    arg: Box<Expr>,
}

#[derive(syn_derive::Parse)]
struct ExprConcat {
    #[syn(braced)]
    _brace: Brace,
    #[syn(in = _brace)]
    #[parse(Punctuated::parse_terminated)]
    elements: Punctuated<Expr, Token![,]>,
}

#[derive(syn_derive::Parse)]
struct ExprReplicaInner {
    count: LitInt,
    concatenation: ExprConcat,
}

#[derive(syn_derive::Parse)]
struct ExprReplica {
    #[syn(braced)]
    _brace: Brace,
    #[syn(in = _brace)]
    inner: ExprReplicaInner,
}

#[derive(syn_derive::Parse)]
struct ExprFunction {
    _dollar: Option<Token![$]>,
    name: Ident,
    args: ParenCommaList<Expr>,
}

enum DynOp {
    PlusColon,
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
        } else {
            Err(input.error("expected dynamic operator"))
        }
    }
}

impl ToTokens for DynOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            DynOp::PlusColon => quote! {vlog::dyn_plus_colon()},
            DynOp::MinusColon => quote! {vlog::dyn_minus_colon()},
        })
    }
}

#[derive(syn_derive::Parse)]
struct ExprDynIndexInner {
    base: Box<Expr>,
    op: DynOp,
    width: Box<Expr>,
}

#[derive(syn_derive::Parse)]
struct ExprDynIndex {
    target: Ident,
    #[syn(bracketed)]
    _bracket: Bracket,
    #[syn(in = _bracket)]
    address: ExprDynIndexInner,
}

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

#[derive(syn_derive::Parse)]
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
        let lsb = option_tokens(self.address.lsb.as_ref().map(|x| &x.1));
        tokens.extend(quote! {vlog::index_expr(stringify!(#target), #msb, #lsb)});
    }
}

#[derive(syn_derive::Parse)]
struct ExprTernary {
    lhs: Box<Expr>,
    _op: Token![?],
    mhs: Box<Expr>,
    _colon: Token![:],
    rhs: Box<Expr>,
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
            UnaryOp::Plus => quote! {vlog::unary_plus()},
            UnaryOp::Minus => quote! {vlog::unary_minus()},
            UnaryOp::Bang => quote! {vlog::unary_bang()},
            UnaryOp::Not => quote! {vlog::unary_not()},
            UnaryOp::And => quote! {vlog::unary_and()},
            UnaryOp::Or => quote! {vlog::unary_or()},
            UnaryOp::Xor => quote! {vlog::unary_xor()},
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
            BinaryOp::Shl => quote! {vlog::binary_shl()},
            BinaryOp::SignedRightShift => quote! {vlog::binary_signed_right_shift()},
            BinaryOp::Shr => quote! {vlog::binary_shr()},
            BinaryOp::ShortAnd => quote! {vlog::binary_short_and()},
            BinaryOp::ShortOr => quote! {vlog::binary_short_or()},
            BinaryOp::CaseEq => quote! {vlog::binary_case_eq()},
            BinaryOp::CaseNe => quote! {vlog::binary_case_ne()},
            BinaryOp::Ne => quote! {vlog::binary_ne()},
            BinaryOp::Eq => quote! {vlog::binary_eq()},
            BinaryOp::Ge => quote! {vlog::binary_ge()},
            BinaryOp::Le => quote! {vlog::binary_le()},
            BinaryOp::Gt => quote! {vlog::binary_gt()},
            BinaryOp::Lt => quote! {vlog::binary_lt()},
            BinaryOp::Plus => quote! {vlog::binary_plus()},
            BinaryOp::Minus => quote! {vlog::binary_minus()},
            BinaryOp::And => quote! {vlog::binary_and()},
            BinaryOp::Or => quote! {vlog::binary_or()},
            BinaryOp::Xor => quote! {vlog::binary_xor()},
            BinaryOp::Mod => quote! {vlog::binary_mod()},
            BinaryOp::Mul => quote! {vlog::binary_mul()},
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
            vlog::lit_verilog(#width, stringify!(#lifetime).into())
        })
    }
}

pub struct ModuleList {
    modules: Vec<ModuleDef>,
}

impl ToTokens for ModuleList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let modules = vec_tokens(&self.modules);
        tokens.extend(quote! {
            vlog::module_list(#modules)
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
        let args = iter_tokens(self.args.as_ref().iter().flat_map(|args| args.inner.iter()));
        let items = vec_tokens(&self.items);
        tokens.extend(quote! {
            vlog::module_def(stringify!(#name), #args, #items)
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
        let args = iter_tokens(self.args.inner.iter());
        let items = vec_tokens(&self.items);
        tokens.extend(quote! {
            vlog::function_def(#width, stringify!(#name), #args, #items)
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
