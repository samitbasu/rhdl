//! Circuit Descriptors (run time descriptions of circuits)
//!
//! Most of RHDL's compilation and optimization of your circuit designs
//! happen at run time, not when your code is first compiled by Rust.
//!
//! A key part of this is the [Descriptor] type, which describes
//! the interface and implementation of a circuit at run time.
//! It includes information about the input and output kinds,
//! the internal feedback types, the compiled kernel object,
//! and optionally the netlist and HDL description of the circuit.
//!
//! It also provides a way to iterate over the subcircuits of a
//! circuit, allowing for a run time iteration over the (heterogeneous)
//! hierarchy of circuits that make up a design.
//!
//! You can obtain a [Descriptor] for a circuit by calling the
//! `descriptor` method on the circuit, which is part of the
//! [Circuit](crate::Circuit) trait, or part of the [Synchronous](crate::Synchronous) trait.
//!
//! Note also that the [Descriptor] type is generic over a marker type
//! that indicates whether the circuit is asynchronous or synchronous.
//! This allows for type-safe handling of descriptors for different
//! kinds of circuits.
use std::marker::PhantomData;

use crate::{
    HDLDescriptor, Kind, RHDLError,
    circuit::{schematic::Schematic, scoped_name::ScopedName},
    rtl,
};

/// Marker type for asynchronous circuits.
pub struct AsyncKind;
/// Marker type for synchronous circuits.
pub struct SyncKind;

/// Run time description of a circuit.
#[derive(Debug)]
pub struct Descriptor<T> {
    /// The scoped name of the circuit.
    pub name: ScopedName,
    /// The type name of the circuit.
    pub type_name: &'static str,
    /// The kind of the input type.
    pub input_kind: Kind,
    /// The kind of the output type.
    pub output_kind: Kind,
    /// The kind of the internal feedback type to the inputs of the children.
    pub d_kind: Kind,
    /// The kind of the internal feedback type from the outputs of the children.
    pub q_kind: Kind,
    /// The compiled kernel object.
    pub kernel: Option<rtl::Object>,
    /// The HDL (Verilog) description of the circuit, if available.
    pub hdl: Option<HDLDescriptor>,
    /// The schematic description of the circuit
    pub schematic: Option<Schematic>,
    /// Phantom data for the marker type.
    pub _phantom: PhantomData<T>,
}

impl<T> Descriptor<T> {
    /// Get a reference to the HDL (Verilog) description of the circuit, if available.
    pub fn hdl(&self) -> Result<&HDLDescriptor, RHDLError> {
        let hdl = self.hdl.as_ref().ok_or(RHDLError::HDLNotAvailable {
            name: self.name.to_string(),
        })?;
        hdl.modules.checked()?;
        Ok(hdl)
    }
    /// Get a reference to the schematic representation of the circuit, if available.
    pub fn schematic(&self) -> Result<&Schematic, RHDLError> {
        self.schematic
            .as_ref()
            .ok_or(RHDLError::SchematicNotAvailable {
                name: self.name.to_string(),
            })
    }
    /// Check for combinatorial paths in the circuit
    pub fn check_for_combinatorial_paths(&self) -> miette::Result<()> {
        let schematic = self.schematic()?;
        schematic.check_for_combinatorial_io_paths()?;
        Ok(())
    }
}
