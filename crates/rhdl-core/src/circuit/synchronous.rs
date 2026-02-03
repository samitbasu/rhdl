//! The core traits for all RHDL [Synchronous] circuits.
//!
//! A synchronous circuit is defined by the [Synchronous] trait, which
//! in turn depends on the [SynchronousIO] and [SynchronousDQ] traits
//! to define the input/output and feedback times of the circuit.
//!
//! The [Synchronous] trait defines methods for initializing the circuit state,
//! simulating the circuit behavior, and generating HDL descriptions.
//!
//! In almost all cases, you will want to `derive` the [Synchronous]
//! trait for your circuit struct using the [rhdl_derive::Synchronous] macro,
//! which will automatically implement the necessary methods based on the
//! fields of your struct and the [kernel] function you provide.
//!
//! # The Canonical Diagram
//!
//! Compared to the diagram for [Circuit](crate::Circuit), the diagram for a synchronous
//! circuit includes an additional mandated clock and reset input, and a fan out of that
//! signal to all of the children of the circuit:
#![doc = badascii_doc::badascii!(r"
        +---------------------------------------------------------------+        
        |   +--+ SynchronousIO::I           SynchronousIO::O +--+       |        
  input |   v               +-----------------------+           v       | output 
 +----->+------------------>|input            output+-------------------+------->
        | +---------------->|c&r      Kernel        |                   |        
        | |            +--->|q                     d+-----+             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
        | |            |    +-----------------------+     |             |        
        | |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        | +-----------+|+-->|c&r                    |     |             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
  clock | |            |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |                   |        
  (c&r) |                   +-----------------------+                   |        
        +---------------------------------------------------------------+        
")]
//! See the [book] for more details on the canonical diagram and how to use
//! synchronous circuits in RHDL.

use crate::{
    ClockReset, Digital, DigitalFn,
    circuit::{
        descriptor::{Descriptor, SyncKind},
        scoped_name::ScopedName,
    },
    digital_fn::DigitalFn3,
    error::RHDLError,
};

/// The [Synchronous] circuit's internal feedback types D and Q.
///
/// This trait defines the types of the feedback signals used in a synchronous
/// circuit. The type D represents the data input to the internal children of
/// the circuit, while the type Q represents the data output from those children.
///
/// The `D` and `Q` types really require structural correspondence, but there is
/// not convenient way to express that in the type system.  So the requirements
/// are documented here instead.  If you provide a `D` and `Q` type that do not
/// meet these requirements, you will generally get compile time or run time errors.
///
/// Suppose that your synchronous circuit `S` has subcircuits `C1`, ... `C3`,
/// defined as
///
/// ```rust, ignore
/// struct S {
///    c1: C1,
///    c2: C2,
///    c3: C3,
/// }
/// ```
///
/// where `C1, C2, C3` are also synchronous circuits.  Then the `D` and `Q` types
/// for `S` must be defined such that
///
/// 1.  The `D` type must have fields named `c1`, `c2`, `c3`.
/// 2.  The `Q` type must have fields named `c1`, `c2`, `c3`.
/// 3.  The type of `Q.c1` be the same as the `O` type of `C1`, the type of `Q.c2` be the same as the `O` type of `C2`, etc.
/// 4.  The type of `D.c1` be the same as the `I` type of `C1`, the type of `D.c2` be the same as the `I` type of `C2`, etc.
///
/// /// That is to say, the `D` type must be structurally equiavelent to:
/// ```rust, ignore
/// struct D {
///    c1: C1::I,
///    c2: C2::I,
///    c3: C3::I,
/// }
/// ```
///
/// and the `Q` type must be structurally equivalent to:
///
/// ```rust, ignore
/// struct Q {
///   c1: C1::I,
///   c2: C2::I,
///   c3: C3::I,
/// }
/// ```
///
/// On the canonical diagram, you can see the `D` and `Q` types as the
/// collections of all of the `d` and `q` signals going to and from the
/// children of the circuit.
///
#[doc = badascii_doc::badascii!(r"
        +---------------------------------------------------------------+        
        |        SynchronousDQ::D           SynchronousDQ::Q ++         |        
  input |          +        +-----------------------+         |         | output 
 +----->+--------+ | +----->|input            output+--------+|+--------+------->
        | +-------+|+------>|c&r      Kernel        |         |         |        
        | |        |   +--->|q                     d+-----+   |         |        
        | |        |   |    +-----------------------+     |   |         |        
        | |        +-+>|                                  |<--+         |        
        | |            |    +-----------------------+     |             |        
        | |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        | +-----------+|+-->|c&r                    |     |             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
  clock | |            |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |                   |        
  (c&r) |                   +-----------------------+                   |        
        +---------------------------------------------------------------+        
")]
pub trait SynchronousDQ: 'static {
    /// The type of the internal data input to the children of the circuit.
    type D: Digital;
    /// The type of the internal data output from the children of the circuit.
    type Q: Digital;
}

/// Synchronous Input and Ouput trait
///
/// This trait defines the input and output types of a synchronous circuit, as well as
/// the kernel function type used to compute the output and internal feedback signals.
/// It is used in conjunction with the [Synchronous] trait to define the behavior of
/// a synchronous circuit.
///
/// Note: This trait cannot be auto-derived.  You need to specify it manually.
///
/// In terms of the canonical diagram:
#[doc = badascii_doc::badascii!(r"
                 SynchronousIO::Kernel +-+                                       
        +-------------------------------+|+-----------------------------+        
        |   +--+ SynchronousIO::I        v  SynchronousIO::O +--+       |        
  input |   v               +-----------------------+           v       | output 
 +----->+------------------>|input            output+-------------------+------->
        | +---------------->|c&r      Kernel        |                   |        
        | |            +--->|q                     d+-----+             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
        | |            |    +-----------------------+     |             |        
        | |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        | +-----------+|+-->|c&r                    |     |             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
  clock | |            |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |                   |        
  (c&r) |                   +-----------------------+                   |        
        +---------------------------------------------------------------+        
