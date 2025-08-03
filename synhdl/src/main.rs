use std::fs::FileType;

use syn::{
    Ident, Lifetime, LitInt, LitStr, Result, Token, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Bracket},
};

use derive_syn_parse::Parse;

/*

#[derive(Debug, Clone, Hash, Default)]
pub struct Module {
    pub name: String,
    pub description: String,
    pub ports: Vec<Port>,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
    pub functions: Vec<Function>,
    pub submodules: Vec<Module>,
}

The syntax for a module definition is:

module <name> (port_list) {
   <declarations>;
   <statements>;
   <functions>;
   <submodules>;
}

A port_list is a comma separated list of port_defs

#[derive(Debug, Clone, Hash)]
pub struct Port {
    pub name: String,
    pub direction: Direction,
    pub kind: HDLKind,
    pub width: SignedWidth,
}

A port is declared as:

<name>: <direction> <kind> <width>

e.g.:

arg1: in<r16>

or

arg1: in<w16>


That doesn't look particularly Rust-ish.  Maybe something more like:

arg1: In<Reg<16>>, arg2: In<Wire<16>>,

Or go with the original Verilog:

<input/output> <reg/wire> (signed)? [{N}:0] name,



#[derive(Debug, Clone, Hash, Copy, PartialEq)]
pub enum Direction {
    Input,
    Output,
    Inout,
}


*/

// Parser for Direction

mod pratt;

#[derive(Debug, Clone, Hash, Copy, PartialEq)]
pub enum HDLKind {
    Wire(kw::wire),
    Reg(kw::reg),
}

#[derive(Debug, Clone, Hash, Copy, PartialEq)]
pub enum Direction {
    Input(kw::input),
    Output(kw::output),
    Inout(kw::inout),
}

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

impl Parse for Direction {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::input) {
            input.parse().map(Direction::Input)
        } else if lookahead.peek(kw::output) {
            input.parse().map(Direction::Output)
        } else if lookahead.peek(kw::inout) {
            input.parse().map(Direction::Inout)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for HDLKind {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::reg) {
            input.parse().map(HDLKind::Reg)
        } else if lookahead.peek(kw::wire) {
            input.parse().map(HDLKind::Wire)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone, Parse)]
pub struct SignedWidth {
    pub signed: Option<kw::signed>,
    pub width: WidthSpec,
}

#[derive(Debug, Clone, Parse)]
pub struct Declaration {
    pub kind: HDLKind,
    pub signed_width: SignedWidth,
    pub name: Ident,
    pub term: Option<Token![;]>,
}

#[derive(Debug, Clone, Parse)]
pub struct Port {
    pub direction: Direction,
    pub decl: Declaration,
}

/*

// Parse an outer attribute like:
//
//     #[repr(C, packed)]
struct OuterAttribute {
    pound_token: Token![#],
    bracket_token: token::Bracket,
    content: TokenStream,
}

impl Parse for OuterAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(OuterAttribute {
            pound_token: input.parse()?,
            bracket_token: bracketed!(content in input),
            content: content.parse()?,
        })
    }
}

*/

