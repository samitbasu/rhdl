#![warn(missing_docs)]
use crate::Digital;

pub trait Timed: Digital {}

impl Timed for () {}

impl<T: Timed> Timed for (T,) {}

impl<T0: Timed, T1: Timed> Timed for (T0, T1) {}

impl<T0: Timed, T1: Timed, T2: Timed> Timed for (T0, T1, T2) {}

impl<T0: Timed, T1: Timed, T2: Timed, T3: Timed> Timed for (T0, T1, T2, T3) {}

impl<T: Timed, const N: usize> Timed for [T; N] {}
