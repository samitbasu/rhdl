use crate::{
    digital_fn::DigitalFn3, error::RHDLError, flow_graph::optimization::optimize_flow_graph,
    CircuitDescriptor, ClockReset, Digital, DigitalFn, FlowGraph, HDLDescriptor, Tristate,
};

pub trait SynchronousDQZ: 'static + Sized + Clone {
    type D: Digital;
    type Q: Digital;
    type Z: Tristate;
}

pub trait SynchronousIO: 'static + Sized + Clone + SynchronousDQZ {
    type I: Digital;
    type O: Digital;
    type Kernel: DigitalFn
        + DigitalFn3<A0 = ClockReset, A1 = Self::I, A2 = Self::Q, O = (Self::O, Self::D)>;
}

pub trait Synchronous: 'static + Sized + Clone + SynchronousIO {
    type S: Digital;

    fn sim(
        &self,
        clock_reset: ClockReset,
        input: Self::I,
        state: &mut Self::S,
        io: &mut Self::Z,
    ) -> Self::O;

    fn description(&self) -> String {
        format!("synchronous circuit {}", std::any::type_name::<Self>())
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError>;

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }

    fn flow_graph(&self, name: &str) -> Result<FlowGraph, RHDLError> {
        let flow_graph = self.descriptor(name)?.flow_graph.clone();
        optimize_flow_graph(flow_graph)
    }
}
