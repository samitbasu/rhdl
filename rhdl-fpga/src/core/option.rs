//! Functions for dealing with options
//!
//! This module provides some useful functions
//! for destructuring options.
use rhdl::prelude::*;

#[kernel]
/// Unpacks an [Option<T>] into a tag (or valid flag)
/// of `bool`, and the underlying `T`.  Requires that
/// you provide a value for `T` that is returned for
/// the `None` variant.
pub fn unpack<T: Digital>(opt: Option<T>, fallback: T) -> (bool, T) {
    match opt {
        None => (false, fallback),
        Some(t) => (true, t),
    }
}

#[kernel]
/// Packs a tag and a data value back into an [Option<T>].  
/// The data value argument is ignored if the tag is false.
pub fn pack<T: Digital>(valid: bool, data: T) -> Option<T> {
    if valid {
        Some(data)
    } else {
        None
    }
}

#[kernel]
/// Returns the tag of an [Option<T>].
pub fn is_some<T: Digital>(x: Option<T>) -> bool {
    match x {
        Some(_) => true,
        None => false,
    }
}
