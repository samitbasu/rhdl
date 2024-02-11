use anyhow::bail;
use anyhow::Result;
use rhdl_core::{Digital, DigitalFn, Kind};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::backend::verilog::root_verilog;

pub type CircuitUpdateFn<C> =
    fn(<C as Circuit>::I, <C as Circuit>::Q) -> (<C as Circuit>::O, <C as Circuit>::D);

pub type CircuitLinkFn<C> = fn(<C as Circuit>::IO) -> <C as Circuit>::C;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HDLKind {
    Verilog,
}

pub trait Circuit: 'static + Sized + Clone {
    // Input type - not auto derived
    type I: Digital;
    // Output type - not auto derived
    type O: Digital;
    // InputOutput type - not auto derived
    type IO: Digital;

    type Update: DigitalFn;
    const UPDATE: CircuitUpdateFn<Self>;

    type Link: DigitalFn;
    const LINK: CircuitLinkFn<Self>;

    // Outputs of internal circuitry - auto derived
    type Q: Digital;
    // Inputs of internal circuitry - auto derived
    type D: Digital;
    // InputOutputs of internal circuitry - auto derived
    type C: Digital;

    // State for simulation - auto derived
    type S: Default + PartialEq + Clone;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }

    // auto derived
    fn name(&self) -> &'static str;

    // auto derived
    fn descriptor(&self) -> CircuitDescriptor;

    // auto derived
    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor>;
}

fn hash_id(fn_id: std::any::TypeId) -> u64 {
    // Hash the typeID into a 64 bit unsigned int
    let mut hasher = fnv::FnvHasher::default();
    fn_id.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Debug)]
pub struct CircuitDescriptor {
    pub unique_name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub children: HashMap<String, CircuitDescriptor>,
}

pub fn root_descriptor<C: Circuit>(circuit: &C) -> CircuitDescriptor {
    CircuitDescriptor {
        unique_name: format!(
            "{}_{:x}",
            circuit.name(),
            hash_id(std::any::TypeId::of::<C>())
        ),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        children: Default::default(),
    }
}

#[derive(Clone, Debug)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: String,
    pub children: HashMap<String, HDLDescriptor>,
}

pub fn root_hdl<C: Circuit>(circuit: &C, kind: HDLKind) -> Result<HDLDescriptor> {
    match kind {
        HDLKind::Verilog => root_verilog(circuit),
    }
}

impl std::fmt::Display for HDLDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.body)?;
        for hdl in self.children.values() {
            writeln!(f, "{}", hdl)?;
        }
        Ok(())
    }
}

pub struct NoLink {}

impl DigitalFn for NoLink {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        todo!()
    }
}

pub fn no_link<C: Circuit>(_: C::IO) -> C::C {
    Default::default()
}
