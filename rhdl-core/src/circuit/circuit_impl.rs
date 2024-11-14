use crate::{
    digital_fn::{DigitalFn2, NoKernel2},
    error::RHDLError,
    flow_graph::optimization::optimize_flow_graph,
    trace_pop_path, trace_push_path, Digital, DigitalFn, FlowGraph, Timed,
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

    // Return a top level flow graph for this circuit, optimized
    fn flow_graph(&self, name: &str) -> Result<FlowGraph, RHDLError> {
        let flow_graph = self.descriptor(name)?.flow_graph.clone();
        optimize_flow_graph(flow_graph)
    }
}

// Blanket implementation for an array of circuits.
impl<T: CircuitIO, const N: usize> CircuitIO for [T; N] {
    type I = [T::I; N];
    type O = [T::O; N];
    type Kernel = NoKernel2<Self::I, Self::Q, (Self::O, Self::D)>;
}

impl<T: CircuitIO, const N: usize> CircuitDQ for [T; N] {
    type D = [T::I; N];
    type Q = [T::O; N];
}

impl<T: Circuit, const N: usize> Circuit for [T; N] {
    type S = [T::S; N];

    fn init(&self) -> Self::S {
        array_init::array_init(|i| self[i].init())
    }

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("array");
        let mut output = [T::O::init(); N];
        for i in 0..N {
            output[i] = self[i].sim(input[i], &mut state[i]);
        }
        trace_pop_path();
        output
    }

    fn description(&self) -> String {
        format!("array of {} x {}", N, self[0].description())
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut descriptors = Vec::new();
        for i in 0..N {
            descriptors.push(self[i].descriptor(&format!("{}[{}]", name, i))?);
        }
        Ok(CircuitDescriptor::new(name, descriptors))
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let mut descriptors = Vec::new();
        for i in 0..N {
            descriptors.push(self[i].hdl(&format!("{}[{}]", name, i))?);
        }
        Ok(HDLDescriptor::new(name, descriptors))
    }
}
