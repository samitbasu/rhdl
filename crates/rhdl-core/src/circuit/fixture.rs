//! Export RHDL [Circuit] as a top level module and add custom interface logic.
//!
//! A fixture is a top level module that contains a circuit, and
//! allows you to attach various drivers to drive inputs and
//! observe outputs.  A fixture is helpful when you need to take
//! a RHDL [Circuit] and connect it to some external system (like
//! a testbench or a hardware description language).
//!
//! A fixture contains a circuit, and a set of [Driver] instances.
//! A [Driver] is a fragment of Verilog that can be used to
//! either drive inputs to a circuit, or collect outputs from a circuit.
//! A driver is meant to handle the details of interfacing a circuit
//! to the outside world.
//!
#![doc = badascii_doc::badascii!(r"
+-------+Fixture+--------------------------------+
|  pin +------+                    +------+pin   |
|  +-->|Driver+-+               +->|Driver+--->  |
|I     +------+ |               |  +------+     O|
|N              |               |               U|
|P pin +------+ | I +-------+ O |  +------+pin  T|
|U +-->|Driver+-+-->|Circuit+---+->|Driver+---> P|
|T     +------+ |   +-------+   |  +------+     U|
|S              |               |               T|
|  pin +------+ |               |  +------+pin  S|
|  +-->|Driver+-+               +->|Driver+--->  |
|      +------+                    +------+      |
+------------------------------------------------+
")]
//!
//! You can also use a single [Driver] to provide both inputs and outputs
//! to the circuit.  For example, when using a hard IP core on an FPGA,
//! you may have a single driver that instantiates the hard IP core, and
//! then connects it to the circuit.
//!
#![doc = badascii_doc::badascii!(r"
+-+Fixture+------------------------------+   
|        inputs        ++DRAM++          |   
|+---------------+     |Driver|  Out pins|   
||               |<----+      +----------+-->
||  +-------+    |<----+      |Input pins|   
|+->|Circuit|    +     |      |<---------+--+
|   |       |     +--->|      |Inout pins|   
|   |       +---->|    |      |<---------+-->
|   |       |     +--->|      |          |   
|   +-------+  outputs +------+          |   
+----------------------------------------+   
")]
//!
//! Drivers connect to the input circuit via [MountPoint]s.  A mount point
//! is simply a range of bits on the input or output of the circuit.
//! It is best illustrated with a diagram:
//!
#![doc = badascii_doc::badascii!(r"
                     Input            
pin x+------+        +-+   Mount Point
+--->|      |bits<2> |8|   Input(7..9)
pin y|Driver+------->|7|<------+      
+--->|      |        +-+              
     +------+        | |              
                     |.|              
         Other +---->|.|              
       Drivers       |.|              
                     | |              
                     +-+              
")]
//!
//! Finally, the `bind` macro is a helper to make it easier to connect
//! inputs and outputs of a circuit to the top level of the fixture.
//! It allows you to specify paths on the input and output of the circuit,
//! and automatically creates the necessary drivers and mount points.
//!
//# Example
//!```rust
//!use rhdl::prelude::*;
//!
//!#[kernel]
//!fn adder(a: Signal<(b4, b4), Red>) -> Signal<b4, Red> {
//!    let (a, b) = a.val();
//!    signal(a + b) // Return signal with value
//!}
//!
//!let adder = AsyncFunc::new::<adder>()?;
//!let mut fixture = Fixture::new("adder_top", adder);
//!bind!(fixture, a -> input.val().0);
//!bind!(fixture, b -> input.val().1);
//!bind!(fixture, sum -> output.val());
//!let vlog = fixture.module()?;  
//!```
//!
//! This example creates a top level fixture that looks like this:
//!
#![doc = badascii_doc::badascii!(r"
    +-+Fixture+-------+      
a   |                 |      
+---+---+    +-----+  |      
    |   |    |Adder|  |      
b   |   +--->|     +--+-> sum
+---+---+    +-----+  |      
    |                 |      
    +-----------------+      
