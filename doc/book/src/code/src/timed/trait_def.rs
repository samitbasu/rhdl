use rhdl::prelude::*;

// ANCHOR: circuit_io_trait
pub trait CircuitIO: 'static + Sized + Clone + CircuitDQ {
    /// The input type of the circuit
    type I: Timed;
    /// The output type of the circuit
    type O: Timed;
    // snip...
}
// ANCHOR_END: circuit_io_trait

// ANCHOR: timed_trait
pub trait Timed: Digital {}
// ANCHOR_END: timed_trait

// ANCHOR: signal_def
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Signal<T: Digital, C: Domain> {
    val: T,
    domain: std::marker::PhantomData<C>,
}
// ANCHOR_END: signal_def

// ANCHOR: domain_trait
pub trait Domain: Copy + PartialEq + 'static + Default {
    fn color() -> Color;
}
// ANCHOR_END: domain_trait

// ANCHOR: color_enum
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
}
// ANCHOR_END: color_enum

// ANCHOR: synchronous_io_impl
pub trait SynchronousIO: 'static + Sized + Clone + SynchronousDQ {
    /// The input type of the synchronous circuit
    type I: Digital;
    /// The output type of the synchronous circuit
    type O: Digital;
    // snip...
}
// ANCHOR_END: synchronous_io_impl
