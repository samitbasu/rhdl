use crate::{
    rhif::spec::{AluBinary, AluUnary},
    types::bit_string::BitString,
    util::binary_string,
};

use super::ast::{
    Assignment, Binary, Case, CaseItem, ComponentInstance, Connection, Declaration, DynamicIndex,
    DynamicSplice, Expression, Function, FunctionCall, Index, Kind, Literals, Repeat, Select,
    SignedWidth, Splice, Statement, Unary,
};

const VERILOG_INDENT_INCREASERS: [&str; 4] = ["module", "begin", "function", "case"];
const VERILOG_INDENT_DECREASERS: [&str; 4] = ["endmodule", "end", "endfunction", "endcase"];

fn kind(ast: &Kind) -> &'static str {
    match ast {
        Kind::Reg => "reg",
        Kind::Wire => "wire",
    }
}

fn signed_width(ast: &SignedWidth) -> String {
    match ast {
        SignedWidth::Unsigned(width) => format!("[{}:0]", width.saturating_sub(1)),
        SignedWidth::Signed(width) => format!("signed [{}:0]", width.saturating_sub(1)),
    }
}

fn argument(ast: &Declaration) -> String {
    format!(
        "input {} {} {}",
        kind(&ast.kind),
        signed_width(&ast.width),
        ast.name
    )
}

fn register(ast: &Declaration) -> String {
    let alias = ast
        .alias
        .as_ref()
        .map(|x| format!(" // {x}"))
        .unwrap_or_default();
    format!(
        "{} {} {}; {}",
        kind(&ast.kind),
        signed_width(&ast.width),
        ast.name,
        alias
    )
}

fn bit_string(bs: &BitString) -> String {
    let signed = if bs.is_signed() { "s" } else { "" };
    let width = bs.len();
    let bs = binary_string(bs.bits());
    format!("{width}'{signed}b{bs}")
}

fn literal(ast: &Literals) -> String {
    format!("localparam {} = {};", ast.name, bit_string(&ast.value))
}

fn continuous_assignment(ast: &Assignment) -> String {
    format!("assign {} = {};", ast.target, expression(&ast.source))
}

fn assignment(ast: &Assignment) -> String {
    format!("{} = {};", ast.target, expression(&ast.source))
}

fn connection(ast: &Connection) -> String {
    format!(".{}({})", ast.target, expression(&ast.source))
}

fn component_instance(ast: &ComponentInstance) -> String {
    let connections = ast
        .connections
        .iter()
        .map(connection)
        .collect::<Vec<_>>()
        .join(",");
    format!("{} {} ({});", ast.name, ast.instance_name, connections)
}

fn dynamic_splice(ast: &DynamicSplice) -> String {
    format!(
        "{lhs} = {arg}; {lhs}[{offset} +: {len}] = {value};",
        lhs = ast.lhs,
        arg = expression(&ast.arg),
        offset = expression(&ast.offset),
        len = ast.len,
        value = expression(&ast.value)
    )
}

fn splice(ast: &Splice) -> String {
    format!(
        "{lhs} = {source}; {lhs}[{end}:{start}] = {value};",
        lhs = ast.target,
        source = expression(&ast.source),
        start = ast.replace_range.start,
        end = ast.replace_range.end.saturating_sub(1),
        value = expression(&ast.value)
    )
}

fn case_item(ast: &CaseItem) -> String {
    match ast {
        CaseItem::Literal(literal) => bit_string(literal),
        CaseItem::Wild => "default".to_string(),
    }
}

fn case(ast: &Case) -> String {
    let case = ast
        .cases
        .iter()
        .map(|(cond, stmt)| format!("{}: {}", case_item(cond), statement(stmt)))
        .collect::<Vec<_>>()
        .join("\n");
    format!("case({})\n{}endcase\n", expression(&ast.discriminant), case)
}

fn statement(ast: &Statement) -> String {
    match ast {
        Statement::ContinuousAssignment(ast) => continuous_assignment(ast),
        Statement::ComponentInstance(ast) => component_instance(ast),
        Statement::Assignment(ast) => assignment(ast),
        Statement::DynamicSplice(ast) => dynamic_splice(ast),
        Statement::Initial(initial) => todo!(),
        Statement::Splice(ast) => splice(ast),
        Statement::Case(ast) => case(ast),
    }
}

