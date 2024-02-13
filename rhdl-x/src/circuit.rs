use anyhow::bail;
use anyhow::Result;
use rhdl_bits::Bits;
use rhdl_core::{Digital, DigitalFn, Kind};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::backend::verilog::root_verilog;

pub type CircuitUpdateFn<C> =
    fn(<C as Circuit>::I, <C as Circuit>::Q) -> (<C as Circuit>::O, <C as Circuit>::D);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HDLKind {
    Verilog,
}

pub trait Tristate: Digital {
    type Mask: Digital;
}

impl Tristate for Bits<8> {
    type Mask = Bits<8>;
}

impl Tristate for () {
    type Mask = ();
}

pub struct BufZ<T: Tristate> {
    value: T,
    mask: T::Mask,
}

impl Default for BufZ<()> {
    fn default() -> Self {
        Self {
            value: (),
            mask: (),
        }
    }
}

pub trait Circuit: 'static + Sized + Clone {
    // Input type - not auto derived
    type I: Digital;
    // Output type - not auto derived
    type O: Digital;
    // IO type - not auto derived
    type IO: Tristate;

    type Update: DigitalFn;
    const UPDATE: CircuitUpdateFn<Self>;

    // Outputs of internal circuitry - auto derived
    type Q: Digital;
    // Inputs of internal circuitry - auto derived
    type D: Digital;

    // State for simulation - auto derived
    type S: Default + PartialEq + Clone;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, z_in: Self::IO, state: &mut Self::S)
        -> (Self::O, BufZ<Self::IO>);

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
