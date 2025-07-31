use syn::{
    Ident, Lifetime, LitInt, Result, Token, braced, bracketed, parenthesized,
    parse::{Lookahead1, Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    token,
};

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

#[derive(Debug, Clone, Hash)]
pub struct Port {
    pub name: Ident,
    pub direction: Direction,
    pub kind: HDLKind,
    pub signed: Option<kw::signed>,
    pub width: WidthSpec,
}

impl Parse for Port {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let direction = input.parse()?;
        let kind = input.parse()?;
        let signed = input.parse()?;
        let width = input.parse()?;
        let name = input.parse()?;
        Ok(Self {
            name,
            direction,
            kind,
            signed,
            width,
        })
    }
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
pub enum Stmt {
    If(If),
    NonblockAssign(NonblockAssign),
    Always(Always),
    Case(Case),
    Assign(Assign),
    ContinuousAssign(ContinuousAssign),
    LocalParam(LocalParam),
    Splice(Splice),
    DynamicSplice(DynamicSplice),
    Instance(Instance),
    Block(Block),
}

#[derive(Debug, Clone)]
pub enum CaseItem {
    Literal(LitVerilog),
    Wild(kw::default),
}

#[derive(Debug, Clone)]
pub struct CaseLine {
    item: CaseItem,
    stmt: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Case {
    case: kw::case,
    discriminant: Box<Expr>,
    lines: Vec<CaseLine>,
    endcase: kw::endcase,
}

#[derive(Debug, Clone)]
pub struct Connection {
    dot: Token![.],
    target: Ident,
    local: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Instance {
    module: Ident,
    instance: Ident,
    connections: Punctuated<Connection, Token![,]>,
}

#[derive(Debug, Clone)]
pub struct Block {
    begin: kw::begin,
    body: Vec<Stmt>,
    end: kw::end,
}

#[derive(Debug, Clone)]
pub struct DynamicSplice {
    lhs: Box<ExprDynIndex>,
    eq: Token![=],
    rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Splice {
    lhs: Box<ExprIndex>,
    eq: Token![=],
    rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Sensitivity {
    PosEdge(PosEdgeSensitivity),
    NegEdge(NegEdgeSensitivity),
    Signal(Ident),
    Star(Token![*]),
}

#[derive(Debug, Clone)]
pub struct PosEdgeSensitivity {
    posedge: kw::posedge,
    ident: Ident,
}

#[derive(Debug, Clone)]
pub struct NegEdgeSensitivity {
    negedge: kw::negedge,
    ident: Ident,
}

#[derive(Debug, Clone)]
pub struct SensitivityList {
    at: Token![@],
    elements: Punctuated<Sensitivity, Token![,]>,
}

#[derive(Debug, Clone)]
pub struct Always {
    always: kw::always,
    sensitivity: SensitivityList,
    body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct LocalParam {
    pub localparam: kw::localparam,
    pub target: Ident,
    pub eq: Token![=],
    pub rhs: LitVerilog,
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Expr>,
    pub true_stmt: Box<Stmt>,
    pub false_stmt: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct NonblockAssign {
    pub target: Ident,
    pub larrow: LeftArrow,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ContinuousAssign {
    pub kw: kw::assign,
    pub assign: Assign,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub target: Ident,
    pub eq: Token![=],
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Literal(LitInt),
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
    eprintln!("exprbp: {input:?}, min_bp: {min_bp}");
    let lookahead = input.lookahead1();
    let mut lhs = if lookahead.peek(LitInt) {
        input.parse().map(Expr::Literal)?
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
    eprintln!("lhs = {lhs:?}");
    loop {
        if input.is_empty() {
            break;
        }
        eprintln!("input = {input:?}");
        // Check for a trinary operator
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![:]) || lookahead.peek(Token![,]) || lookahead.peek(PlusColon) {
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

#[derive(Debug, Clone)]
pub struct ExprFunction {
    pub dollar: Option<Token![$]>,
    pub name: Ident,
    pub args: Punctuated<Expr, Token![,]>,
}

impl Parse for ExprFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let dollar = input.parse()?;
        let name = input.parse()?;
        let content;
        parenthesized!(content in input);
        let args = content.parse_terminated(Expr::parse, Token![,])?;
        Ok(Self { dollar, name, args })
    }
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
}

impl Parse for BinaryOp {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<<]) {
            input.parse().map(BinaryOp::Shl)
        } else if lookahead.peek(Token![>>]) {
            input.parse().map(BinaryOp::Shr)
        } else if lookahead.peek(Token![&&]) {
            input.parse().map(BinaryOp::ShortAnd)
        } else if lookahead.peek(Token![||]) {
            input.parse().map(BinaryOp::ShortOr)
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
            BinaryOp::Shl(_) | BinaryOp::Shr(_) => (16, 17),
            BinaryOp::Ge(_) | BinaryOp::Le(_) | BinaryOp::Gt(_) | BinaryOp::Lt(_) => (14, 15),
            BinaryOp::Ne(_) | BinaryOp::Eq(_) => (12, 13),
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

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct BitRange {
    start: LitInt,
    colon: token::Colon,
    end: LitInt,
}

impl Parse for BitRange {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            start: input.parse()?,
            colon: input.parse()?,
            end: input.parse()?,
        })
    }
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

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum SignedWidth {
    Unsigned(usize),
    Signed(usize),
}

#[derive(Debug, Clone)]
struct LitVerilog {
    width: LitInt,
    lifetime: Lifetime,
}

#[derive(Debug)]
struct OperatorList {
    list: Punctuated<Operator, Token![,]>,
}

impl Parse for OperatorList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            list: Punctuated::parse_terminated(&input)?,
        })
    }
}

impl Parse for LitVerilog {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            width: input.parse()?,
            lifetime: input.parse()?,
        })
    }
}

fn main() -> Result<()> {
    // Get a string of text
    let test = "41'sd2332";
    let foo = syn::parse_str::<LitVerilog>(test)?;
    eprintln!("Foo {foo:#?}");
    let test = "[3:0]";
    let width = syn::parse_str::<WidthSpec>(test)?;
    eprintln!("width {width:#?}");
    let test = "input reg signed [3:0] nibble";
    let port = syn::parse_str::<Port>(test)?;
    eprintln!("Port {port:#?}");
    let operator_list = syn::parse_str::<OperatorList>("+,-,&&,&,||,<<,>>,%,!,~")?;
    eprintln!("ops {operator_list:#?}");
    let expr = "d+3+~(^4+4*c*6%8&5)";
    let expr = syn::parse_str::<Expr>(expr)?;
    eprintln!("expr {expr:#?}");
    let expr = "a > 3 ? 1 : 7";
    let expr = syn::parse_str::<Expr>(expr)?;
    eprintln!("{expr:#?}");
    let expr = "{a, 3, 1}";
    let expr = syn::parse_str::<Expr>(expr)?;
    eprintln!("{expr:#?}");
    let expr = syn::parse_str::<Expr>("a + {4 {w}}")?;
    eprintln!("{expr:#?}");
    let expr = syn::parse_str::<Expr>("a[3] + b[5:2] - {4 {w}}")?;
    eprintln!("{expr:#?}");
    let expr = syn::parse_str::<Expr>("h[a +: 3]")?;
    eprintln!("{expr:#?}");
    let expr = syn::parse_str::<Expr>("$signed(a)")?;
    eprintln!("{expr:#?}");
    Ok(())
}
