use crate::{
    error::RHDLError, root_descriptor, types::tristate::Tristate, Digital, DigitalFn, Timed,
};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

pub type CircuitUpdateFn<C> =
    fn(<C as CircuitIO>::I, <C as Circuit>::Q) -> (<C as CircuitIO>::O, <C as Circuit>::D);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HDLKind {
    Verilog,
}

pub trait CircuitIO: 'static + Sized + Clone {
    type I: Timed;
    type O: Timed;
}

pub trait Circuit: 'static + Sized + Clone + CircuitIO {
    type D: Timed;
    type Q: Timed;

    // auto derived as the sum of NumZ of the children
    type Z: Tristate;

    type Update: DigitalFn;

    const UPDATE: CircuitUpdateFn<Self> = |_, _| unimplemented!();

    // State for simulation - auto derived
    type S: Digital;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O;

    // auto derived
    fn name(&self) -> &'static str;

    // Default provides the root descriptor, but children are added via proc macro
    fn descriptor(&self) -> Result<CircuitDescriptor, RHDLError> {
        root_descriptor(self)
    }

    // auto derived
    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor, RHDLError>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
}