")]
//!
//! When exported as Verilog, the fixture will look like this:
//!
//!```verilog
//!module adder_top(input wire [3:0] a, input wire [3:0] b, output wire [3:0] sum);
//!   wire [7:0] inner_input;
//!   wire [3:0] inner_output;
//!   assign inner_input[3:0] = a;
//!   assign inner_input[7:4] = b;
//!   assign sum = inner_output[3:0];
//!   inner inner_inst(.i(inner_input), .o(inner_output));
//!endmodule
//!module inner(input wire [7:0] i, output wire [3:0] o);
//!   assign o = kernel_adder(i);
//!   function [3:0] kernel_adder(input reg [7:0] arg_0);
//!         reg [7:0] r0;
//!         reg [3:0] r1;
//!         reg [3:0] r2;
//!         reg [3:0] r3;
//!         begin
//!            r0 = arg_0;
//!            r1 = r0[3:0];
//!            r2 = r0[7:4];
//!            r3 = r1 + r2;
//!            kernel_adder = r3;
//!         end
//!   endfunction
//!endmodule
//! ```
use super::circuit_impl::Circuit;
use crate::{
    CircuitIO, Digital, Kind, RHDLError,
    types::path::{Path, bit_range, leaf_paths},
};
use miette::Diagnostic;
use quote::{ToTokens, format_ident, quote};
use syn::parse_quote;
use thiserror::Error;

use rhdl_vlog as vlog;

/// Errors that can occur when exporting a circuit as a fixture.
#[derive(Error, Debug, Diagnostic)]
pub enum ExportError {
    /// Multiple drivers to circuit input
    #[error("Multiple drivers to circuit input")]
    MultipleDrivers,
    /// Inputs are not covered in exported core
    #[error("Inputs are not covered in exported core:\n{0}")]
    InputsNotCovered(String),
    /// Wrong constant type provided to input
    #[error("Wrong constant type provided.  Expected {required:?}, and got {provided:?}")]
    WrongConstantType {
        /// The type provided
        provided: Kind,
        /// The type required
        required: Kind,
    },
    /// Attempt to feed a clock signal to a non-clock input
    #[error("Path {path:?} on input is not a clock input - it is of type {kind:?} ")]
    NotAClockInput {
        /// The path to the signal
        path: Path,
        /// The kind of the signal
        kind: Kind,
    },
    #[error(
        "Mismatch in signal width on input: expected {expected} bits, but got {actual} with path {path:?}"
    )]
    /// Mismatch in signal width on input
    SignalWidthMismatchInput {
        /// The expected width
        expected: usize,
        /// The actual width
        actual: usize,
        /// The path to the signal
        path: Path,
    },
    #[error(
        "Mismatch in signal width on output: expected {expected} bits, but got {actual} with path {path:?}"
    )]
    /// Mismatch in signal width on output
    SignalWidthMismatchOutput {
        /// The expected width
        expected: usize,
        /// The actual width
        actual: usize,
        /// The path to the signal
        path: Path,
    },
    /// Attempt to feed a clock signal from a non-clock input
    #[error("Path {path:?} on output is not a clock output, it is of type {kind:?}")]
    NotAClockOutput {
        /// The path to the signal
        path: Path,
        /// The kind of the signal
        kind: Kind,
    },
    /// The circuit cannot be exported as a fixture, due to some BSP specific issue.
    #[error("BSP Error {0}")]
    Custom(anyhow::Error),
}

/// A mount point for a driver.
///
/// A mount point is simply a range of bits on the input or output of the circuit.
/// It is best illustrated with a diagram:
///
#[doc = badascii_doc::badascii!(r"
                     Input            
pin x+------+        +-+   Mount Point
+--->|      |bits<2> |8|   Input(7..9)
pin y|Driver+------->|7|<------+      
+--->|      |        +-+              
     +------+        | |              
                     |.|              
         Other +---->|.|              
       Drivers       |.|              
                     | |              
                     +-+              
")]
///
/// Each input driver collects some inputs from outside the circuit and then presents
/// these as a bit vector to the circuit at the given mount point.  There is an equivalent
/// diagram for outputs.
///
/// Ultimately, all input bits must be covered by exactly one mount point.  Outputs can
/// remain unused.
#[derive(Clone, Debug)]
pub enum MountPoint {
    /// A mount point on the input of the circuit.
    Input(std::ops::Range<usize>),
    /// A mount point on the output of the circuit.
    Output(std::ops::Range<usize>),
}

