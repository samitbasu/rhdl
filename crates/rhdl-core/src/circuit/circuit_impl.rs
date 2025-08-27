use crate::{
    DigitalFn, Timed, circuit::yosys::run_yosys_synth, digital_fn::DigitalFn2, error::RHDLError,
    hdl::ast::Module, ntl::hdl::generate_hdl,
};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

pub trait CircuitIO: 'static + Sized + Clone + CircuitDQ {
    type I: Timed;
    type O: Timed;
    type Kernel: DigitalFn + DigitalFn2<A0 = Self::I, A1 = Self::Q, O = (Self::O, Self::D)>;
}

pub trait CircuitDQ: 'static + Sized + Clone {
    type D: Timed;
    type Q: Timed;
}

pub trait Circuit: 'static + Sized + Clone + CircuitIO {
    // State for simulation - auto derived
    type S: Clone + PartialEq;

    // Simulation initial state
    fn init(&self) -> Self::S;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;

    // auto derived
    fn description(&self) -> String {
        format!("circuit {}", std::any::type_name::<Self>())
    }

    // auto derived
    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError>;

    // auto derived
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError>;

    fn yosys_check(&self) -> Result<(), RHDLError> {
        run_yosys_synth(self.hdl("top")?)
    }

    fn netlist_hdl(&self, name: &str) -> Result<Module, RHDLError> {
        let descriptor = self.descriptor(name)?;
        generate_hdl(name, &descriptor.ntl)
    }
}
