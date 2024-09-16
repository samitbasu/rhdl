use crate::{error::RHDLError, types::tristate::Tristate, Digital, DigitalFn, FlowGraph, Timed};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

pub type CircuitUpdateFn<C> =
    fn(<C as CircuitIO>::I, <C as CircuitDQ>::Q) -> (<C as CircuitIO>::O, <C as CircuitDQ>::D);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HDLKind {
    Verilog,
}

pub trait CircuitIO: 'static + Sized + Clone {
    type I: Timed;
    type O: Timed;
}

pub trait CircuitDQ: 'static + Sized + Clone {
    type D: Timed;
    type Q: Timed;
}

pub trait Circuit: 'static + Sized + Clone + CircuitIO + CircuitDQ {
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

    // auto derived
    fn descriptor(&self) -> Result<CircuitDescriptor, RHDLError>;

    // auto derived
    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor, RHDLError>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
}