impl ToTokens for MountPoint {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            MountPoint::Input(range) => {
                let bit_range: vlog::BitRange = range.into();
                quote! {
                    inner_input[#bit_range]
                }
            }
            MountPoint::Output(range) => {
                let bit_range: vlog::BitRange = range.into();
                quote! {
                    inner_output[#bit_range]
                }
            }
        })
    }
}

/// A driver for a RHDL circuit.
///
/// This struct represents a driver.  A driver is a fragment of Verilog that can
/// be used to either drive inputs to a circuit, or collect outputs from a circuit.
/// A driver is meant to handle the details of interfacing a circuit to the outside world.
///
/// A driver may contain multiple input and output ports, and may drive both inputs and
/// collect outputs of a circuit element.  For example, if your FPGA had a hard IP block
/// to control external DDR memory, then you may have a driver that magics the code needed
/// to instantiate and configure that block, and then presents a logical interface to your
/// RHDL [Circuit].  
///
#[doc = badascii_doc::badascii!(r"
+-+Fixture+------------------------------+   
|        inputs        ++DRAM++          |   
|+---------------+     |Driver|  Out pins|   
||               |<----+      +----------+-->
||  +-------+    |<----+      |Input pins|   
|+->|Circuit|    +     |      |<---------+--+
|   |       |     +--->|      |Inout pins|   
|   |       +---->|    |      |<---------+-->
|   |       |     +--->|      |          |   
|   +-------+  outputs +------+          |   
+----------------------------------------+   
")]
/// Practical designs almost always require custom interface logic to connect to physical
/// parts of a system.  A [Driver] is a way to package up that interface logic so that it
/// can be reused in multiple places.
///
/// Note that you can also use a [Driver] to create a cleaner exported HDL description of your
/// design.  To make the resulting Verilog easier to interface to, you can use a [Driver] to
/// extract and rename ports on your circuit, making it easier to connect to the outside world.
#[derive(Clone)]
pub struct Driver<T> {
    marker: std::marker::PhantomData<T>,
    mounts: Vec<MountPoint>,
    /// The ports for this driver.
    ///
    /// These will be the top level ports on the fixture.
    ports: Vec<vlog::Port>,
    /// The HDL for this driver.
    ///
    /// This should be a fragment of Verilog that implements the driver.
    pub hdl: vlog::ItemList,
    /// The constraints for this driver.
    ///
    /// This should be whatever text needs to generated to supply constraints for this driver.
    pub constraints: String,
}

impl<T> Default for Driver<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
            mounts: Default::default(),
            ports: Default::default(),
            hdl: vlog::ItemList::default(),
            constraints: Default::default(),
        }
    }
}

impl<T: CircuitIO> Driver<T> {
    /// Add an input port to this driver.
    ///
    /// The name is the name of the port, and the width is the width in bits.
    pub fn input_port(&mut self, name: &str, width: usize) {
        self.ports.push(vlog::port(
            vlog::Direction::Input,
            vlog::wire_decl(name, vlog::unsigned_width(width)),
        ));
    }
    /// Add an output port to this driver.
    ///
    /// The name is the name of the port, and the width is the width in bits.
    pub fn output_port(&mut self, name: &str, width: usize) {
        self.ports.push(vlog::port(
            vlog::Direction::Output,
            vlog::wire_decl(name, vlog::unsigned_width(width)),
        ));
    }
    /// Add an inout port to this driver.
    ///
    /// The name is the name of the port, and the width is the width in bits.
    pub fn inout_port(&mut self, name: &str, width: usize) {
        self.ports.push(vlog::port(
            vlog::Direction::Inout,
            vlog::wire_decl(name, vlog::unsigned_width(width)),
        ));
    }
    /// Connect this driver's input to an inner input path on the circuit.
    pub fn write_to_inner_input(&mut self, path: &Path) -> Result<MountPoint, RHDLError> {
        let (bits, _) = bit_range(<T::I as Digital>::static_kind(), path)?;
        let mount = MountPoint::Input(bits);
        self.mounts.push(mount.clone());
        Ok(mount)
    }
    /// Connect this driver's output to an inner output path on the circuit.
    pub fn read_from_inner_output(&mut self, path: &Path) -> Result<MountPoint, RHDLError> {
        let (bits, _) = bit_range(<T::O as Digital>::static_kind(), path)?;
        let mount = MountPoint::Output(bits);
        self.mounts.push(mount.clone());
        Ok(mount)
    }
}

