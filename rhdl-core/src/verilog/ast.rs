use crate::{
    rhif::spec::{AluBinary, AluUnary},
    types::bit_string::BitString,
};

#[derive(Debug, Clone, Hash, Default)]
pub struct Module {
    pub name: String,
    pub ports: Vec<Port>,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Hash)]
pub struct Function {
    pub name: String,
    pub width: SignedWidth,
    pub arguments: Vec<Declaration>,
    pub registers: Vec<Declaration>,
    pub literals: Vec<Literals>,
    pub block: Vec<Statement>,
}

#[derive(Debug, Clone, Hash)]
pub struct Literals {
    pub name: String,
    pub value: BitString,
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum SignedWidth {
    Unsigned(usize),
    Signed(usize),
}

pub fn signed_width(width: usize) -> SignedWidth {
    SignedWidth::Signed(width)
}

pub fn unsigned_width(width: usize) -> SignedWidth {
    SignedWidth::Unsigned(width)
}

#[derive(Debug, Clone, Hash)]
pub struct Port {
    pub name: String,
    pub direction: Direction,
    pub kind: Kind,
    pub width: SignedWidth,
}

pub fn port(name: &str, direction: Direction, kind: Kind, width: SignedWidth) -> Port {
    Port {
        name: name.to_string(),
        direction,
        kind,
        width,
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, Hash)]
pub enum Kind {
    Wire,
    Reg,
}

#[derive(Debug, Clone, Hash)]
pub struct Declaration {
    pub kind: Kind,
    pub name: String,
    pub width: SignedWidth,
    pub alias: Option<String>,
}

pub fn declaration(kind: Kind, name: &str, width: SignedWidth, alias: Option<&str>) -> Declaration {
    Declaration {
        kind,
        name: name.to_string(),
        width,
        alias: alias.map(|s| s.to_string()),
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Assignment {
    pub target: String,
    pub source: Expression,
}

#[derive(Debug, Clone, Hash, Default)]
pub struct ComponentInstance {
    pub name: String,
    pub instance_name: String,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone, Hash)]
pub struct Connection {
    pub target: String,
    pub source: Box<Expression>,
}

pub fn connection(target: &str, source: Box<Expression>) -> Connection {
    Connection {
        target: target.to_string(),
        source,
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Expression {
    FunctionCall(FunctionCall),
    BitRange(BitRange),
    Identifier(String),
    Literal(BitString),
    Unary(Unary),
    Splice(Splice),
    Select(Select),
    Binary(Binary),
    Concat(Vec<Expression>),
    DynamicIndex(DynamicIndex),
    Index(Index),
}

pub fn id(name: &str) -> Box<Expression> {
    Box::new(Expression::Identifier(name.to_string()))
}

#[derive(Debug, Clone, Hash)]
pub struct Index {
    pub target: Box<Expression>,
    pub msb: Option<usize>,
    pub lsb: usize,
}

pub fn index(target: Box<Expression>, lsb: usize, msb: Option<usize>) -> Index {
    Index { target, lsb, msb }
}

pub fn index_range(target: Box<Expression>, range: std::ops::Range<usize>) -> Box<Expression> {
    Box::new(Expression::Index(Index {
        target,
        lsb: range.start,
        msb: Some(range.end.saturating_sub(1)),
    }))
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicIndex {
    pub target: Box<Expression>,
    pub offset: Box<Expression>,
    pub len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, Hash)]
pub struct BitRange {
    pub target: Box<Expression>,
    pub start: Box<Expression>,
    pub end: Box<Expression>,
}

#[derive(Debug, Clone, Hash)]
pub enum Statement {
    ContinuousAssignment(Assignment),
    ComponentInstance(ComponentInstance),
    Assignment(Assignment),
    DynamicSplice(DynamicSplice),
    Initial(Initial),
}

#[derive(Debug, Clone, Hash)]
pub struct Initial {}

#[derive(Debug, Clone, Hash)]
pub struct DynamicSplice {
    pub lhs: String,
    pub arg: String,
    pub offset: String,
    pub value: String,
    pub len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct Unary {
    pub operator: AluUnary,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, Hash)]
pub struct Binary {
    pub operator: AluBinary,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, Hash)]
pub struct Splice {
    pub target: String,
    pub source: String,
    pub start: usize,
    pub end: usize,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, Hash)]
pub struct Select {
    pub condition: Box<Expression>,
    pub true_expr: Box<Expression>,
    pub false_expr: Box<Expression>,
}

#[derive(Debug, Clone, Hash)]
pub struct Case {
    pub discriminant: Box<Expression>,
    pub cases: Vec<(CaseItem, Assignment)>,
}

#[derive(Debug, Clone, Hash)]
pub enum CaseItem {
    Literal(BitString),
    Wild,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module() {
        let module = Module {
            name: "test".to_string(),
            ports: vec![],
            declarations: vec![],
            statements: vec![],
        };
        assert_eq!(module.name, "test");
    }
}
