use crate::circuit::circuit_impl::Tristate;
use crate::types::digital::Digital;
use crate::{util::hash_id, Kind};
use std::collections::HashMap;

use super::circuit_impl::Circuit;

#[derive(Clone, Debug)]
pub struct CircuitDescriptor {
    pub unique_name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub num_tristate: usize,
    pub tristate_offset_in_parent: usize,
    pub children: HashMap<String, CircuitDescriptor>,
}

impl CircuitDescriptor {
    pub fn add_child<C: Circuit>(&mut self, name: &str, circuit: &C) {
        self.children.insert(name.into(), circuit.descriptor());
    }
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
        num_tristate: C::Z::N,
        tristate_offset_in_parent: 0,
        children: Default::default(),
    }
}