fn binop(ast: AluBinary) -> &'static str {
    match ast {
        AluBinary::Add => "+",
        AluBinary::Sub => "-",
        AluBinary::Mul => "*",
        AluBinary::BitAnd => "&",
        AluBinary::BitOr => "|",
        AluBinary::BitXor => "^",
        AluBinary::Shl => "<<",
        AluBinary::Shr => ">>>",
        AluBinary::Eq => "==",
        AluBinary::Ne => "!=",
        AluBinary::Lt => "<",
        AluBinary::Le => "<=",
        AluBinary::Gt => ">",
        AluBinary::Ge => ">=",
    }
}

fn binary(ast: &Binary) -> String {
    format!(
        "{} {} {}",
        expression(&ast.left),
        binop(ast.operator),
        expression(&ast.right)
    )
}

fn unop(ast: AluUnary) -> &'static str {
    match ast {
        AluUnary::Neg => "-",
        AluUnary::Not => "~",
        AluUnary::All => "&",
        AluUnary::Any => "|",
        AluUnary::Xor => "^",
        AluUnary::Signed => "$signed",
        AluUnary::Unsigned => "$unsigned",
        AluUnary::Val => "",
    }
}

fn unary(ast: &Unary) -> String {
    format!("{}({})", unop(ast.operator), expression(&ast.operand))
}

fn select(ast: &Select) -> String {
    format!(
        "({}) ? ({}) : ({})",
        expression(&ast.condition),
        expression(&ast.true_expr),
        expression(&ast.false_expr)
    )
}

fn concatenate(ast: &[Box<Expression>]) -> String {
    let expr = ast
        .iter()
        .map(|expr| expression(expr))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{ {expr} }}",)
}

fn function_call(ast: &FunctionCall) -> String {
    let args = ast
        .arguments
        .iter()
        .map(|arg| expression(arg))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}({args})", ast.name,)
}

fn dynamic_index(ast: &DynamicIndex) -> String {
    format!(
        "{}[({}) +: {}]",
        &ast.target,
        expression(&ast.offset),
        ast.len
    )
}

fn index(ast: &Index) -> String {
    let start = ast.range.start;
    let end = ast.range.end.saturating_sub(1);
    format!("{}[{}:{}]", &ast.target, end, start)
}

fn repeat(ast: &Repeat) -> String {
    format!(
        "{{{count}{{{target}}}}}",
        count = ast.count,
        target = expression(&ast.target)
    )
}

fn expression(ast: &Expression) -> String {
    match ast {
        Expression::Binary(ast) => binary(ast),
        Expression::Unary(ast) => unary(ast),
        Expression::Literal(ast) => bit_string(ast),
        Expression::Identifier(ast) => ast.clone(),
        Expression::Select(ast) => select(ast),
        Expression::Concat(ast) => concatenate(ast),
        Expression::FunctionCall(ast) => function_call(ast),
        Expression::DynamicIndex(ast) => dynamic_index(ast),
        Expression::Index(ast) => index(ast),
        Expression::Repeat(ast) => repeat(ast),
        Expression::Const(ast) => {
            if *ast {
                "1'b1".to_string()
            } else {
                "1'b0".to_string()
            }
        }
    }
}

pub fn function(ast: &Function) -> String {
    let name = &ast.name;
    let signed_width = signed_width(&ast.width);
    let args = ast
        .arguments
        .iter()
        .map(|arg| argument(arg))
        .collect::<Vec<_>>()
        .join(",");
    let header = format!("function {signed_width} {name}({args});");
    let registers = ast
        .registers
        .iter()
        .map(register)
        .collect::<Vec<_>>()
        .join("\n");
    let literals = ast
        .literals
        .iter()
        .map(literal)
        .collect::<Vec<_>>()
        .join("\n");
    let statements = ast
        .block
        .iter()
        .map(statement)
        .collect::<Vec<_>>()
        .join("\n");
    format!("{header}\n{registers}\n{literals}\nbegin\n{statements}\nend\nendfunction",)
}
