use crate::{CircuitDescriptor, HDLDescriptor, RHDLError};

pub trait Descriptor {
    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError>;
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError>;
    fn netlist(&self, name: &str) -> Result<HDLDescriptor, RHDLError>;
}
