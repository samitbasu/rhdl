use anyhow::Result;
use rhdl_core::test_module::VerilogDescriptor;
use std::path::Path;

// We want #[derive(Circuit)] --> automatically provide
// an impl of Circuit that includes a Verilog implementation.
// The verilog implementation can be generated for any circuit.
// But we also need to be able to override the default implementation.
//
// It seems like the way to handle this is provide a default
// implementation in the trait itself, and then allow different
// concrete types to override it.
//
// So the Circuit trait would have something like:
pub trait Crct {
    fn as_verilog(&self, path: &Path) -> Result<VerilogDescriptor> {
        // Do default stuff here.
        todo!()
    }
}

// We could make this slightly more general by providing the HDL backend
// as an argument to the trait.

pub enum HDLBackend {
    Verilog,
    VHDL,
    // etc.
}
