use crate::{
    CircuitDescriptor, ClockReset, Digital, DigitalFn, HDLDescriptor,
    circuit::yosys::run_yosys_synth, digital_fn::DigitalFn3, error::RHDLError, hdl::ast::Module,
    ntl::hdl::generate_hdl,
};

pub trait SynchronousDQ: 'static + Sized + Clone {
    type D: Digital;
    type Q: Digital;
}

pub trait SynchronousIO: 'static + Sized + Clone + SynchronousDQ {
    type I: Digital;
    type O: Digital;
    type Kernel: DigitalFn
        + DigitalFn3<A0 = ClockReset, A1 = Self::I, A2 = Self::Q, O = (Self::O, Self::D)>;
}

pub trait Synchronous: 'static + Sized + Clone + SynchronousIO {
    type S: PartialEq + Clone;

    fn init(&self) -> Self::S;

    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;

    fn description(&self) -> String {
        format!("synchronous circuit {}", std::any::type_name::<Self>())
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError>;

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError>;

    fn yosys_check(&self) -> Result<(), RHDLError> {
        run_yosys_synth(self.hdl("top")?)
    }

    fn netlist_hdl(&self, name: &str) -> Result<Module, RHDLError> {
        let descriptor = self.descriptor(name)?;
        generate_hdl(name, &descriptor.ntl)
    }
}
