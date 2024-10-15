use crate::{
    rhif::spec::{AluBinary, AluUnary},
    rtl::object::RegisterKind,
    types::bit_string::BitString,
};

use super::formatter;

#[derive(Debug, Clone, Hash, Default)]
pub struct Module {
    pub name: String,
    pub ports: Vec<Port>,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
    pub functions: Vec<Function>,
}

impl Module {
    pub fn as_verilog(&self) -> String {
        formatter::module(self)
    }
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

impl Function {
    pub fn as_verilog(&self) -> String {
        formatter::function(self)
    }
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

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum SignedWidth {
    Unsigned(usize),
    Signed(usize),
}

impl SignedWidth {
    pub fn is_empty(&self) -> bool {
        matches!(self, SignedWidth::Unsigned(0) | SignedWidth::Signed(0))
    }
    pub fn len(&self) -> usize {
        match self {
            SignedWidth::Unsigned(len) => *len,
            SignedWidth::Signed(len) => *len,
        }
    }
    pub fn is_signed(&self) -> bool {
        matches!(self, SignedWidth::Signed(_))
    }
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
    pub kind: HDLKind,
    pub width: SignedWidth,
}

pub fn port(name: &str, direction: Direction, kind: HDLKind, width: SignedWidth) -> Port {
    Port {
        name: name.to_string(),
        direction,
        kind,
        width,
    }
}

#[derive(Debug, Clone, Hash, Copy, PartialEq)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, Hash, Copy, PartialEq)]
pub enum HDLKind {
    Wire,
    Reg,
}

#[derive(Debug, Clone, Hash)]
pub struct Declaration {
    pub kind: HDLKind,
    pub name: String,
    pub width: SignedWidth,
    pub alias: Option<String>,
}

pub fn declaration(
    kind: HDLKind,
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
    declaration(HDLKind::Reg, name, width, None)
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

pub fn non_blocking_assignment(target: &str, source: Box<Expression>) -> Statement {
    Statement::NonblockingAssignment(Assignment {
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

pub fn component_instance(
    name: &str,
    instance_name: &str,
    connections: Vec<Connection>,
) -> Statement {
    Statement::ComponentInstance(ComponentInstance {
        name: name.to_string(),
        instance_name: instance_name.to_string(),
        connections,
    })
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
    Identifier(String),
    Literal(BitString),
    Unary(Unary),
    Select(Select),
    Binary(Binary),
    Concat(Vec<Box<Expression>>),
    DynamicIndex(DynamicIndex),
    Index(Index),
    Repeat(Repeat),
    Const(bool),
}

pub fn bit_string(value: &BitString) -> Box<Expression> {
    Box::new(Expression::Literal(value.clone()))
}

pub fn binary(op: AluBinary, left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary(Binary {
        operator: op,
        left,
        right,
    }))
}

pub fn concatenate(expressions: Vec<Box<Expression>>) -> Box<Expression> {
    Box::new(Expression::Concat(expressions))
}

#[derive(Debug, Clone, Hash)]
pub struct Repeat {
    pub target: Box<Expression>,
    pub count: usize,
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
    pub target: String,
    pub range: std::ops::Range<usize>,
}

pub fn index(target: &str, range: std::ops::Range<usize>) -> Box<Expression> {
    Box::new(Expression::Index(Index {
        target: target.into(),
        range,
    }))
}

pub fn index_bit(target: &str, bit: usize) -> Box<Expression> {
    index(target, bit..(bit + 1))
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicIndex {
    pub argument: String,
    pub offset: Box<Expression>,
    pub len: usize,
}

/*
    self.func
        .push_str(&format!("    {lhs} = {arg}[{offset} +: {len}];\n",));
*/

pub fn dynamic_index(argument: &str, offset: Box<Expression>, len: usize) -> Box<Expression> {
    Box::new(Expression::DynamicIndex(DynamicIndex {
        argument: argument.into(),
        offset,
        len,
    }))
}

#[derive(Debug, Clone, Hash)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Box<Expression>>,
}

#[derive(Debug, Clone, Hash)]
pub enum Statement {
    ContinuousAssignment(Assignment),
    ComponentInstance(ComponentInstance),
    Assignment(Assignment),
    DynamicSplice(DynamicSplice),
    Initial(Initial),
    Splice(Splice),
    Case(Case),
    Always(Always),
    NonblockingAssignment(Assignment),
    If(If),
    Delay(usize),
    Display(Display),
    Finish,
    Assert(Assert),
}

#[derive(Debug, Clone, Hash)]
pub struct Assert {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub ndx: usize,
}

pub fn assert(left: Box<Expression>, right: Box<Expression>, ndx: usize) -> Statement {
    Statement::Assert(Assert { left, right, ndx })
}

pub fn finish() -> Statement {
    Statement::Finish
}

#[derive(Debug, Clone, Hash)]
pub struct Display {
    pub format: String,
    pub args: Vec<Box<Expression>>,
}

pub fn display(format: &str, args: Vec<Box<Expression>>) -> Statement {
    Statement::Display(Display {
        format: format.to_string(),
        args,
    })
}

pub fn delay(time: usize) -> Statement {
    Statement::Delay(time)
}

#[derive(Debug, Clone, Hash)]
pub struct If {
    pub condition: Box<Expression>,
    pub true_expr: Vec<Statement>,
    pub false_expr: Vec<Statement>,
}

pub fn if_statement(
    condition: Box<Expression>,
    true_expr: Vec<Statement>,
    false_expr: Vec<Statement>,
) -> Statement {
    Statement::If(If {
        condition,
        true_expr,
        false_expr,
    })
}

#[derive(Debug, Clone, Hash)]
pub struct Always {
    pub sensitivity: Vec<Events>,
    pub block: Vec<Statement>,
}

pub fn always(sensitivity: Vec<Events>, block: Vec<Statement>) -> Statement {
    Statement::Always(Always { sensitivity, block })
}

#[derive(Debug, Clone, Hash)]
pub enum Events {
    Posedge(String),
    Negedge(String),
    Change(String),
    Star,
}

pub fn continuous_assignment(target: &str, source: Box<Expression>) -> Statement {
    Statement::ContinuousAssignment(Assignment {
        target: target.to_string(),
        source,
    })
}

#[derive(Debug, Clone, Hash)]
pub struct Initial {
    pub block: Vec<Statement>,
}

pub fn initial(block: Vec<Statement>) -> Statement {
    Statement::Initial(Initial { block })
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicSplice {
    pub lhs: String,
    pub arg: Box<Expression>,
    pub offset: Box<Expression>,
    pub value: Box<Expression>,
    pub len: usize,
}

pub fn dynamic_splice(
    lhs: &str,
    arg: Box<Expression>,
    offset: Box<Expression>,
    value: Box<Expression>,
    len: usize,
) -> Statement {
    Statement::DynamicSplice(DynamicSplice {
        lhs: lhs.to_string(),
        arg,
        offset,
        value,
        len,
    })
}
/*
        self.func.push_str(&format!(
            "    {lhs} = {arg}; {lhs}[{offset} +: {len}] = {value};\n",
        ));

*/

#[derive(Debug, Clone, Hash)]
pub struct Unary {
    pub operator: AluUnary,
    pub operand: Box<Expression>,
}

/*

        match &unary.op {
            AluUnary::Signed => {
                self.func
                    .push_str(&format!("    {lhs} = $signed({arg1});\n",));
            }
            AluUnary::Unsigned => {
                self.func
                    .push_str(&format!("    {lhs} = $unsigned({arg1});\n",));
            }
            _ => {
                let op = unary.op.verilog_unop();
                self.func.push_str(&format!("    {lhs} = {op}{arg1};\n",));
            }
        }

*/

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
    pub source: Box<Expression>,
    pub replace_range: std::ops::Range<usize>,
    pub value: Box<Expression>,
}

pub fn splice(
    target: &str,
    source: Box<Expression>,
    replace_range: std::ops::Range<usize>,
    value: Box<Expression>,
) -> Statement {
    Statement::Splice(Splice {
        target: target.to_string(),
        source,
        replace_range,
        value,
    })
}

#[derive(Debug, Clone, Hash)]
pub struct Select {
    pub condition: Box<Expression>,
    pub true_expr: Box<Expression>,
    pub false_expr: Box<Expression>,
}

pub fn select(
    condition: Box<Expression>,
    true_expr: Box<Expression>,
    false_expr: Box<Expression>,
) -> Box<Expression> {
    Box::new(Expression::Select(Select {
        condition,
        true_expr,
        false_expr,
    }))
}

#[derive(Debug, Clone, Hash)]
pub struct Case {
    pub discriminant: Box<Expression>,
    pub cases: Vec<(CaseItem, Statement)>,
}

#[derive(Debug, Clone, Hash)]
pub enum CaseItem {
    Literal(BitString),
    Wild,
}

pub fn case(discriminant: Box<Expression>, cases: Vec<(CaseItem, Statement)>) -> Statement {
    Statement::Case(Case {
        discriminant,
        cases,
    })
}
