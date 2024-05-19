use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
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
    #[diagnostic(help("You cannot use a range expression here in RHDL"))]
    RangesInForLoopsOnly,
    #[error("Fallible let expressions currently unsupported")]
    #[diagnostic(help("Use a match statement to handle fallible expressions"))]
    FallibleLetExpr,
    #[error("For loop with non-ident pattern is unsupported")]
    #[diagnostic(help("Use an ident pattern like `for x in 0..5`"))]
    ForLoopNonIdentPattern,
    #[error("For loop with non-range expression is not supported")]
    #[diagnostic(help("Use a literal integer range like 0..5 for the for loop range"))]
    ForLoopNonRangeExpr,
    #[error("For loop without start value is not supported")]
    #[diagnostic(help("Use a literal integer range like 0..5 for the for loop range"))]
    ForLoopNoStartValue,
    #[error("For loop without end value is not supported")]
    #[diagnostic(help("Use a literal integer range like 0..5 for the for loop range"))]
    ForLoopNoEndValue,
    #[error("For loop with non-integer start value is not supported")]
    #[diagnostic(help("Use a literal integer range like 0..5 for the for loop range"))]
    ForLoopNonIntegerStartValue,
    #[error("For loop with non-integer end value is not supported")]
    #[diagnostic(help("Use a literal integer range like 0..5 for the for loop range"))]
    ForLoopNonIntegerEndValue,
    #[error("Unsupported method call")]
    #[diagnostic(help(
        "Only .all(), .any(), .xor(), .as_unsigned() and .as_signed() are supported in kernels"
    ))]
    UnsupportedMethodCall,
    #[error("Unsupported path with arguments")]
    #[diagnostic(help("Use a path without generic arguments here, if possible"))]
    UnsupportedPathWithArguments,
}

#[derive(Debug, Error)]
#[error("RHDL Syntax Error")]
pub struct RHDLSyntaxError {
    pub cause: Syntax,
    pub src: String,
    pub err_span: SourceSpan,
}

impl Diagnostic for RHDLSyntaxError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.help()
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(std::iter::once(
            miette::LabeledSpan::new_primary_with_span(Some(self.cause.to_string()), self.err_span),
        )))
    }
}

#[derive(Debug, Error)]
#[error("RHDL Internal Compile Error")]
pub struct RHDLCompileError {
    pub cause: ICE,
    pub src: String,
    pub err_span: SourceSpan,
}

impl Diagnostic for RHDLCompileError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.help()
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(std::iter::once(
            miette::LabeledSpan::new_primary_with_span(Some(self.cause.to_string()), self.err_span),
        )))
    }
}