#[derive(Debug, Clone)]
pub enum Item {
    Statement(Stmt),
    Declaration(Declaration),
    FunctionDef(FunctionDef),
    Initial(Initial),
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

#[derive(Debug, Clone, Parse)]
pub struct Initial {
    pub initial_kw: kw::initial,
    pub statment: Stmt,
}

#[derive(Debug, Clone, Default, Parse)]
pub struct Stmt {
    pub kind: StmtKind,
    pub terminator: Option<Token![;]>,
}

#[derive(Debug, Clone, Parse)]
pub struct Delay {
    pub hash_token: Token![#],
    pub length: LitInt,
}

#[derive(Debug, Clone, Default)]
pub enum StmtKind {
    If(If),
    Always(Always),
    Case(Case),
    LocalParam(LocalParam),
    Block(Block),
    ContinuousAssign(ContinuousAssign),
    NonblockAssign(NonblockAssign),
    Assign(Assign),
    Instance(Instance),
    Splice(Splice),
    DynamicSplice(DynamicSplice),
    #[default]
    Noop,
    ConcatAssign(ConcatAssign),
    Delay(Delay),
    FunctionCall(FunctionCall),
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

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub dollar: Option<Token![$]>,
    pub name: Ident,
    pub args: Option<ParenCommaList<Expr>>,
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
pub enum CaseItem {
    Literal(Pair<LitVerilog, Token![:]>),
    Wild(Pair<kw::default, Option<Token![:]>>),
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

#[derive(Debug, Clone, Parse)]
pub struct CaseLine {
    pub item: CaseItem,
    pub stmt: Box<Stmt>,
}

/*

case (rega)
16'd0: result = 10'b0111111111;
16'd1: result = 10'b1011111111;
16'd2: result = 10'b1101111111;
16'd3: result = 10'b1110111111;
16'd4: result = 10'b1111011111;
16'd5: result = 10'b1111101111;
16'd6: result = 10'b1111110111;
16'd7: result = 10'b1111111011;
16'd8: result = 10'b1111111101;
16'd9: result = 10'b1111111110;
default result = 'bx;
endcase
*/

pub struct Pair<S, T>(pub S, pub T);

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

pub struct ParenCommaList<T>(pub Punctuated<T, Token![,]>);

impl<T: Parse> Parse for ParenCommaList<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(Self(content.parse_terminated(T::parse, Token![,])?))
    }
}

impl<T: Clone> Clone for ParenCommaList<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ParenCommaList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ParenCommaList").field(&self.0).finish()
    }
}

pub struct Parenthesized<T>(pub T);

impl<T: Parse> Parse for Parenthesized<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(Self(content.parse::<T>()?))
    }
}

impl<T: Clone> Clone for Parenthesized<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Parenthesized<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Parenthesized").field(&self.0).finish()
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub case: kw::case,
    pub discriminant: Box<Expr>,
    pub lines: Vec<CaseLine>,
    pub endcase: kw::endcase,
}

impl Parse for Case {
    fn parse(input: ParseStream) -> Result<Self> {
        let case = input.parse()?;
        let discriminant;
        parenthesized!(discriminant in input);
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
            case,
            discriminant,
            lines,
            endcase,
        })
    }
}

#[derive(Debug, Clone, Parse)]
pub struct Connection {
    pub dot: Token![.],
    pub target: Ident,
    pub local: Box<Expr>,
}

#[derive(Debug, Clone, Parse)]
pub struct Instance {
    pub module: Ident,
    pub instance: Ident,
    pub connections: ParenCommaList<Connection>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub begin: kw::begin,
    pub body: Vec<Stmt>,
    pub end: kw::end,
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
        Ok(Self { begin, body, end })
    }
}

#[derive(Debug, Clone, Parse)]
pub struct DynamicSplice {
    pub lhs: Box<ExprDynIndex>,
    pub eq: Token![=],
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Parse)]
pub struct Splice {
    pub lhs: Box<ExprIndex>,
    pub eq: Token![=],
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Sensitivity {
    PosEdge(PosEdgeSensitivity),
    NegEdge(NegEdgeSensitivity),
    Signal(Ident),
    Star(Token![*]),
}

impl Parse for Sensitivity {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::posedge) {
            input.parse().map(Sensitivity::PosEdge)
        } else if lookahead.peek(kw::negedge) {
            input.parse().map(Sensitivity::NegEdge)
        } else if lookahead.peek(Ident) {
            input.parse().map(Sensitivity::Signal)
        } else if lookahead.peek(Token![*]) {
            input.parse().map(Sensitivity::Star)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone, Parse)]
pub struct PosEdgeSensitivity {
    pub posedge: kw::posedge,
    pub ident: Ident,
}

#[derive(Debug, Clone, Parse)]
pub struct NegEdgeSensitivity {
    pub negedge: kw::negedge,
    pub ident: Ident,
}

#[derive(Debug, Clone, Parse)]
pub struct SensitivityList {
    pub at: Token![@],
    pub elements: ParenCommaList<Sensitivity>,
}

