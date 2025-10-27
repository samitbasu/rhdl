use crate::{
    CircuitDescriptor, HDLDescriptor, RHDLError, circuit::circuit_descriptor::CircuitType,
};

mod asynchronous;
mod synchronous;

pub fn build_hdl(descriptor: &CircuitDescriptor) -> Result<HDLDescriptor, RHDLError> {
    match descriptor.circuit_type {
        CircuitType::Synchronous => synchronous::build_synchronous_hdl(descriptor),
        CircuitType::Asynchronous => asynchronous::build_asynchronous_hdl(descriptor),
    }
}
