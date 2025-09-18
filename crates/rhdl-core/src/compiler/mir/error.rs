use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::{
    Kind, RHDLError, TypedBits,
    ast::{
        SourcePool,
        ast_impl::{ExprCall, ExprPath, FunctionId, Pat},
    },
    builder::BinOp,
    common::symtab::{LiteralId, RegisterId},
    rhif::spec::{AluBinary, AluUnary, OpCode, Slot, SlotKind},
    rtl::spec::{Operand, OperandKind},
    types::path::Path,
};

use super::{compiler::ScopeIndex, ty::SignFlag};

#[derive(Error, Debug, Diagnostic)]
pub enum TypeCheck {
    #[error("A request was made for .val() on something that is not a signal")]
    ExpectedSignalValue,
    #[error("Literal with explicit type {typ:?} is inferred as {kind:?} instead")]
    InferredLiteralTypeMismatch { typ: Kind, kind: Kind },
    #[error("Unable to determine type of this item")]
    #[diagnostic(help("Please provide an explicit type annotation"))]
    UnableToDetermineType,
    #[error(
        "Literal {literal:?} is outside the range of the inferred type {flag:?} {len} bit integer"
    )]
    LiteralOutsideInferredRange {
        literal: TypedBits,
        flag: SignFlag,
        len: usize,
    },
    #[error("Partially initialized value is used in an operation")]
    #[diagnostic(help(
        "You need to initialize all elements of a value before using it in an operation"
    ))]
    PartiallyInitializedValue,
    #[error("Path mismatch in type inference")]
    #[diagnostic(help(
        "There is a type mismatch in the type inference step.  This is likely due 
        the use of nested enums in pattern matching, like Some(Ok(x)).  RHDL can 
        only destructure one level of enums at a time.  Replace with Some(y) and
        use a nested match expression to further destructure the inner type."
    ))]
    PathMismatchInTypeInference,
    #[error("Cannot determine the sign of this value")]
    ExpectedSignFlag,
    #[error(
        "Expression causes an overflow in bit widths (currently a maximum of 128 bits is supported)"
    )]
    BitWidthOverflow,
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
    #[error("Non self assign binary operation found in assign_binop code {op:?}")]
    NonSelfAssignBinop { op: BinOp },
    #[error("Unexpected binary op in self assign {op:?}")]
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
    #[error("Cannot write to a literal {ndx:?}")]
    CannotWriteToRHIFLiteral { ndx: LiteralId<SlotKind> },
    #[error("Cannot write to a RTL literal {ndx:?}")]
    CannotWriteToRTLLiteral { ndx: LiteralId<OperandKind> },
    #[error("Slot {slot:?} is written twice")]
    SlotIsWrittenTwice { slot: Slot },
    #[error("Mismatch in data types (clock domain ignored) {lhs:?} and {rhs:?}")]
    MismatchInDataTypes { lhs: Kind, rhs: Kind },
    #[error("Unsigned cast requires a signed argument")]
    UnsignedCastRequiresSignedArgument,
    #[error("Signed cast requires an unsigned argument")]
    SignedCastRequiresUnsignedArgument,
    #[error("Shift operator requires an unsigned argument instead of {kind:?}")]
    ShiftOperatorRequiresUnsignedArgument { kind: Kind },
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
    #[error("Symbol table has no entry for slot {slot:?}")]
    SymbolTableIsIncomplete { slot: Slot },
    #[error("Symbol table has no entry for operand {operand:?}")]
    SymbolTableIsIncompleteForRTL { operand: Operand },
    #[error("Unable to infer clock domain for retime operation {op:?}")]
    UnableToInferClockDomainForRetime { op: OpCode },
    #[error("Empty slot passed to code generator in RTL")]
    EmptySlotInRTL,
    #[error("Function {fn_id:?} not found in object map")]
    MissingObject { fn_id: FunctionId },
    #[error("Invalid signed cast in RTL {lhs:?} and {arg:?} with length {len}")]
    InvalidSignedCast {
        lhs: Operand,
        arg: Operand,
        len: usize,
    },
    #[error("Malformed RTL flow graph returned")]
    MalformedRTLFlowGraph,
    #[error("VM encountered an uninitialized RHIF register {r:?}")]
    UninitializedRegister { r: RegisterId<SlotKind> },
    #[error("VM encountered an uninitialized RTL register {r:?}")]
    UninitializedRTLRegister { r: RegisterId<OperandKind> },
    #[error("VM cannot write a non-empty value to an empty slot")]
    CannotWriteNonEmptyValueToEmptySlot,
    #[error("VM encountered a discriminant {discriminant:?} with no matching arm")]
    NoMatchingArm { discriminant: TypedBits },
    #[error("VM expected argument of type {expected:?} but found {arg:?}")]
    ArgumentTypeMismatchOnCall { arg: Kind, expected: Kind },
    #[error("VM return argument for function was not initialized")]
    ReturnSlotNotInitialized,
    #[error("VM encountered a non-empty/empty value/argument conflict")]
    NonemptyToEmptyArgumentMismatch,
    #[error("VM encountered an error on a binary operation")]
    BinaryOperatorError(Box<RHDLError>),
    #[error("VM encountered an error on a unary operation")]
    UnaryOperatorError(Box<RHDLError>),
    #[error("RTL symbol table is incomplete for operand {operand:?}")]
    RTLSymbolTableIsIncomplete { operand: Operand },
    #[error("RTL Resize operation is invalid {lhs:?} and {arg:?} with length {len}")]
    InvalidResize {
        lhs: Operand,
        arg: Operand,
        len: usize,
    },
    #[error("Shift operator requires an unsigned argument constant instead of {shift}")]
    ShiftOperatorRequiresUnsignedArgumentConstant { shift: i64 },
    #[error("Wrap opcode is missing the inferred type")]
    WrapMissingKind,
    #[error("Wrap opcode requires a result kind, not {kind:?}")]
    WrapRequiresResultKind { kind: Kind },
    #[error("Wrap opcode requires an option kind, not {kind:?}")]
    WrapRequiresOptionKind { kind: Kind },
    #[error("Attempted to select based on an uninitialized value")]
    SelectOnUninitializedValue { value: TypedBits },
    #[error("Invalid arguments to Xops operation {a:?} and {b:?}")]
    InvalidXopsKind { a: Kind, b: Kind },
    #[error("Result of an Xops (xadd, xsub, xmul) cannot be assigned to a literal")]
    XopsResultMustBeRegister,
    #[error("Argument of pad operation must be either a Bits or SignedBits value, not {a:?}")]
    InvalidPadKind { a: Kind },
    #[error("Argument of cut operation must be either a Bits or SignedBits value, not {a:?}")]
    InvalidCutKind { a: Kind },
    #[error("Dynamic index in path {path:?} is a literal value")]
    DynamicIndexHasLiteral { path: Path },
    #[error("Multiple writes to a single register {op:?}")]
    MultipleWritesToRegister { op: crate::ntl::spec::Wire },
    #[error("Symbol table is incomplete")]
    IncompleteSymbolTable,
    #[error("Loop Isolation Algorithm Failed")]
    LoopIsolationAlgorithmFailed,
    #[error("Netlist contains an incomplete symbol table")]
    IncompleteSymbolTableInNetList,
    #[error("Cannot coerce empty to an integer")]
    CannotCoerceEmptyToInteger,
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
        "Only .all(), .any(), .xor(), .as_unsigned(), .as_signed(), .resize() and .pad() are supported in kernels"
    ))]
    UnsupportedMethodCall,
    #[error("Unsupported path with arguments")]
    #[diagnostic(help("Use a path without generic arguments here, if possible"))]
    UnsupportedPathWithArguments,
    #[error("RHDL does not support the use of unary operators on this type")]
    #[diagnostic(help(
        "You cannot roll your own {op:?} operator in RHDL.  You should write a kernel and call it as a regular function."
    ))]
    RollYourOwnUnary { op: AluUnary },
    #[error("RHDL does not support the use of binary operators on this type")]
    #[diagnostic(help(
        "You cannot roll your own binary operator in RHDL.  You should write a kernel and call it as a regular function."
    ))]
    RollYourOwnBinary,
    #[error("RHDL does not support functions with empty return types")]
    #[diagnostic(help(
        "You cannot have a function with an empty return type in RHDL.  You should return a value or a tuple of values."
    ))]
    EmptyReturnForFunction,
    #[error("RHDL cannot infer the number of bits in an xext/xshl/xshr operation")]
    #[diagnostic(help(
        "Use a turbofish to indicate how many bits you want to prepend (msb), e.g., a.xext::<U4>() or how many bits to shift left or right, as a.xshr<U2>()"
    ))]
    XOpsWithoutLength,
}