#[derive(Debug, Clone, Parse)]
pub struct Always {
    pub always: kw::always,
    pub sensitivity: SensitivityList,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, Parse)]
pub struct LocalParam {
    pub localparam: kw::localparam,
    pub target: Ident,
    pub eq: Token![=],
    pub rhs: LitVerilog,
}

#[derive(Debug, Clone)]
pub struct If {
    pub if_token: Token![if],
    pub condition: Parenthesized<Box<Expr>>,
    pub true_stmt: Box<Stmt>,
    pub else_branch: Option<(Token![else], Box<Stmt>)>,
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
                if_token,
                condition,
                true_stmt,
                else_branch: None,
            };

            if !input.peek(Token![else]) {
                break;
            }

            let else_token: Token![else] = input.parse()?;
            if input.peek(Token![if]) {
                stmt.else_branch = Some((else_token, Box::new(Stmt::default())));
                clauses.push(stmt);
            } else {
                stmt.else_branch = Some((else_token, input.parse()?));
                break;
            }
        }

        while let Some(mut prev) = clauses.pop() {
            *prev.else_branch.as_mut().unwrap().1 = Stmt {
                kind: StmtKind::If(stmt),
                terminator: None,
            };
            stmt = prev;
        }
        Ok(stmt)
    }
}

#[derive(Debug, Clone, Parse)]
pub struct NonblockAssign {
    pub target: Ident,
    pub larrow: LeftArrow,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Parse)]
pub struct ContinuousAssign {
    pub kw: kw::assign,
    pub assign: Assign,
}

