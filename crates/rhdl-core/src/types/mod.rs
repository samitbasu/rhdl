//! Core RHDL types.
#![warn(missing_docs)]
pub mod bit_string;
//pub mod bitz;
pub mod clock;
pub mod clock_reset;
/// The core Digital trait and implementations for standard types.
pub mod digital;
/// The trait used to describe synthesizable functions
pub mod digital_fn;
/// Time domain marker trait and color implementations.
pub mod domain;
pub mod error;
pub mod kernel;
/// Run time type representation for RHDL types.
pub mod kind;
pub mod path;
pub mod reset;
pub mod reset_n;
/// A signal carrying a Digital value in a specific time domain.
pub mod signal;
pub(crate) mod svg;
/// Marker trait for types in which all elements belong to a time domain
pub mod timed;
/// Struct that holds a Digital value, along with a timestamp
pub mod timed_sample;
/// Struct that holds a [Kind](crate::types::kind::Kind) and a bit representation
pub mod typed_bits;