#[derive(Debug, Error, Diagnostic)]
pub enum ClockError {
    #[error("Clock domain mismatch in binary operation {op:?}")]
    #[diagnostic(help(
        "You cannot perform binary operations on signals from different clock domains"
    ))]
    BinaryOperationClockMismatch { op: AluBinary },
    #[error("Clock domain mismatch in unary operation {op:?}")]
    #[diagnostic(help(
        "You cannot perform unary operation {op:?} on signals from different clock domains"
    ))]
    UnaryOperationClockMismatch { op: AluUnary },
    #[error("Clock domain mismatch in assignment")]
    #[diagnostic(help("You cannot assign signals from different clock domains"))]
    AssignmentClockMismatch,
    #[error("Clock domain mismatch in cast operation")]
    #[diagnostic(help("You cannot cast signals from different clock domains"))]
    CastClockMismatch,
    #[error("Clock domain mismatch in retime operation")]
    #[diagnostic(help(
        "You cannot retime signals from different clock domains.  You may need a clock domain crosser in your design."
    ))]
    RetimeClockMismatch,
    #[error("Clock domain mismatch in select operation")]
    #[diagnostic(help(
        "A select operation (if) requires the selection signal and both branches to be in the same clock domain"
    ))]
    SelectClockMismatch,
    #[error("Clock domain mismatch in index operation")]
    #[diagnostic(help("You cannot index signals from different clock domains"))]
    IndexClockMismatch,
    #[error("Clock domain analysis failed to resolve the clock domain for this signal")]
    #[diagnostic(help(
        "You need to provide a clock domain for this expression - rhdl cannot determine what clock domain it belongs to.  This usually indicates that the value is ultimately unused."
    ))]
    UnresolvedClock,
    #[error("Clock domain mismatch in tuple operation")]
    #[diagnostic(help(
        "This tuple operation is mapping signals from one clock domain to another, which is not allowed.  You can have multiple clock domains in a tuple."
    ))]
    TupleClockMismatch,
    #[error("Clock domain mismatch in array operation")]
    #[diagnostic(help(
        "All elements of an array must be in a single clock domain.  Use a tuple if you want to hold multiple clock domains."
    ))]
    ArrayClockMismatch,
    #[error("Clock domain mismatch in match statement")]
    #[diagnostic(help(
        "All branches of a match statement, the discriminant, and the result must be in the same clock domain"
    ))]
    CaseClockMismatch,
    #[error("Clock domain mismatch in enum operation")]
    #[diagnostic(help("All fields of an enum must be in the same clock domain"))]
    EnumClockMismatch,
    #[error("Clock domain mismatch in struct operation")]
    #[diagnostic(help(
        "The supplied field in the struct does not match the expected clock domain for that field"
    ))]
    StructClockMismatch,
    #[error("Clock domain mismatch in splice operation")]
    #[diagnostic(help(
        "In a splice, the original and resulting values must have matching clock domain structures, and the spliced data and the replaced data must also have matching clock domain structures"
    ))]
    SpliceClockMismatch,
    #[error("Clock domain mismatch in call to external function")]
    #[diagnostic(help(
        "The clock domain of the input and output signals must match the clock domains of the inputs for the function"
    ))]
    ExternalClockMismatch,
    #[error("Clock domain mismatch in wrap operation")]
    #[diagnostic(help(
        "The clock domain of the input signal must match the clock domain of the output signal"
    ))]
    WrapClockMismatch,
}

