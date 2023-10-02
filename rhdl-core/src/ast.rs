#[derive(Debug, Clone)]
pub enum Stmt {
    Local(Local),
    Expr(Expr),
    Semi(Expr),
}

#[derive(Debug, Clone)]
pub struct Block(pub Vec<Stmt>);

#[derive(Debug, Clone)]
pub struct Local {
    pub pattern: Pattern,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(PatternIdent),
    Tuple(Vec<Pattern>),
    TupleStruct(PatternTupleStruct),
    Lit(ExprLit),
    Or(Vec<Pattern>),
    Paren(Box<Pattern>),
    Path(ExprPath),
}

#[derive(Debug, Clone)]
pub struct PatternTupleStruct {
    pub path: Box<Expr>,
    pub elems: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct PatternIdent {
    pub name: String,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(ExprBinary),
    Unary(ExprUnary),
    Match(ExprMatch),
    Return(Option<Box<Expr>>),
    If(ExprIf),
    Index(ExprIndex),
    Lit(ExprLit),
    Paren(Box<Expr>),
    Tuple(Vec<Expr>),
    ForLoop(ExprForLoop),
    While(ExprWhile),
    Assign(ExprAssign),
    Group(Box<Expr>),
    Field(ExprField),
    Block(Block),
    Array(ExprArray),
    Range(ExprRange),
    Path(ExprPath),
    Let(ExprLet),
    Repeat(ExprRepeat),
    Struct(ExprStruct),
    Call(ExprCall),
}

#[derive(Debug, Clone)]
pub struct ExprCall {
    pub path: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprPath {
    pub path: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExprStruct {
    pub path: Box<Expr>,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprWhile {
    pub cond: Box<Expr>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct ExprRepeat {
    pub value: Box<Expr>,
    pub len: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprLet {
    pub pattern: Pattern,
    pub value: Box<Expr>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprArray {
    pub elems: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprField {
    pub expr: Box<Expr>,
    pub member: Member,
}

#[derive(Debug, Clone)]
pub struct FieldValue {
    pub member: Member,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Debug, Clone)]
pub struct ExprAssign {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    AddAssign,
    SubAssign,
    MulAssign,
    BitXorAssign,
    BitAndAssign,
    BitOrAssign,
    ShlAssign,
    ShrAssign,
}

#[derive(Debug, Clone)]
pub struct ExprUnary {
    pub op: UnOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub struct ExprIf {
    pub cond: Box<Expr>,
    pub then_branch: Block,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct ExprMatch {
    pub expr: Box<Expr>,
    pub arms: Vec<Arm>,
}

#[derive(Debug, Clone)]
pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum ExprLit {
    Int(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprForLoop {
    pub pat: Box<Pattern>,
    pub expr: Box<Expr>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct ExprRange {
    pub start: Option<Box<Expr>>,
    pub limits: RangeLimits,
    pub end: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub enum RangeLimits {
    HalfOpen,
    Closed,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_fn() {
        fn jnk() -> Vec<Stmt> {
            vec![
                Stmt::Local(Local {
                    pattern: Pattern::Ident(PatternIdent {
                        name: stringify!(a).to_string(),
                        mutable: false,
                    }),
                    value: Box::new(Expr::Lit(ExprLit::Int("1".to_string()))),
                }),
                Stmt::Local(Local {
                    pattern: Pattern::Ident(PatternIdent {
                        name: stringify!(b).to_string(),
                        mutable: false,
                    }),
                    value: Box::new(Expr::Lit(ExprLit::Int("2".to_string()))),
                }),
                Stmt::Local(Local {
                    pattern: Pattern::Ident(PatternIdent {
                        name: stringify!(c).to_string(),
                        mutable: false,
                    }),
                    value: Box::new(Expr::Binary(ExprBinary {
                        op: BinOp::Add,
                        lhs: Box::new(Expr::Path(ExprPath {
                            path: vec![stringify!(a).to_string()],
                        })),
                        rhs: Box::new(Expr::Path(ExprPath {
                            path: vec![stringify!(b).to_string()],
                        })),
                    })),
                }),
                Stmt::Expr(Expr::Path(ExprPath {
                    path: vec![stringify!(c).to_string()],
                })),
            ]
        }
        println!("{:#?}", jnk());
    }
}
