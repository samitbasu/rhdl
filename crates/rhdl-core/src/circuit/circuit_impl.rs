use crate::{
    DigitalFn, Timed, circuit::yosys::run_yosys_synth, digital_fn::DigitalFn2, error::RHDLError,
    ntl::hdl::generate_hdl,
};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

/// Circuit Input and Output trait
///
/// This trait defines the input and output types of a circuit, as well as the kernel
/// function that processes the input to produce the output.  It is used in conjunction
/// with the `Circuit` trait to define the behavior of a circuit.
///
/// Note: This trait cannot be auto-derived.  You need to specify it manually.
///
/// If you refer to the canonical diagram:
///
#[doc = badascii_doc::badascii!(r"
       +----------------------------------------------------------------+        
       |                                                                |        
 input |                   +-----------------------+                    | output 
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
/// the `I` type represents the input signal, the `O` type represents the output signal,
///
/// These types, in turn, appear in the type signature of the kernel function, which
/// has type:
///
/// ```rust,ignore
/// fn kernel(input: Self::I, state: &mut Self::S) -> (Self::O, Self::D);
/// ```
///
pub trait CircuitIO: 'static + Sized + Clone + CircuitDQ {
    type I: Timed;
    type O: Timed;
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
/// 1.  The `D` type must have fields named `c1`, `c2`, `c3`.
/// 2.  The `Q` type must have fields named `c1`, `c2`, `c3`.
/// 3.  The type of `D.c1` be the same as the `O` type of `C1`, the type of `D.c2` be the same
///    as the `O` type of `C2`, etc.
/// 4.  The type of `Q.c1` be the same as the `I` type of `C1`, the type of `Q.c2` be the same
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
///
/// # Note
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
pub trait CircuitDQ: 'static + Sized + Clone {
    type D: Timed;
    type Q: Timed;
}

pub trait Circuit: 'static + Sized + Clone + CircuitIO {
    // State for simulation - auto derived
    type S: Clone + PartialEq;

    // Simulation initial state
    fn init(&self) -> Self::S;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;

    // auto derived
    fn description(&self) -> String {
        format!("circuit {}", std::any::type_name::<Self>())
    }

    // auto derived
    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError>;

    // auto derived
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError>;

    fn netlist_hdl(&self, name: &str) -> Result<rhdl_vlog::ModuleList, RHDLError> {
        let descriptor = self.descriptor(name)?;
        generate_hdl(name, &descriptor.ntl)
    }
}