")]
/// The [SynchronousIO::I] type represents the input to the circuit,
/// the [SynchronousIO::O] type represents the output from the circuit,
/// and the [SynchronousIO::Kernel] type represents the kernel function.
///
/// The kernel function must take the form of a synthesizable function that
/// has the following type signature:
///
/// ```rust, ignore
/// fn kernel(clock_reset: ClockReset,
///           input: <Self as SynchronousIO>::I,
///          q: <Self as SynchronousDQ>::Q) ->
///            (
///              <Self as SynchronousIO>::O,
///              <Self as SynchronousDQ>::D
///            );
/// ```
pub trait SynchronousIO: 'static + SynchronousDQ {
    /// The type of the input to the circuit.
    type I: Digital;
    /// The type of the output from the circuit.
    type O: Digital;
    /// The type of the kernel function used to compute the output and feedback.
    /// The kernel function must be a synthesizable function that takes a clock and reset signal,
    /// the input signal, and the feedback signal Q, and produces the output signal and feedback signal D.
    /// ```rust, ignore
    /// fn kernel(clock_reset: ClockReset,
    ///           input: <Self as SynchronousIO>::I,
    ///           q: <Self as SynchronousDQ>::Q) ->
    ///             (
    ///               <Self as SynchronousIO>::O,
    ///               <Self as SynchronousDQ>::D
    ///             );
    /// ```
    /// and be annotated with `#[kernel]`.
    type Kernel: DigitalFn
        + DigitalFn3<A0 = ClockReset, A1 = Self::I, A2 = Self::Q, O = (Self::O, Self::D)>;
}

/// The Synchronous circuit trait.
///
/// The other Main Thing.  While every circuit in RHDL is a [Circuit](crate::Circuit),,
/// a subset of circuits are synchronous circuits, which means that they
/// operate in a synchronous time domain with a clock and reset signal.
/// Such circuits implement the [Synchronous] trait, which defines
/// methods for initializing the circuit state, simulating the circuit behavior,
/// and generating HDL descriptions.
///
/// In almost all cases, you will want to `derive` the [Synchronous]
/// trait for your circuit struct.  But you must specify the [SynchronousIO]
/// trait manually, since it cannot be auto-derived.  
///
/// In terms of the canonical diagram, a synchronous circuit is defined by the
/// following structure and types:
#[doc = badascii_doc::badascii!(r"
                 SynchronousIO::Kernel +-+                                       
        +-------------------------------+|+-----------------------------+        
        |   +--+ SynchronousIO::I        v  SynchronousIO::O +--+       |        
  input |   v               +-----------------------+           v       | output 
 +----->+------------------>|input            output+-------------------+------->
        | +---------------->|c&r      Kernel        |                   |        
        | |            +--->|q                     d+-----+             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
        | |            |    +-----------------------+     |             |        
        | |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        | +-----------+|+-->|c&r                    |     |             |        
        | |            |    +-----------------------+     |             |        
        | |     +----->|                                  |             |        
  clock | |     +      |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |    ^              |        
  (c&r) |       +           +-----------------------+    |              |        
        |       ++ SynchronousDQ::Q    SynchronousDQ::D ++              |        
        +---------------------------------------------------------------+        
")]
/// where
///
///  - [SynchronousIO::I] is the input type,
///  - [SynchronousIO::O] is the output type,
///  - [SynchronousDQ::Q] is the child output feedback type, and
///  - [SynchronousDQ::D] is the child input feedback type.
///  - [SynchronousIO::Kernel] is the kernel function type that computes the output and internal feedback.
///
/// In most cases, you will want to derive this trait.  But you may need to write it manually if
/// your circuit has special requirements.
///
/// # Note
/// Note that `Synchronous: 'static + Sized + Clone + SynchronousIO`.  This means
/// that you must be able to freely clone a synchronous circuit struct and that
/// the circuit struct must have a known size at compile time.
///
/// There are implementations of `Synchronous` provided for arrays of circuits, so
/// you can create arrays of synchronous circuits as long as the element type
/// also implements `Synchronous`.
pub trait Synchronous: 'static + Sized + SynchronousIO {
    /// The simulation state type.
    /// This type is used to represent the internal state of the circuit
    /// during simulation.  It must be `PartialEq` and `Clone`.  It holds whatever
    /// state is needed to compute the output of the circuit given it's input.
    /// This state is typically auto-derived by the `Synchronous` derive macro,
    /// and it's typically a struct containing the states of all the subcircuits,
    /// along with their last know outputs.
    type S: PartialEq + Clone;

    /// Initialize the simulation state.
    /// This method returns the initial state of the circuit for simulation.
    /// This is typically auto-derived by the `Synchronous` derive macro,
    /// and it typically initializes the states of all the subcircuits to their
    /// initial states.
    fn init(&self) -> Self::S;

    /// Simulate the circuit given it's input, clock reset, and current state.
    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;

    /// Provides run time reflection of the circuit.
    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError> {
        super::hdl::synchronous::build_synchronous_descriptor(self, scoped_name)
    }

    /// Iterate over the child circuits of this circuit.
    fn children(
        &self,
        _parent_scope: &ScopedName,
    ) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>> {
        std::iter::empty()
    }
}