#[derive(Debug, Error)]
#[error("RHDL Syntax Error")]
pub struct RHDLSyntaxError {
    pub cause: Syntax,
    pub src: SourcePool,
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
    pub src: SourcePool,
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
    pub src: SourcePool,
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
#[error("RHDL Clock Domain Violation")]
pub struct RHDLClockDomainViolation {
    pub src: SourcePool,
    pub elements: Vec<(String, SourceSpan)>,
    pub cause: ClockError,
    pub cause_span: SourceSpan,
}

impl Diagnostic for RHDLClockDomainViolation {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.help()
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(
            self.elements
                .iter()
                .map(|(name, span)| {
                    miette::LabeledSpan::new_primary_with_span(Some(name.to_string()), *span)
                })
                .chain(std::iter::once(miette::LabeledSpan::new_with_span(
                    Some(self.cause.to_string()),
                    self.cause_span,
                ))),
        ))
    }
}

#[derive(Debug, Error)]
#[error("RHDL Partial Initialization Error")]
pub struct RHDLPartialInitializationError {
    pub src: SourcePool,
    pub err_span: SourceSpan,
    pub fn_span: SourceSpan,
    pub details: String,
}

impl Diagnostic for RHDLPartialInitializationError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(format!(
            "You need to initialize all elements of a value before using it in an operation:\n{}",
            self.details
        )))
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(
            std::iter::once(miette::LabeledSpan::new_primary_with_span(
                Some(self.details.to_string()),
                self.err_span,
            ))
            .chain(std::iter::once(miette::LabeledSpan::new_with_span(
                Some("Function definition".to_string()),
                self.fn_span,
            ))),
        ))
    }
}

#[derive(Debug, Error)]
#[error("RHDL Type Check Error")]
pub struct RHDLTypeCheckError {
    pub src: SourcePool,
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
