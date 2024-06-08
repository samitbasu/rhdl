use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::{
    ast::ast_impl::{ExprCall, ExprPath, FunctionId, Pat},
    ast_builder::BinOp,
    path::Path,
    rhif::spec::Slot,
    Kind,
};

use super::compiler::ScopeIndex;

#[derive(Error, Debug, Diagnostic)]
pub enum TypeCheck {
    #[error("A request was made for .val() on something that is not a signal")]
    ExpectedSignalValue,
    #[error("Literal with explicit type {typ:?} is inferred as {kind:?} instead")]
    InferredLiteralTypeMismatch { typ: Kind, kind: Kind },
    #[error("Unable to determine type of this item")]
    #[diagnostic(help("Please provide an explicit type annotation"))]
    UnableToDetermineType,
}

#[derive(Error, Debug, Diagnostic)]
pub enum ICE {
    #[error("Attempt to set local variable {name} that does not exist")]
    LocalVariableDoesNotExist { name: String },
    #[error("Argument pattern {arg:?} not supported")]
    UnsupportedArgumentPattern { arg: Box<Pat> },
    #[error("Rebind of unbound variable {name}")]
    RebindOfUnboundVariable { name: String },
    #[error("Calling slot-to-index mapping on non-literal slot {slot:?}")]
    SlotToIndexNonLiteralSlot { slot: Slot },
    #[error("Attempt to initialize unbound local variable {name}")]
    InitializeLocalOnUnboundVariable { name: String },
    #[error("Unsupported pattern in initialize local {pat:?}")]
    UnsupportedPatternInInitializeLocal { pat: Box<Pat> },
    #[error("No early return flag found in function {func:?}")]
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
    #[error("Expected a struct template for this op instead of {kind:?}")]
    ExpectedStructTemplate { kind: Kind },
    #[error("Expected an enum template for this op instead of {kind:?}")]
    ExpectedEnumTemplate { kind: Kind },
    #[error("Unexpected complex path where an identifier was expected {path:?}")]
    UnexpectedComplexPath { path: ExprPath },
    #[error("Missing slot {slot:?} in color map")]
    MissingSlotInColorMap { slot: Slot },
    #[error("Slot {slot:?} missing in type map")]
    SlotMissingInTypeMap { slot: Slot },
    #[error("Slot {slot:?} has conflicting colors")]
    SlotHasConflictingColors { slot: Slot },
    #[error("Slot {slot:?} is read before being written")]
    SlotIsReadBeforeBeingWritten { slot: Slot },
    #[error("Cannot write to a literal slot {ndx}")]
    CannotWriteToLiteral { ndx: usize },
    #[error("Slot {slot:?} is written twice")]
    SlotIsWrittenTwice { slot: Slot },
    #[error("Mismatch in data types (clock domain ignored)")]
    MismatchInDataTypes { lhs: Kind, rhs: Kind },
    #[error("Unsigned cast requires a signed argument")]
    UnsignedCastRequiresSignedArgument,
    #[error("Signed cast requires an unsigned argument")]
    SignedCastRequiresUnsignedArgument,
    #[error("Shift operator requires an unsigned argument")]
    ShiftOperatorRequiresUnsignedArgument,
    #[error("Index value must be unsigned")]
    IndexValueMustBeUnsigned,
    #[error("Expected an array type for this op instead of {kind:?}")]
    ExpectedArrayType { kind: Kind },
    #[error("Match patten value must be a literal")]
    MatchPatternValueMustBeLiteral,
    #[error("Argument count mismatch on call")]
    ArgumentCountMismatchOnCall,
    #[error("Bit cast missing required length")]
    BitCastMissingRequiredLength,
    #[error("Path contains dynamic indices {path:?}")]
    PathContainsDynamicIndices { path: Path },
    #[error("Path does not contain dynamic indices {path:?}")]
    PathDoesNotContainDynamicIndices { path: Path },
    #[error("Mismatched types from dynamic indexing {base:?} and {slot:?}")]
    MismatchedTypesFromDynamicIndexing { base: Kind, slot: Kind },
    #[error("Mismatched bit widths from dynamic indexing {base:?} and {slot:?}")]
    MismatchedBitWidthsFromDynamicIndexing { base: usize, slot: usize },
    #[error("Empty slots are not allowed in Verilog")]
    EmptySlotInVerilog,
    #[error("Functions with no return values not allowed in Verilog")]
    FunctionWithNoReturnInVerilog,
    #[error("Variant {variant} not found in type {ty:?}")]
    VariantNotFoundInType { variant: i64, ty: Kind },
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

#[derive(Debug, Error)]
#[error("RHDL Type Error")]
pub struct RHDLTypeError {
    pub cause: TypeCheck,
    pub src: String,
    pub err_span: SourceSpan,
}

impl Diagnostic for RHDLTypeError {
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
#[error("RHDL Clock Coherence Violation")]
pub struct RHDLClockCoherenceViolation {
    pub src: String,
    pub elements: Vec<(String, SourceSpan)>,
    pub cause_description: String,
    pub cause_span: SourceSpan,
}

impl Diagnostic for RHDLClockCoherenceViolation {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new("These elements are not coherent with the clock"))
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(
            self.elements
                .iter()
                .map(|(name, span)| {
                    miette::LabeledSpan::new_primary_with_span(Some(name.to_string()), *span)
                })
                .chain(std::iter::once(miette::LabeledSpan::new_with_span(
                    Some(self.cause_description.to_string()),
                    self.cause_span,
                ))),
        ))
    }
}

#[derive(Debug, Error)]
#[error("RHDL Type Check Error")]
pub struct RHDLTypeCheckError {
    pub src: String,
    pub lhs_type: String,
    pub lhs_span: SourceSpan,
    pub rhs_type: String,
    pub rhs_span: SourceSpan,
    pub cause_description: String,
    pub cause_span: SourceSpan,
}

impl Diagnostic for RHDLTypeCheckError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new("These two types are not compatible"))
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(
            vec![
                miette::LabeledSpan::new_primary_with_span(
                    Some(self.lhs_type.to_string()),
                    self.lhs_span,
                ),
                miette::LabeledSpan::new_primary_with_span(
                    Some(self.rhs_type.to_string()),
                    self.rhs_span,
                ),
                miette::LabeledSpan::new_with_span(
                    Some(self.cause_description.to_string()),
                    self.cause_span,
                ),
            ]
            .into_iter(),
        ))
    }
}
