use std::collections::BTreeMap;

use crate::{Circuit, HDLDescriptor, RHDLError, Synchronous};

pub fn build_hdl<C: Circuit>(
    circuit: &C,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    todo!()
}

pub fn build_synchronous_hdl<S: Synchronous>(
    synchronous: &S,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    todo!()
}
