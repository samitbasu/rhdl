use anyhow::Result;
use rhdl_core::{Digital, DigitalFn, Kind};
use std::hash::{Hash, Hasher};

pub trait Circuit: 'static + Sized + Clone {
    // Input type - not auto derived
    type I: Digital;
    // Output type - not auto derived
    type O: Digital;

    // Outputs of internal circuitry - auto derived
    type Q: Digital;
    // Inputs of internal circuitry - auto derived
    type D: Digital;

    type Update: DigitalFn;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D);

    // State for simulation - auto derived
    type S: Default + PartialEq + Clone;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }

    fn name(&self) -> &'static str;

    fn descriptor(&self) -> CircuitDescriptor {
        CircuitDescriptor {
            unique_name: format!(
                "{}_{:x}",
                self.name(),
                hash_id(std::any::TypeId::of::<Self>())
            ),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
        }
    }

    fn verilog(self) -> Result<String> {
        crate::verilog::verilog(self)
    }

    fn components(&self) -> impl Iterator<Item = (String, CircuitDescriptor)>;

    fn child_verilog(self) -> impl Iterator<Item = Result<String>>;
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
}