#[derive(Debug, Clone, Parse)]
pub struct Assign {
    pub target: Ident,
    pub eq: Token![=],
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Parse)]
pub struct ConcatAssign {
    pub target: ExprConcat,
    pub eq: Token![=],
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
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

impl Parse for Expr {
    fn parse(mut input: ParseStream) -> Result<Self> {
        expr_bp(&mut input, 0)
    }
}

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
                op,
                mhs,
                colon,
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
pub struct ExprBinary {
    pub op: BinaryOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprUnary {
    pub op: UnaryOp,
    pub arg: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprConcat {
    pub elements: Punctuated<Expr, Token![,]>,
}

impl Parse for ExprConcat {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(Self {
            elements: content.parse_terminated(Expr::parse, Token![,])?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ExprReplica {
    pub count: LitInt,
    pub concatenation: ExprConcat,
}

impl Parse for ExprReplica {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(Self {
            count: content.parse()?,
            concatenation: content.parse()?,
        })
    }
}

#[derive(Debug, Clone, Parse)]
pub struct ExprFunction {
    pub dollar: Option<Token![$]>,
    pub name: Ident,
    pub args: ParenCommaList<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprDynIndex {
    pub target: Ident,
    pub base: Box<Expr>,
    pub op: PlusColon,
    pub width: Box<Expr>,
}

impl Parse for ExprDynIndex {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse()?;
        let content;
        bracketed!(content in input);
        let base = content.parse()?;
        let op = content.parse()?;
        let width = content.parse()?;
        Ok(Self {
            target,
            base,
            op,
            width,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ExprIndex {
    pub target: Ident,
    pub msb: Box<Expr>,
    pub lsb: Option<Box<Expr>>,
}

impl Parse for ExprIndex {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse()?;
        let content;
        bracketed!(content in input);
        let msb = content.parse()?;
        let lsb = if !content.is_empty() {
            let _colon = content.parse::<Token![:]>()?;
            let lsb = content.parse()?;
            Some(lsb)
        } else {
            None
        };
        Ok(Self { target, msb, lsb })
    }
}

#[derive(Debug, Clone)]
pub struct ExprTernary {
    pub lhs: Box<Expr>,
    pub op: Token![?],
    pub mhs: Box<Expr>,
    pub colon: Token![:],
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus(Token![+]),
    Minus(Token![-]),
    Bang(Token![!]),
    Not(Token![~]),
    And(Token![&]),
    Or(Token![|]),
    Xor(Token![^]),
}

impl UnaryOp {
    pub fn binding_power(&self) -> u8 {
        50
    }
}

impl Parse for UnaryOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![+]) {
            input.parse().map(UnaryOp::Plus)
        } else if lookahead.peek(Token![-]) {
            input.parse().map(UnaryOp::Minus)
        } else if lookahead.peek(Token![!]) {
            input.parse().map(UnaryOp::Bang)
        } else if lookahead.peek(Token![~]) {
            input.parse().map(UnaryOp::Not)
        } else if lookahead.peek(Token![&]) {
            input.parse().map(UnaryOp::And)
        } else if lookahead.peek(Token![|]) {
            input.parse().map(UnaryOp::Or)
        } else if lookahead.peek(Token![^]) {
            input.parse().map(UnaryOp::Xor)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Shl(Token![<<]),
    Shr(Token![>>]),
    ShortAnd(Token![&&]),
    ShortOr(Token![||]),
    Ne(Token![!=]),
    Eq(Token![==]),
    Ge(Token![>=]),
    Le(Token![<=]),
    Gt(Token![>]),
    Lt(Token![<]),
    Plus(Token![+]),
    Minus(Token![-]),
    And(Token![&]),
    Or(Token![|]),
    Xor(Token![^]),
    Mod(Token![%]),
    Mul(Token![*]),
    CaseEq(CaseEqual),
    CaseNe(CaseUnequal),
    SignedRightShift(SignedRightShift),
}

impl Parse for BinaryOp {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<<]) {
            input.parse().map(BinaryOp::Shl)
        } else if lookahead.peek(SignedRightShift) {
            input.parse().map(BinaryOp::SignedRightShift)
        } else if lookahead.peek(Token![>>]) {
            input.parse().map(BinaryOp::Shr)
        } else if lookahead.peek(Token![&&]) {
            input.parse().map(BinaryOp::ShortAnd)
        } else if lookahead.peek(Token![||]) {
            input.parse().map(BinaryOp::ShortOr)
        } else if lookahead.peek(CaseEqual) {
            input.parse().map(BinaryOp::CaseEq)
        } else if lookahead.peek(CaseUnequal) {
            input.parse().map(BinaryOp::CaseNe)
        } else if lookahead.peek(Token![!=]) {
            input.parse().map(BinaryOp::Ne)
        } else if lookahead.peek(Token![==]) {
            input.parse().map(BinaryOp::Eq)
        } else if lookahead.peek(Token![>=]) {
            input.parse().map(BinaryOp::Ge)
        } else if lookahead.peek(Token![<=]) {
            input.parse().map(BinaryOp::Le)
        } else if lookahead.peek(Token![>]) {
            input.parse().map(BinaryOp::Gt)
        } else if lookahead.peek(Token![<]) {
            input.parse().map(BinaryOp::Lt)
        } else if lookahead.peek(Token![+]) {
            input.parse().map(BinaryOp::Plus)
        } else if lookahead.peek(Token![-]) {
            input.parse().map(BinaryOp::Minus)
        } else if lookahead.peek(Token![&]) {
            input.parse().map(BinaryOp::And)
        } else if lookahead.peek(Token![|]) {
            input.parse().map(BinaryOp::Or)
        } else if lookahead.peek(Token![^]) {
            input.parse().map(BinaryOp::Xor)
        } else if lookahead.peek(Token![%]) {
            input.parse().map(BinaryOp::Mod)
        } else if lookahead.peek(Token![*]) {
            input.parse().map(BinaryOp::Mul)
        } else {
            Err(lookahead.error())
        }
    }
}

impl BinaryOp {
    fn binding_power(&self) -> (u8, u8) {
        match self {
            BinaryOp::Mod(_) | BinaryOp::Mul(_) => (20, 21),
            BinaryOp::Plus(_) | BinaryOp::Minus(_) => (18, 19),
            BinaryOp::Shl(_) | BinaryOp::Shr(_) | BinaryOp::SignedRightShift(_) => (16, 17),
            BinaryOp::Ge(_) | BinaryOp::Le(_) | BinaryOp::Gt(_) | BinaryOp::Lt(_) => (14, 15),
            BinaryOp::Ne(_) | BinaryOp::Eq(_) | BinaryOp::CaseNe(_) | BinaryOp::CaseEq(_) => {
                (12, 13)
            }
            BinaryOp::And(_) => (10, 11),
            BinaryOp::Xor(_) => (9, 10),
            BinaryOp::Or(_) => (7, 8),
            BinaryOp::ShortAnd(_) => (5, 6),
            BinaryOp::ShortOr(_) => (3, 4),
        }
    }
}
pub const TERNARY_BINDING: (u8, u8) = (2, 1);

#[derive(Debug, Clone)]
pub enum Operator {
    Shl(Token![<<]),
    Shr(Token![>>]),
    ShortAnd(Token![&&]),
    ShortOr(Token![||]),
    Ne(Token![!=]),
    Eq(Token![==]),
    Ge(Token![>=]),
    Le(Token![<=]),
    Gt(Token![>]),
    Lt(Token![<]),
    Plus(Token![+]),
    Minus(Token![-]),
    Bang(Token![!]),
    Not(Token![~]),
    And(Token![&]),
    Or(Token![|]),
    Xor(Token![^]),
    Mod(Token![%]),
    Mul(Token![*]),
    Ternary(Token![?]),
    Assign(Token![=]),
}

impl Parse for Operator {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<<]) {
            input.parse().map(Operator::Shl)
        } else if lookahead.peek(Token![>>]) {
            input.parse().map(Operator::Shr)
        } else if lookahead.peek(Token![&&]) {
            input.parse().map(Operator::ShortAnd)
        } else if lookahead.peek(Token![||]) {
            input.parse().map(Operator::ShortOr)
        } else if lookahead.peek(Token![!=]) {
            input.parse().map(Operator::Ne)
        } else if lookahead.peek(Token![==]) {
            input.parse().map(Operator::Eq)
        } else if lookahead.peek(Token![>=]) {
            input.parse().map(Operator::Ge)
        } else if lookahead.peek(Token![<=]) {
            input.parse().map(Operator::Le)
        } else if lookahead.peek(Token![>]) {
            input.parse().map(Operator::Gt)
        } else if lookahead.peek(Token![<]) {
            input.parse().map(Operator::Lt)
        } else if lookahead.peek(Token![+]) {
            input.parse().map(Operator::Plus)
        } else if lookahead.peek(Token![-]) {
            input.parse().map(Operator::Minus)
        } else if lookahead.peek(Token![!]) {
            input.parse().map(Operator::Bang)
        } else if lookahead.peek(Token![~]) {
            input.parse().map(Operator::Not)
        } else if lookahead.peek(Token![&]) {
            input.parse().map(Operator::And)
        } else if lookahead.peek(Token![|]) {
            input.parse().map(Operator::Or)
        } else if lookahead.peek(Token![^]) {
            input.parse().map(Operator::Xor)
        } else if lookahead.peek(Token![%]) {
            input.parse().map(Operator::Mod)
        } else if lookahead.peek(Token![*]) {
            input.parse().map(Operator::Mul)
        } else if lookahead.peek(Token![?]) {
            input.parse().map(Operator::Ternary)
        } else if lookahead.peek(Token![=]) {
            input.parse().map(Operator::Assign)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Parse)]
pub struct BitRange {
    start: LitInt,
    colon: token::Colon,
    end: LitInt,
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct WidthSpec {
    bracket_token: token::Bracket,
    bit_range: BitRange,
}

impl Parse for WidthSpec {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            bracket_token: bracketed!(content in input),
            bit_range: content.parse()?,
        })
    }
}

