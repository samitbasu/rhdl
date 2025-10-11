#![warn(missing_docs)]
use crate::Digital;

/// A marker trait for types in which all elements are `Timed`.
///
/// The `Timed` trait is used as a marker to indicate that a type has elements that belong to some
/// time domain (they do not all have to belong to the same time domain, just _some_ time domain).  
///
/// Asynchronous circuits will require that input and output types `impl Timed`.  In general, this
/// that either:
///
/// - The type is a `Signal<T, C>` for some `T: Digital` and `C: Domain`, in which case it represents
/// a value `T` in the time domain `C`.
/// - The type is zero-sized, and can be freely moved between time domains.  This includes `()`.
///
/// The `Timed` trait is automatically implemented for tuples and arrays of `Timed` types.
///
/// You will need to #[derive(Timed)] for your own types that contain `Timed` fields.  It is part of the
/// contract of an asynchronous circuit that the RHDL compiler will check for time domain consistency.
pub trait Timed: Digital {}

impl Timed for () {}

impl<T: Timed> Timed for (T,) {}

impl<T0: Timed, T1: Timed> Timed for (T0, T1) {}

impl<T0: Timed, T1: Timed, T2: Timed> Timed for (T0, T1, T2) {}

impl<T0: Timed, T1: Timed, T2: Timed, T3: Timed> Timed for (T0, T1, T2, T3) {}

impl<T: Timed, const N: usize> Timed for [T; N] {}
