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

// This would lead to a runtime failure if you do not have support for the
// required backend.  Maybe that is OK?  In that case, we could change it to

pub trait Crct2 {
    fn as_hdl(&self, path: &Path, backend: HDLBackend) -> Result<HDLDescriptor> {}
}

// The downside of this is if a subcircuit implements `as_hdl` directly (to provide a )
// custom implementation, we don't know that it will behave as expected.  What if we
// used a marker trait.

pub trait AsVerilog {
    fn as_verilog(&self, path: &Path) -> Result<VerilogDescriptor> {}
}

// This is slightly better.  Assuming the trait is propagated to the children of the
// current circuit.  Which is a hard thing to achieve.
