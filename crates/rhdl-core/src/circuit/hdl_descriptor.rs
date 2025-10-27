//! A runtime hardware description of a [Circuit] or [Synchronous] circuit.
//!
//! This module defines the `HDLDescriptor` struct, which captures
//! the HDL representation of a circuit, including its name, body,
//! and any child circuits it may have.  It also includes a method
//! to convert the descriptor into a flat list of Verilog modules.
//!
//! You typically don't create `HDLDescriptor` instances directly.
use rhdl_vlog;

/// A hardware description of a circuit.
///
/// This struct captures the HDL representation of a circuit,
/// including its name, body, and any child circuits it may have.
#[derive(Clone, Hash)]
pub struct HDLDescriptor {
    /// The unique name of the circuit.
    pub name: String,
    /// The list of modules that make up this circuit.
    pub modules: rhdl_vlog::ModuleList,
}
