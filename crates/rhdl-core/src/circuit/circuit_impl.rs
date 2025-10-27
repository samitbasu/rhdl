//! The core traits for all RHDL [Circuit]s.
//!
//! A circuit is defined by the [Circuit] trait, which in turn
//! depends on the [CircuitIO] and [CircuitDQ] traits to define
//! the input/output and feedback types of the circuit.
//!
//! The [Circuit] trait also provides methods for simulation,
//! HDL generation, and run time reflection.
//!
//! In almost all cases, you will want to `derive` the [Circuit]
//! trait using the `#[derive(Circuit)]` macro.
//!
//! # The Canonical Diagram
//!
//! The following diagram illustrates the structure of a circuit in RHDL.
//! It shows the relationships between the input/output types, the
//! feedback types, the kernel function, and the subcircuits.
//!
#![doc = badascii_doc::badascii!(r"
       +----------------------------------------------------------------+        
       |   +--+ CircuitIO::I                   CircuitIO::O +--+        |        
 input |   v               +-----------------------+           v        | output 
+----->+------------------>|input            output+--------------------+------->
       |                   |         Kernel        |                    |        
       |              +--->|q                     d+-----+              |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
")]
//! See the [book]() for a detailed explanation of this diagram.
use crate::{
    DigitalFn, Timed, circuit::descriptor::Descriptor, digital_fn::DigitalFn2, error::RHDLError,
};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

/// Circuit Input and Output trait
///
/// This trait defines the input and output types of a circuit, as well as the kernel
/// function that processes the input to produce the output.  It is used in conjunction
/// with the [Circuit] trait to define the behavior of a circuit.
///
/// Note: This trait cannot be auto-derived.  You need to specify it manually.
///
/// If you refer to the canonical diagram:
///
#[doc = badascii_doc::badascii!(r"
       +----------------------------------------------------------------+        
       |   +--+ CircuitIO::I                   CircuitIO::O +--+        |        
 input |   v               +-----------------------+           v        | output 
+----->+------------------>|input            output+--------------------+------->
       |                   |         Kernel        |                    |        
       |              +--->|q                     d+-----+              |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
")]
/// the [CircuitIO::I] type represents the input signal, the [CircuitIO::O] type represents the output signal,
///
/// These types, in turn, appear in the type signature of the kernel function, which
/// has the form:
///
/// ```rust,ignore
/// fn kernel(input: <Self as CircuitIO>::I, q: <Self as CircuitDQ>::Q) ->
///        (<Self as CircuitIO>::O, <Self as CircuitDQ>::D);
/// ```
///
pub trait CircuitIO: 'static + CircuitDQ {
    /// The input type of the circuit
    type I: Timed;
    /// The output type of the circuit
    type O: Timed;
    /// The kernel function type of the circuit.  Must have a signature
    /// of the form:
    /// ```rust,ignore
    /// fn kernel(input: <Self as CircuitIO>::I, q: <Self as CircuitDQ>::Q) ->
    ///        (<Self as CircuitIO>::O, <Self as CircuitDQ>::D);
    /// ```
    /// and be annotated with `#[kernel]`.
    ///
    type Kernel: DigitalFn + DigitalFn2<A0 = Self::I, A1 = Self::Q, O = (Self::O, Self::D)>;
}

/// The Circuit internal feedback trait
///
/// This trait defines the internal feedback types of a circuit.  It works in conjunction
/// with the `Circuit` trait to define the behavior of a circuit.  The `D` and `Q` types
/// _must_ satisfy the following requirements:
///
/// Suppose that the circuit `C` has subcircuits defined as
///
/// ```rust,ignore
/// struct C {
///   c1: C1,
///   c2: C2,
///   c3: C3,
/// }
/// ```
///
/// where `C1`, `C2`, and `C3` are themselves circuits.  Then the `D` and `Q` types must
/// be defined such that:
///
/// 1.  The `D` type must have fields named `c1`, `c2`, `c3`.
/// 2.  The `Q` type must have fields named `c1`, `c2`, `c3`.
/// 3.  The type of `Q.c1` be the same as the `O` type of `C1`, the type of `Q.c2` be the same
///    as the `O` type of `C2`, etc.
/// 4.  The type of `D.c1` be the same as the `I` type of `C1`, the type of `D.c2` be the same
///    as the `I` type of `C2`, etc.
///
/// That is to say, the `D` type must be structurally equiavelent to:
///
///```rust,ignore
/// struct D {
///    c1: C1::I,
///    c2: C2::I,
///    c3: C3::I,
/// }
///```
///
/// and the `Q` type must be structurally equivalent to:
///
/// ```rust,ignore
/// struct Q {
///   c1: C1::O,
///   c2: C2::O,
///   c3: C3::O,
/// }
/// ```
///
/// Because you cannot express these as constraints in Rust, you must either ensure them
/// manually, or use the `CircuitDQ` macro to autoderive them.  Referring to the
/// canonical diagram, RHDL assumes that the fields of `D` and `Q` correspond to
/// the child circuits of the circuit, and that they are connected as shown in the diagram.
///
#[doc = badascii_doc::badascii!(r"
       +----------------------------------------------------------------+        
       |                                                                |        
 input |                   +-----------------------+                    | output 
