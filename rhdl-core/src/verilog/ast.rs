use crate::{
    rhif::spec::{AluBinary, AluUnary},
    rtl::object::RegisterKind,
    types::bit_string::BitString,
};

#[derive(Debug, Clone, Hash, Default)]
pub struct Module {
    pub name: String,
    pub ports: Vec<Port>,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

// function {signed} [width-1:0] name(args);

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

pub fn literal(name: &str, value: &BitString) -> Literals {
    Literals {
        name: name.to_string(),
        value: value.clone(),
    }
}

fn verilog_literal(bs: &BitString) -> String {
    let signed = if bs.is_signed() { "s" } else { "" };
    let width = bs.len();
    let bs = binary_string(bs.bits());
    format!("{width}'{signed}b{bs}")
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum SignedWidth {
    Unsigned(usize),
    Signed(usize),
}

impl From<RegisterKind> for SignedWidth {
    fn from(kind: RegisterKind) -> Self {
        match kind {
            RegisterKind::Signed(len) => SignedWidth::Signed(len),
            RegisterKind::Unsigned(len) => SignedWidth::Unsigned(len),
        }
    }
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

// TODO - add check that len > 0 or redefine it as numbits - 1
fn reg_decl(name: &str, kind: RegisterKind) -> String {
    assert!(!kind.is_empty());
    match kind {
        RegisterKind::Signed(len) => format!("reg signed [{}:0] {}", len - 1, name),
        RegisterKind::Unsigned(len) => format!("reg [{}:0] {}", len - 1, name),
    }
}

pub fn declaration(
    kind: Kind,
    name: &str,
    width: SignedWidth,
    alias: Option<String>,
) -> Declaration {
    Declaration {
        kind,
        name: name.to_string(),
        width,
        alias,
    }
}

pub fn input_reg(name: &str, width: SignedWidth) -> Declaration {
    declaration(Kind::Reg, name, width, None)
}

#[derive(Debug, Clone, Hash)]
pub struct Assignment {
    pub target: String,
    pub source: Box<Expression>,
}

pub fn assign(target: &str, source: Box<Expression>) -> Statement {
    Statement::Assignment(Assignment {
        target: target.to_string(),
        source,
    })
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
    Concat(Vec<Box<Expression>>),
    DynamicIndex(DynamicIndex),
    Index(Index),
    Repeat(Repeat),
    Const(bool),
}

pub fn concatenate(expressions: Vec<Box<Expression>>) -> Box<Expression> {
    Box::new(Expression::Concat(expressions))
}

#[derive(Debug, Clone, Hash)]
pub struct Repeat {
    target: Box<Expression>,
    count: usize,
}

pub fn constant(value: bool) -> Box<Expression> {
    Box::new(Expression::Const(value))
}

pub fn repeat(target: Box<Expression>, count: usize) -> Box<Expression> {
    Box::new(Expression::Repeat(Repeat { target, count }))
}

pub fn function_call(name: &str, arguments: Vec<Box<Expression>>) -> Box<Expression> {
    Box::new(Expression::FunctionCall(FunctionCall {
        name: name.to_string(),
        arguments,
    }))
}

pub fn id(name: &str) -> Box<Expression> {
    Box::new(Expression::Identifier(name.to_string()))
}

#[derive(Debug, Clone, Hash)]
pub struct Index {
    pub target: Box<Expression>,
    pub range: std::ops::Range<usize>,
}

pub fn index(target: Box<Expression>, range: std::ops::Range<usize>) -> Box<Expression> {
    Box::new(Expression::Index(Index { target, range }))
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
    pub arguments: Vec<Box<Expression>>,
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

pub fn continuous_assignment(target: &str, source: Box<Expression>) -> Statement {
    Statement::ContinuousAssignment(Assignment {
        target: target.to_string(),
        source,
    })
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

pub fn unary(operator: AluUnary, operand: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Unary(Unary { operator, operand }))
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
