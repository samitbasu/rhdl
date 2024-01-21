use crate::ast::ast_impl;

// Used to construct a chain of descriptions for why a
// register was allocated.
#[derive(Clone, Debug)]
pub enum Description {
    LocalBinding(String),
    Rebinding(Box<Description>),
    UnopTarget(ast_impl::UnOp, Box<Description>),
    UnopArgument(ast_impl::UnOp, Box<Description>),
    TupleInitLeftHandSide(usize, Box<Description>),
    TupleInitRightHandSide(usize, Box<Description>),
    StructInitLeftHandSide(ast_impl::Member, Box<Description>),
    StructInitRightHandSide(ast_impl::Member, Box<Description>),
    TupleStructInitLeftHandSide(usize, Box<Description>),
    TupleStructInitRightHandSide(usize, Box<Description>),
    SliceInitLeftHandSide(usize, Box<Description>),
    SliceInitRightHandSide(usize, Box<Description>),
    Statement(String, Box<Description>),
    IfStatementResult(Box<Description>),
    IfStatementCondition(Box<Description>),
    IfStatementThenBranch(Box<Description>),
    IfStatementElseBranch(Box<Description>),
    IfStatementResultOfThenBranch(Box<Description>),
    IfStatementResultOfElseBranch(Box<Description>),
    ExpressionListItem(usize, Box<Description>),
    TupleLeftHandSide(Box<Description>),
    TupleRightHandSide(Box<Description>),
    IndexExpressionLeftHandSide(Box<Description>),
    IndexExpressionTarget(Box<Description>),
    IndexExpressionIndex(Box<Description>),
    ArrayExpressionLeftHandSide(Box<Description>),
    ArrayExpressionRightHandSide(Box<Description>),
    StructFieldExpressionLeftHandSide(ast_impl::Member, Box<Description>),
    StructFieldExpressionTarget(ast_impl::Member, Box<Description>),
}

pub fn describe_local_binding(name: &str) -> Description {
    Description::LocalBinding(name.to_string())
}

pub fn describe_rebinding(description: &Description) -> Description {
    Description::Rebinding(Box::new(description.clone()))
}

pub fn describe_unop_target(op: ast_impl::UnOp, description: &Description) -> Description {
    Description::UnopTarget(op, Box::new(description.clone()))
}

pub fn describe_unop_argument(op: ast_impl::UnOp, description: &Description) -> Description {
    Description::UnopArgument(op, Box::new(description.clone()))
}

pub fn describe_tuple_init_left_hand_side(index: usize, description: &Description) -> Description {
    Description::TupleInitLeftHandSide(index, Box::new(description.clone()))
}

pub fn describe_tuple_init_right_hand_side(index: usize, description: &Description) -> Description {
    Description::TupleInitRightHandSide(index, Box::new(description.clone()))
}

pub fn describe_struct_init_left_hand_side(
    member: &ast_impl::Member,
    description: &Description,
) -> Description {
    Description::StructInitLeftHandSide(member.clone(), Box::new(description.clone()))
}

pub fn describe_struct_init_right_hand_side(
    member: &ast_impl::Member,
    description: &Description,
) -> Description {
    Description::StructInitRightHandSide(member.clone(), Box::new(description.clone()))
}

pub fn describe_tuple_struct_init_left_hand_side(
    index: usize,
    description: &Description,
) -> Description {
    Description::TupleStructInitLeftHandSide(index, Box::new(description.clone()))
}

pub fn describe_tuple_struct_init_right_hand_side(
    index: usize,
    description: &Description,
) -> Description {
    Description::TupleStructInitRightHandSide(index, Box::new(description.clone()))
}

pub fn describe_slice_init_left_hand_side(index: usize, description: &Description) -> Description {
    Description::SliceInitLeftHandSide(index, Box::new(description.clone()))
}

pub fn describe_slice_init_right_hand_side(index: usize, description: &Description) -> Description {
    Description::SliceInitRightHandSide(index, Box::new(description.clone()))
}

pub fn describe_statement(statement: &str, description: &Description) -> Description {
    Description::Statement(statement.to_string(), Box::new(description.clone()))
}

pub fn describe_if_statement_result(description: &Description) -> Description {
    Description::IfStatementResult(Box::new(description.clone()))
}

pub fn describe_if_statement_condition(description: &Description) -> Description {
    Description::IfStatementCondition(Box::new(description.clone()))
}

pub fn describe_if_statement_then_branch(description: &Description) -> Description {
    Description::IfStatementThenBranch(Box::new(description.clone()))
}

pub fn describe_if_statement_else_branch(description: &Description) -> Description {
    Description::IfStatementElseBranch(Box::new(description.clone()))
}

pub fn describe_if_statement_result_of_then_branch(description: &Description) -> Description {
    Description::IfStatementResultOfThenBranch(Box::new(description.clone()))
}

pub fn describe_if_statement_result_of_else_branch(description: &Description) -> Description {
    Description::IfStatementResultOfElseBranch(Box::new(description.clone()))
}

pub fn describe_expression_list_item(index: usize, description: &Description) -> Description {
    Description::ExpressionListItem(index, Box::new(description.clone()))
}

pub fn describe_tuple_left_hand_side(description: &Description) -> Description {
    Description::TupleLeftHandSide(Box::new(description.clone()))
}

pub fn describe_tuple_right_hand_side(description: &Description) -> Description {
    Description::TupleRightHandSide(Box::new(description.clone()))
}

pub fn describe_index_expression_left_hand_side(description: &Description) -> Description {
    Description::IndexExpressionLeftHandSide(Box::new(description.clone()))
}

pub fn describe_index_expression_target(description: &Description) -> Description {
    Description::IndexExpressionTarget(Box::new(description.clone()))
}

pub fn describe_index_expression_index(description: &Description) -> Description {
    Description::IndexExpressionIndex(Box::new(description.clone()))
}

pub fn describe_array_expression_left_hand_side(description: &Description) -> Description {
    Description::ArrayExpressionLeftHandSide(Box::new(description.clone()))
}

pub fn describe_array_expression_right_hand_side(description: &Description) -> Description {
    Description::ArrayExpressionRightHandSide(Box::new(description.clone()))
}

pub fn describe_struct_field_expression_left_hand_side(
    member: &ast_impl::Member,
    description: &Description,
) -> Description {
    Description::StructFieldExpressionLeftHandSide(member.clone(), Box::new(description.clone()))
}

pub fn describe_struct_field_expression_target(
    member: &ast_impl::Member,
    description: &Description,
) -> Description {
    Description::StructFieldExpressionTarget(member.clone(), Box::new(description.clone()))
}