#[derive(Debug, Clone, Parse)]
pub struct LitVerilog {
    pub width: LitInt,
    pub lifetime: Lifetime,
}

/*

pub fn module(ast: &Module) -> String {
    let name = &ast.name;
    let description = &ast.description;
    let ports = apply(&ast.ports, port, ", ");
    let declarations = apply(&ast.declarations, register, "\n");
    let statements = apply(&ast.statements, statement, "\n");
    let functions = apply(&ast.functions, function, "\n");
    let sub_modules = ast
        .submodules
        .iter()
        .map(module)
        .collect::<Vec<_>>()
        .join("\n");
    reformat_verilog(&format!(
        "// {description}\nmodule {name}({ports});\n{declarations}\n{statements}\n{functions}\nendmodule\n{sub_modules}\n",
    ))
}

*/

#[derive(Clone, Debug)]
pub struct ModuleList {
    pub modules: Vec<ModuleDef>,
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
pub struct ModuleDef {
    pub module: kw::module,
    pub name: Ident,
    pub args: ParenCommaList<Port>,
    pub semi: Token![;],
    pub items: Vec<Item>,
    pub end_module: kw::endmodule,
}

impl Parse for ModuleDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let module = input.parse()?;
        let name = input.parse()?;
        let args = input.parse()?;
        let semi = input.parse()?;
        let mut items = Vec::new();
        while !input.peek(kw::endmodule) {
            items.push(input.parse()?);
        }
        let end_module = input.parse()?;
        Ok(Self {
            module,
            name,
            args,
            semi,
            items,
            end_module,
        })
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub function: kw::function,
    pub signed_width: SignedWidth,
    pub name: Ident,
    pub args: ParenCommaList<Port>,
    pub semi: Token![;],
    pub items: Vec<Item>,
    pub end_function: kw::endfunction,
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
            function,
            signed_width,
            name,
            args,
            semi,
            items,
            end_function,
        })
    }
}