+----->+------------------>|input            output+--------------------+------->
       | CircuitDQ::Q      |         Kernel        |  CircuitDQ::D      |        
       |     +        +--->|q                     d+-----+    +         |        
       |     |        |    +-----------------------+     |    |         |        
       |     +------->|                                  |<---+         |        
       |              |    +-----------------------+     |              |        
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
")]
///
/// # Technical Note
///
/// It would have been possible to use tuples for `D` and `Q`, so that
/// the types would have been automatically satisfied by construction.
/// Unfortunately, this makes the kernel signature unwieldy, and makes
/// the resulting code _much_ harder to read.  But using these structs is
/// definitely something that feels non-idiomatic in Rust.
///
/// Also, because the types are exposed through a trait, you can always
/// define them yourself, and do not need to use the `CircuitDQ` derive
/// macro.
pub trait CircuitDQ: 'static {
    /// A type which contains the feedback data from the circuit to its subcircuits.
    type D: Timed;
    /// A type which contains the feedback data from the subcircuits to the circuit.
    type Q: Timed;
}

/// The Circuit Trait
///
/// The Main Thing.  The [Circuit] trait defines a circuit in RHDL.  You must either
/// provide an implementation of it, or `derive` it using the `Circuit` macro.  This
/// trait adds the methods needed to simulate the circuit, generate an HDL description
/// of it, manipulate it at run time (by providing reflection), and other things.  In
/// terms of the canonical diagram, a circuit is defined as:
#[doc = badascii_doc::badascii!(r"
                CircuitIO::I    CircuitIO::Kernel          CircuitIO::O          
                      +                 +                          +         
                      |                 |                          |    
       +-------------+|+---------------+|+------------------------+|+---+        
       |              |                 v                          |    |        
 input |              v    +-----------------------+               v    | output 
+----->+------------------>|input            output+--------------------+------->
       | CircuitDQ::Q      |         Kernel        |  CircuitDQ::D      |        
       |     +        +--->|q                     d+-----+    +         |        
       |     |        |    +-----------------------+     |    |         |        
       |     +------->|                                  |<---+         |        
       |              |    +-----------------------+     |              |        
       | q.child_1 +> +----+o        child_1      i|<----+ <+ d.child_1 |        
       |              |    +-----------------------+     |              |        
       |              |                                  |              |        
       |              |    +-----------------------+     |              |        
       | q.child_2 +> +----+o        child_2      i|<----+ <+ d.child_2 |        
       |                   +-----------------------+                    |        
       |                                                                |        
       +----------------------------------------------------------------+        
                           ^                                                     
            impl Circuit +-+                                                     
")]
/// where the [CircuitIO::I], [CircuitIO::O], [CircuitDQ::D], and [CircuitDQ::Q] types
/// are defined by the [CircuitIO] and [CircuitDQ] traits, and the [CircuitIO::Kernel]
/// type is a function which processes the input and feedback to produce the output
/// and feedback.  In almost all cases, you will want to `derive` this trait using
/// the `#[derive(Circuit)]` macro.
///
/// # Note
/// Note that `Circuit: 'static + Sized + Clone + CircuitIO`.  This means that you must be
/// able to freely clone a circuit.  A circuit must also be Sized.
///
/// There are implementations of `Circuit` provided for arrays of circuits, so you can
/// create arrays of circuits and use them as a single circuit.
pub trait Circuit: 'static + CircuitIO {
    /// The simulation state type
    /// This type is used to represent the state of the circuit during simulation.
    /// It must be `Clone` and `PartialEq`.  It holds whatever state is needed to
    /// compute the output of the circuit element given it's input.  This state is
    /// typically autoderived by the `Circuit` macro, and is typically a struct
    /// containing the states of the subcircuits, along with their last known outputs.
    type S: Clone + PartialEq;

    /// Simulation initial state
    /// This method returns the initial state of the circuit for simulation.
    /// This is typically auto-derived by the `Circuit` macro, and is typically
    /// a struct containing the initial states of the subcircuits.
    fn init(&self) -> Self::S;

    /// Simulate the circuit given it's current state and input.
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;

    /// A human readable description of the circuit, unique for each type.
    fn description(&self) -> String {
        format!("circuit {}", std::any::type_name::<Self>())
    }

    /// Provides run time reflection of the circuit.
    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError>;

    /// Hardware Description Language (HDL) representation of the circuit.
    ///
    /// This method returns the HDL representation of the circuit.  This is typically
    /// auto-derived by the `Circuit` macro, and is typically a Verilog representation.
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let descriptor = self.descriptor(name)?;
        super::hdl::build_hdl(&descriptor)
    }

    /// Generate a netlist HDL representation of the circuit.
    ///
    /// This method generates a netlist representation of the circuit in HDL (typically Verilog).
    fn netlist(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let descriptor = self.descriptor(name)?;
        crate::ntl::hdl::build_hdl(name, &descriptor.ntl)
    }

    fn children(&self) -> impl Iterator<Item = (&str, &dyn Descriptor)>;
}
