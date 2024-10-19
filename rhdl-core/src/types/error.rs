use miette::Diagnostic;
use thiserror::Error;

use crate::{types::path::Path, Kind, TypedBits};

#[derive(Error, Debug, Diagnostic)]
pub enum DynamicTypeError {
    #[error("No field in struct {kind:?} with name {field_name}")]
    NoFieldInStruct { kind: Kind, field_name: String },
    #[error("No variant with name {name} in enum {kind:?}")]
    NoVariantInEnum { name: String, kind: Kind },
    #[error("No variant with discriminant {disc} in enum {kind:?}")]
    NoVariantWithDiscriminant { disc: i64, kind: Kind },
    #[error("Attempt to create a template for something other than an enum {kind:?}")]
    NotAnEnum { kind: Kind },
    #[error("Kind {kind:?} is not an array")]
    NotAnArray { kind: Kind },
    #[error("Kind {kind:?} is not a tuple")]
    NotATuple { kind: Kind },
    #[error("Kind {kind:?} is not a struct")]
    NotAStruct { kind: Kind },
    #[error("Illegal splice of {value:?} into {kind:?} with path {path:?}")]
    IllegalSplice {
        value: TypedBits,
        kind: Kind,
        path: Path,
    },
    #[error("Unsigned cast of {value:?} into {bits} failed")]
    UnsignedCastWithWidthFailed { value: TypedBits, bits: usize },
    #[error("Signed cast of {value:?} into {bits} failed")]
    SignedCastWithWidthFailed { value: TypedBits, bits: usize },
    #[error("Unable to interpret {kind:?} as i64")]
    UnableToInterpretAsI64 { kind: Kind },
    #[error("Signed cast of {value:?} failed")]
    SignedCastFailed { value: TypedBits },
    #[error("Unsigned cast of {value:?} failed")]
    UnsignedCastFailed { value: TypedBits },
    #[error("Cannot get sign bit of value {value:?}")]
    CannotGetSignBit { value: TypedBits },
    #[error("Cannot cast value {value:?} to a bool")]
    CannotCastToBool { value: TypedBits },
    #[error("Cannot get bit {ndx} from {value:?}")]
    CannotGetBit { ndx: usize, value: TypedBits },
    #[error("Cannot set bit {ndx} in {value:?} to {bit}")]
    CannotSetBit {
        ndx: usize,
        value: TypedBits,
        bit: bool,
    },
    #[error("Cannot set bit on a composite value {value:?}")]
    CannotSetBitOnComposite { value: TypedBits },
    #[error("Cannot slice composite value {value:?}")]
    CannotSliceComposite { value: TypedBits },
    #[error("Cannot slice bits {start}:{end} from {value:?}")]
    CannotSliceBits {
        start: usize,
        end: usize,
        value: TypedBits,
    },
    #[error(
        "Binary operation requires both arguments to have compatible types: {lhs:?} and {rhs:?}"
    )]
    BinaryOperationRequiresCompatibleType { lhs: Kind, rhs: Kind },
    #[error("Cannot apply binary operation on values of different signs")]
    BinaryOperationRequiresCompatibleSign,
    #[error("Cannot negate composite value {value:?}")]
    CannotNegateComposite { value: TypedBits },
    #[error("Cannot apply binary operations to composite value {value:?}")]
    CannotApplyBinaryOperationToComposite { value: TypedBits },
    #[error("Cannot negate unsigned value {value:?}")]
    CannotNegateUnsigned { value: TypedBits },
    #[error("Cannot apply shift operator to composite value {value:?}")]
    CannotApplyShiftOperationToComposite { value: TypedBits },
    #[error("Shift amount {value:?} must be unsigned")]
    ShiftAmountMustBeUnsigned { value: TypedBits },
    #[error("Shift amount {value:?} must be less than {max}")]
    ShiftAmountMustBeLessThan { value: TypedBits, max: usize },
    #[error("Reinterpret cast of {value:?} into len {len} failed")]
    ReinterpretCastFailed { value: TypedBits, len: usize },
    #[error("Cannot wrap {value:?} into {kind:?} - it is not a result")]
    CannotWrapResult { value: TypedBits, kind: Kind },
    #[error("Cannot wrap {value:?} into {kind:?} - it is not an option")]
    CannotWrapOption { value: TypedBits, kind: Kind },
}
