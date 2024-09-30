use crate::{
    error::RHDLError, flow_graph::optimization::optimize_flow_graph, CircuitDescriptor, ClockReset,
    Digital, DigitalFn, FlowGraph, HDLDescriptor, HDLKind, Tristate,
};

pub type SynchronousUpdateFn<C> = fn(
    ClockReset,
    <C as SynchronousIO>::I,
    <C as SynchronousDQ>::Q,
) -> (<C as SynchronousIO>::O, <C as SynchronousDQ>::D);

pub trait SynchronousIO: 'static + Sized + Clone {
    type I: Digital;
    type O: Digital;
}

pub trait SynchronousDQ: 'static + Sized + Clone {
    type D: Digital;
    type Q: Digital;
}

pub trait Synchronous: 'static + Sized + Clone + SynchronousIO + SynchronousDQ {
    type Z: Tristate;

    type Update: DigitalFn;

    const UPDATE: SynchronousUpdateFn<Self> = |_, _, _| unimplemented!();

    type S: Digital;

    fn sim(
        &self,
        clock_reset: ClockReset,
        input: Self::I,
        state: &mut Self::S,
        io: &mut Self::Z,
    ) -> Self::O;

    fn name(&self) -> String;

    fn descriptor(&self) -> Result<CircuitDescriptor, RHDLError>;

    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor, RHDLError>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }

    fn flow_graph(&self) -> Result<FlowGraph, RHDLError> {
        let flow_graph = self.descriptor()?.flow_graph.clone();
        optimize_flow_graph(flow_graph)
    }
}
