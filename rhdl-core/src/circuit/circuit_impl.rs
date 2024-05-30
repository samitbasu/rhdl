use crate::{Digital, DigitalFn, Timed};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

pub type CircuitUpdateFn<C> =
    fn(<C as CircuitIO>::I, <C as Circuit>::Q) -> (<C as CircuitIO>::O, <C as Circuit>::D);

pub type CircuitInitFn<C> = fn(<C as CircuitIO>::I) -> (<C as CircuitIO>::O, <C as Circuit>::D);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HDLKind {
    Verilog,
}

pub trait Tristate: Default + Clone + Copy {
    const N: usize;
}

impl Tristate for () {
    const N: usize = 0;
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

    const UPDATE: CircuitUpdateFn<Self>; // = |_, _| (Default::default(), Default::default());

    const INIT: CircuitInitFn<Self>; // = |_| (Default::default(), Default::default());

    // State for simulation - auto derived
    type S: PartialEq + Clone;

    // Simulation update - auto derived
    // The simulation function will now be:
    // This is the init tree
    // let (o, d) = self.init(i);
    // let q.a = self.a.init(d.a);
    // let q.b = self.b.init(d.b);
    // let mut state = (q, self.a.)
    // Then we do the update thing
    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O;

    //
    fn init_state(&self) -> Self::S {
        Default::default()
    }

    // auto derived
    fn name(&self) -> &'static str;

    // auto derived
    fn descriptor(&self) -> CircuitDescriptor;

    // auto derived
    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
}
