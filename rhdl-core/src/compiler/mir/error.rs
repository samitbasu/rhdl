use miette::Diagnostic;
use thiserror::Error;

use crate::{
    ast::ast_impl::{ExprCall, ExprPath, FunctionId, NodeId, Pat},
    ast_builder::BinOp,
    rhif::spec::Slot,
};

use super::compiler::ScopeIndex;

#[derive(Error, Debug, Diagnostic)]
pub enum ICE {
    #[error("Attempt to set local variable {name} that does not exist")]
    LocalVariableDoesNotExist { name: String },
    #[error("Argument pattern {arg:?} not supported")]
    UnsupportedArgumentPattern { arg: Box<Pat> },
    #[error("Rebind of unbound variable {name}")]
    RebindOfUnboundVariable { name: String },
    #[error("Calling slot-to-index mapping on non-literal slot {slot}")]
    SlotToIndexNonLiteralSlot { slot: Slot },
    #[error("Attempt to initialize unbound local variable {name}")]
    InitializeLocalOnUnboundVariable { name: String },
    #[error("Unsupported pattern in initialize local {pat:?}")]
    UnsupportedPatternInInitializeLocal { pat: Box<Pat> },
    #[error("No early return flag found in function {func}")]
    NoEarlyReturnFlagFound { func: FunctionId },
    #[error("Local variable {id:?} not found in branch map")]
    LocalVariableNotFoundInBranchMap { id: ScopeIndex },
    #[error("Return slot {name} not found")]
    ReturnSlotNotFound { name: String },
    #[error("Non self assign binary operation found in assign_binop code {op}")]
    NonSelfAssignBinop { op: BinOp },
    #[error("Unexpected binary op in self assign {op}")]
    UnexpectedBinopInSelfAssign { op: BinOp },
    #[error("No local variable found for pattern {pat:?} in type_pattern")]
    NoLocalVariableFoundForTypedPattern { pat: Box<Pat> },
    #[error("Unsupported pattern in type pattern {pat:?}")]
    UnsupportedPatternInTypePattern { pat: Box<Pat> },
    #[error("Unsupported pattern in bind pattern {pat:?}")]
    UnsupportedPatternInBindPattern { pat: Box<Pat> },
    #[error("Call made {call:?} to kernel with no code found")]
    CallToKernelWithNoCode { call: ExprCall },
    #[error("Missing local variable for binding {var:?} in then-branch")]
    MissingLocalVariableForBindingInThenBranch { var: ScopeIndex },
    #[error("Missing local variable for binding {var:?} in else-branch")]
    MissingLocalVariableForBindingInElseBranch { var: ScopeIndex },
    #[error("Missing local variable for binding {var:?} in match arm")]
    MissingLocalVariableForBindingInMatchArm { var: ScopeIndex },
    #[error("Name {name} not found in path {path:?}")]
    NameNotFoundInPath { name: String, path: ExprPath },
    #[error("Missing kernel function provided for {name}")]
    MissingKernelFunction { name: String },
}

#[derive(Error, Debug, Diagnostic)]
pub enum Syntax {
    #[error("Ranges are only supported in for loops")]
    RangesInForLoopsOnly(NodeId),
    #[error("Fallible let expressions currently unsupported")]
    FallibleLetExpr(NodeId),
    #[error("For loop with non-ident pattern is unsupported")]
    ForLoopNonIdentPattern(NodeId),
    #[error("For loop with non-range expression is not supported")]
    #[diagnostic(help("Use a range expression instead"))]
    ForLoopNonRangeExpr(NodeId),
    #[error("For loop without start value is not supported")]
    ForLoopNoStartValue(NodeId),
    #[error("For loop without end value is not supported")]
    ForLoopNoEndValue(NodeId),
    #[error("For loop with non-integer start value is not supported")]
    ForLoopNonIntegerStartValue(NodeId),
    #[error("For loop with non-integer end value is not supported")]
    #[diagnostic(help("Use an integer expression for the end value"))]
    ForLoopNonIntegerEndValue(NodeId),
    #[error("Unsupported method call")]
    UnsupportedMethodCall(NodeId),
    #[error("Unsupported path with arguments")]
    UnsupportedPathWithArguments(NodeId),
}
