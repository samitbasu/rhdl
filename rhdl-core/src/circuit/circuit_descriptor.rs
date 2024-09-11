use super::circuit_impl::Circuit;
use crate::flow_graph::flow_graph_impl::FlowGraph;
use crate::types::digital::Digital;
use crate::types::tristate::Tristate;
use crate::{build_rtl_flow_graph, compile_design, CompilationMode, RHDLError, Synchronous};
use crate::{util::hash_id, Kind};
use std::collections::HashMap;

// A few notes on the circuit descriptor struct
// The idea here is to capture the details on the circuit in such
// a way that it can be manipulated at run time.  This means that
// information encoded in the type system must be lifted into the
// runtime description.  And the repository for that information
// is the CircuitDescriptor struct.  We cannot, for example, iterate
// over the types that make up our children.
#[derive(Clone, Debug)]
pub struct CircuitDescriptor {
    pub unique_name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub d_kind: Kind,
    pub q_kind: Kind,
    pub num_tristate: usize,
    pub tristate_offset_in_parent: usize,
    pub update_flow_graph: FlowGraph,
    pub children: HashMap<String, CircuitDescriptor>,
}

impl CircuitDescriptor {
    pub fn add_child<C: Circuit>(&mut self, name: &str, circuit: &C) -> Result<(), RHDLError> {
        self.children.insert(name.into(), circuit.descriptor()?);
        Ok(())
    }
    pub fn add_synchronous<S: Synchronous>(
        &mut self,
        name: &str,
        circuit: &S,
    ) -> Result<(), RHDLError> {
        self.children.insert(name.into(), circuit.descriptor()?);
        Ok(())
    }
}

pub fn root_descriptor<C: Circuit>(circuit: &C) -> Result<CircuitDescriptor, RHDLError> {
    let module = compile_design::<C::Update>(CompilationMode::Asynchronous)?;
    let update_fg = build_rtl_flow_graph(&module);
    Ok(CircuitDescriptor {
        unique_name: format!(
            "{}_{:x}",
            circuit.name(),
            hash_id(std::any::TypeId::of::<C>())
        ),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        d_kind: C::D::static_kind(),
        q_kind: C::Q::static_kind(),
        num_tristate: C::Z::N,
        update_flow_graph: update_fg,
        tristate_offset_in_parent: 0,
        children: Default::default(),
    })
}