fn test_parse<T: Parse>(text: impl AsRef<str>) -> std::result::Result<T, miette::Report> {
    let text = text.as_ref();
    syn::parse_str::<T>(text).map_err(|err| syn_miette::Error::new(err, text).into())
}

fn main() -> miette::Result<()> {
    // Get a string of text
    let test = "41'sd2332";
    let foo = test_parse::<LitVerilog>(test)?;
    eprintln!("Foo {foo:#?}");
    let test = "[3:0]";
    let width = test_parse::<WidthSpec>(test)?;
    eprintln!("width {width:#?}");
    let test = "input reg signed [3:0] nibble";
    let port = test_parse::<Port>(test)?;
    eprintln!("Port {port:#?}");
    let expr = "d+3+~(^4+4*c*6%8&5)";
    let expr = test_parse::<Expr>(expr)?;
    eprintln!("expr {expr:#?}");
    let expr = "a > 3 ? 1 : 7";
    let expr = test_parse::<Expr>(expr)?;
    eprintln!("{expr:#?}");
    let expr = "{a, 3, 1}";
    let expr = test_parse::<Expr>(expr)?;
    eprintln!("{expr:#?}");
    let expr = test_parse::<Expr>("a + {4 {w}}")?;
    eprintln!("{expr:#?}");
    let expr = test_parse::<Expr>("a[3] + b[5:2] - {4 {w}}")?;
    eprintln!("{expr:#?}");
    let expr = test_parse::<Expr>("h[a +: 3]")?;
    eprintln!("{expr:#?}");
    let expr = test_parse::<Expr>("$signed(a)")?;
    eprintln!("{expr:#?}");
    let stmt = test_parse::<Stmt>(
        r"
begin
   if (a > 3)
      b = 4;
   else
      c = b;
end    
",
    )?;
    let stmt = test_parse::<Stmt>(
        r"
case (rega)
16'd0: result = 10'b0111111111;
16'd1: result = 10'b1011111111;
16'd2: result = 10'b1101111111;
16'd3: result = 10'b1110111111;
16'd4: result = 10'b1111011111;
16'd5: result = 10'b1111101111;
16'd6: result = 10'b1111110111;
16'd7: result = 10'b1111111011;
16'd8: result = 10'b1111111101;
16'd9: result = 10'b1111111110;
default result = 10'bx;
endcase
        ",
    )?;

    let dff = r"
        module foo(input wire[2:0] clock_reset, input wire[7:0] i, output wire[7:0] o);
        endmodule        
";
    let foo = test_parse::<ModuleDef>(dff)?;
    let dff = r"
        module foo(input wire[2:0] clock_reset, input wire[7:0] i, output wire[7:0] o);
           wire [0:0] clock;
           wire [0:0] reset;
           assign clock = clock_reset[0];
           assign wire = clock_reset[1];
           always @(posedge clock) begin
               if (reset) begin
                  o <= 8'b0;
                end else begin
                   o <= i;
                end
           end
        endmodule        
";
    let foo = test_parse::<ModuleDef>(dff)?;
    let add = r"
module dut(input wire [7:0] arg_0, output reg [7:0] out);
    reg [0:0] r0;
    reg [0:0] r1;
    reg [0:0] r2;
    reg [0:0] r3;
    reg [0:0] r4;
    reg [0:0] r5;
    reg [0:0] r6;
    reg [0:0] r7;
    reg [0:0] r8;
    reg [0:0] r9;
    reg [0:0] r10;
    reg [0:0] r11;
    reg [0:0] r12;
    reg [0:0] r13;
    reg [0:0] r14;
    reg [0:0] r15;
    always @(*) begin
        r0 = arg_0[0];
        r1 = arg_0[1];
        r2 = arg_0[2];
        r3 = arg_0[3];
        r4 = arg_0[4];
        r5 = arg_0[5];
        r6 = arg_0[6];
        r7 = arg_0[7];
        // let b = Bar/* tuple::Bar */ {a: bits(1), b: Foo/* tuple::Foo */ {a: bits(2), b: bits(3),},};
        //
        // let Bar {a: a, b: Foo {a: c, b: d,},} = b;
        //
        // signal((a + c + d + a0.val()).resize())
        //
        { r15,r14,r13,r12,r11,r10,r9,r8 } = { 1'b0, 1'b0, 1'b0, 1'b0, 1'b0, 1'b1, 1'b1, 1'b0 } + { r7, r6, r5, r4, r3, r2, r1, r0 };
        out = { r15, r14, r13, r12, r11, r10, r9, r8 };
    end
endmodule
    ";
    let foo = test_parse::<ModuleDef>(add)?;
    let foo = test_parse::<ModuleDef>(
        r"
    module dut(input wire [7:0] arg_0, input wire [7:0] arg_1, output reg [0:0] out);
    reg [0:0] r0;
    reg [0:0] r1;
    reg [0:0] r2;
    reg [0:0] r3;
    reg [0:0] r4;
    reg [0:0] r5;
    reg [0:0] r6;
    reg [0:0] r7;
    reg [0:0] r8;
    reg [0:0] r9;
    reg [0:0] r10;
    reg [0:0] r11;
    reg [0:0] r12;
    reg [0:0] r13;
    reg [0:0] r14;
    reg [0:0] r15;
    reg [0:0] r16;
    always @(*) begin
        r0 = arg_0[0];
        r1 = arg_0[1];
        r2 = arg_0[2];
        r3 = arg_0[3];
        r4 = arg_0[4];
        r5 = arg_0[5];
        r6 = arg_0[6];
        r7 = arg_0[7];
        r8 = arg_1[0];
        r9 = arg_1[1];
        r10 = arg_1[2];
        r11 = arg_1[3];
        r12 = arg_1[4];
        r13 = arg_1[5];
        r14 = arg_1[6];
        r15 = arg_1[7];
        // signal(a.val() >= b.val())
        //
        { r16 } = $signed({ r7, r6, r5, r4, r3, r2, r1, r0 }) >= $signed({ r15, r14, r13, r12, r11, r10, r9, r8 });
        out = { r16 };
    end
endmodule",
    )?;
    let foo = include_str!("../vlog/controller.v");
    let error = test_parse::<ModuleList>(foo)?;
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
        let Ok(module) = std::fs::read(entry.path()) else {
            continue;
        };
        let text = String::from_utf8_lossy(&module);
        let _ = test_parse::<ModuleList>(text)?;
    }
    Ok(())
}