/// Create a passthrough driver for an output path.
///
/// This creates a top level output port with the given name,
/// and connects it to the given path on the circuit.
pub fn passthrough_output_driver<T: Circuit>(
    name: &str,
    path: &Path,
) -> Result<Driver<T>, RHDLError> {
    let (bits, _) = bit_range(<T::O as Digital>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.output_port(name, bits.len());
    let output = driver.read_from_inner_output(path)?;
    let name = format_ident!("{}", name);
    driver.hdl = parse_quote!(assign #name = #output;);
    Ok(driver)
}

/// Create a passthrough driver for an input path.
///
/// This creates a top level input port with the given name,
/// and connects it to the given path on the circuit.
pub fn passthrough_input_driver<T: Circuit>(
    name: &str,
    path: &Path,
) -> Result<Driver<T>, RHDLError> {
    let (bits, _) = bit_range(<T::I as Digital>::static_kind(), path)?;
    let mut driver = Driver::default();
    driver.input_port(name, bits.len());
    let input = driver.write_to_inner_input(path)?;
    let name = format_ident!("{}", name);
    driver.hdl = parse_quote!(assign #input = #name;);
    Ok(driver)
}

/// Create a constant driver for an input path.
///
/// The value is the constant value to drive on the given path.
/// The type of the value must match the type of the path.
pub fn constant_driver<T: Circuit, S: Digital>(
    val: S,
    path: &Path,
) -> Result<Driver<T>, RHDLError> {
    let (_bits, sub_kind) = bit_range(<T::I as Digital>::static_kind(), path)?;
    if S::static_kind() != sub_kind {
        return Err(RHDLError::ExportError(ExportError::WrongConstantType {
            provided: S::static_kind(),
            required: sub_kind,
        }));
    }
    let mut driver = Driver::<T>::default();
    let input = driver.write_to_inner_input(path)?;
    let lit: vlog::LitVerilog = val.typed_bits().into();
    driver.hdl = parse_quote!(assign #input = #lit;);
    Ok(driver)
}

/// A top level fixture for a RHDL circuit.  
///
/// A [Fixture] contains a circuit, and allows you to attach
/// various [Driver] instances to drive inputs and observe outputs.
/// A [Fixture] is helpful when you need to take a RHDL [Circuit] and
/// connect it to some external system (like a real FPGA!).  
///
#[doc = badascii_doc::badascii!(r"
+-------+Fixture+--------------------------------+
|  pin +------+                    +------+pin   |
|  +-->|Driver+-+               +->|Driver+--->  |
|I     +------+ |               |  +------+     O|
|N              |               |               U|
|P pin +------+ | I +-------+ O |  +------+pin  T|
|U +-->|Driver+-+-->|Circuit+---+->|Driver+---> P|
|T     +------+ |   +-------+   |  +------+     U|
|S              |               |               T|
|  pin +------+ |               |  +------+pin  S|
|  +-->|Driver+-+               +->|Driver+--->  |
|      +------+                    +------+      |
+------------------------------------------------+
")]
///
/// The [Driver] instances are simply fragments of Verilog and constraints
/// that are combined to form a complete Verilog module for the fixture.
///
/// An important usability feature of the [Fixture] is that it allows you to
/// decode paths that feed the circuit inputs and outputs.  It will also assert
/// that all input paths are covered by exactly one driver.
///
///# The Bind Macro
///
///Often, you will want to simply connect inputs and outputs of a circuit to
/// the top level of the fixture.  For example, if you have an AXI bus interface,
/// you need to export the various AXI signals at the top level using the nomenclature
/// expected by your tooling.  The `bind` macro is a helper to make this easier.
pub struct Fixture<T> {
    name: String,
    drivers: Vec<Driver<T>>,
    circuit: T,
}

fn build_coverage_error(kind: Kind, coverage: &[bool]) -> String {
    let paths = leaf_paths(&kind, Path::default());
    let mut details = String::new();
    for path in paths {
        let (bits, _) = bit_range(kind, &path).unwrap();
        let covered = coverage[bits].iter().all(|b| *b);
        if !covered {
            details.push_str(&format!("Path {path:?} is not covered\n"));
        }
    }
    details
}

impl<T: Circuit> Fixture<T> {
    /// Create a new fixture with the given name and circuit.
    pub fn new(name: &str, t: T) -> Self {
        Self {
            name: name.into(),
            drivers: vec![],
            circuit: t,
        }
    }
    /// Get the name of the fixture.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Add a driver to the fixture.
    pub fn add_driver(&mut self, driver: Driver<T>) {
        self.drivers.push(driver)
    }
    /// Add a passthrough driver for an input path.
    ///
    /// The name is the name of the input port that will be fed through to the circuit
    /// on the given path.
    pub fn pass_through_input(&mut self, name: &str, path: &Path) -> Result<(), RHDLError> {
        self.add_driver(passthrough_input_driver::<T>(name, path)?);
        Ok(())
    }
    /// Add a passthrough driver for an output path.
    ///
    /// The name is the name of the output port that will be driven from the circuit
    /// on the given path.
    pub fn pass_through_output(&mut self, name: &str, path: &Path) -> Result<(), RHDLError> {
        self.add_driver(passthrough_output_driver::<T>(name, path)?);
        Ok(())
    }
    /// Add a constant driver for an input path.
    ///
    /// The value is the constant value to drive on the given path.
    /// The type of the value must match the type of the path.
    pub fn constant_input<S: Digital>(&mut self, val: S, path: &Path) -> Result<(), RHDLError> {
        self.add_driver(constant_driver::<T, S>(val, path)?);
        Ok(())
    }
    /// Generate the Verilog module for this fixture.
    ///
    /// This will combine the Verilog from the circuit and all the drivers,
    /// and will check that all inputs are covered by exactly one driver.
    ///
    pub fn module(&self) -> Result<vlog::ModuleList, RHDLError> {
        let ports = self.drivers.iter().flat_map(|t| t.ports.iter());
        // Declare the mount points for the circuit
        let i_kind = <<T as CircuitIO>::I as Digital>::static_kind();
        let inputs_len = i_kind.bits();
        let outputs_len = <<T as CircuitIO>::O as Digital>::static_kind().bits();
        let declarations = [
            vlog::maybe_decl_wire(inputs_len, "inner_input"),
            vlog::maybe_decl_wire(outputs_len, "inner_output"),
        ]
        .into_iter()
        .flatten();
        let mut i_cover = vec![false; inputs_len];
        self.drivers
            .iter()
            .flat_map(|x| x.mounts.iter())
            .flat_map(|m| match m {
                MountPoint::Input(range) => Some(range.clone()),
                _ => None,
            })
            .try_for_each(|range| {
                for bit in range {
                    if i_cover[bit] {
                        return Err::<(), RHDLError>(ExportError::MultipleDrivers.into());
                    }
                    i_cover[bit] = true;
                }
                Ok(())
            })?;
        if i_cover.iter().any(|b| !b) {
            let coverage = build_coverage_error(i_kind, &i_cover);
            return Err(ExportError::InputsNotCovered(coverage).into());
        }
        let driver_items = self.drivers.iter().flat_map(|x| &x.hdl.items);
        // Instantiate the thing
        let desc = self.circuit.descriptor("inner".into())?;
        let hdl = desc.hdl()?;
        let verilog = &hdl.modules;
        let name_ident = format_ident!("{}", self.name);
        let inner_ident = format_ident!("{}", hdl.name);
        let module: vlog::ModuleList = parse_quote! {
            module #name_ident (#(#ports),*);
                #( #declarations ;)*
                #( #driver_items ;)*
                #inner_ident inner_inst (
                    .i(inner_input),
                    .o(inner_output)
                );
            endmodule
            #verilog
        };
        Ok(module)
    }
    /// Generate the constraints for this fixture.
    ///
    /// This will combine the constraints from all the drivers.
    ///
    pub fn constraints(&self) -> String {
        let xdc = self
            .drivers
            .iter()
            .map(|x| x.constraints.clone())
            .collect::<Vec<_>>();
        xdc.join("\n")
    }
    /// Get an input/output value pair for the circuit wrapped by this fixture.
    pub fn io(&self) -> (<T as CircuitIO>::I, <T as CircuitIO>::O) {
        (
            <T::I as Digital>::dont_care(),
            <T::O as Digital>::dont_care(),
        )
    }
}
